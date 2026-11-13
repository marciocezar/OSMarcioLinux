//! Listagem de diretórios via GIO (tipo MIME, ícone do tema, permissões,
//! dono) e ordenação. Sem interface gráfica.

use gtk4::gio;
use gtk4::gio::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub link_target: Option<String>,
    pub size: u64,
    pub modified: Option<i64>,
    pub mode: u32,
    pub owner: String,
    pub content_type: String,
    pub description: String,
    pub icon: Option<gio::Icon>,
    /// Itens da lixeira: caminho original e data de exclusão.
    pub trash: Option<crate::trash::Trashed>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortKey {
    #[default]
    Name,
    Type,
    Size,
    Modified,
}

const ATTRS: &str = "standard::name,standard::display-name,standard::type,standard::size,standard::is-symlink,\
standard::symlink-target,standard::content-type,standard::icon,time::modified,unix::mode,owner::user";

/// Lê um item (sem seguir link para os metadados; segue só para saber se o alvo é pasta).
pub fn stat(path: &Path) -> Result<Entry, String> {
    let f = gio::File::for_path(path);
    let info = f
        .query_info(
            ATTRS,
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            None::<&gio::Cancellable>,
        )
        .map_err(|e| crate::fsops::describe(&e))?;
    Ok(from_info(path.to_path_buf(), &info))
}

fn from_info(path: PathBuf, info: &gio::FileInfo) -> Entry {
    let is_symlink = info.is_symlink();
    let mut is_dir = info.file_type() == gio::FileType::Directory;
    let mut ctype = info
        .content_type()
        .map(|s| s.to_string())
        .unwrap_or_default();
    if is_symlink && let Ok(md) = std::fs::metadata(&path) {
        is_dir = md.is_dir();
        if is_dir {
            ctype = "inode/directory".into();
        }
    }
    let description = if ctype.is_empty() {
        "desconhecido".into()
    } else {
        gio::content_type_get_description(&ctype).to_string()
    };
    let icon = if is_dir {
        Some(gio::ThemedIcon::new("folder").upcast())
    } else {
        info.icon()
    };
    Entry {
        name: info.display_name().to_string(),
        path,
        is_dir,
        is_symlink,
        link_target: info.symlink_target().map(|p| p.display().to_string()),
        size: info.size().max(0) as u64,
        modified: info.modification_date_time().map(|d| d.to_unix()),
        mode: info.attribute_uint32("unix::mode"),
        owner: info
            .attribute_string("owner::user")
            .map(|s| s.to_string())
            .unwrap_or_default(),
        content_type: ctype,
        description,
        icon,
        trash: None,
    }
}

/// Lista uma pasta. Erros (permissão, inexistente, volume removido) voltam como texto.
pub fn list_dir(dir: &Path, show_hidden: bool) -> Result<Vec<Entry>, String> {
    let f = gio::File::for_path(dir);
    let en = f
        .enumerate_children(
            ATTRS,
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            None::<&gio::Cancellable>,
        )
        .map_err(|e| crate::fsops::describe(&e))?;
    let mut out = Vec::new();
    while let Some(info) = en
        .next_file(None::<&gio::Cancellable>)
        .map_err(|e| crate::fsops::describe(&e))?
    {
        let name = info.name();
        if !show_hidden && name.to_string_lossy().starts_with('.') {
            continue;
        }
        out.push(from_info(dir.join(&name), &info));
    }
    Ok(out)
}

/// Pastas primeiro; depois a chave escolhida; empate por nome (natural, sem caixa).
pub fn sort(entries: &mut [Entry], key: SortKey) {
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| {
            let by_name = || a.name.to_lowercase().cmp(&b.name.to_lowercase());
            match key {
                SortKey::Name => by_name(),
                SortKey::Type => a.description.cmp(&b.description).then_with(by_name),
                SortKey::Size => b.size.cmp(&a.size).then_with(by_name),
                SortKey::Modified => b.modified.cmp(&a.modified).then_with(by_name),
            }
        })
    });
}

pub fn human_size(n: u64) -> String {
    const U: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{n} B")
    } else {
        format!("{v:.1} {}", U[i]).replace('.', ",")
    }
}

/// "rwxr-xr-x (755)".
pub fn perms(mode: u32) -> String {
    let bits = ['r', 'w', 'x'];
    let s: String = (0..9)
        .map(|i| {
            if mode & (1 << (8 - i)) != 0 {
                bits[i % 3]
            } else {
                '-'
            }
        })
        .collect();
    format!("{s} ({:03o})", mode & 0o777)
}

pub fn date(ts: Option<i64>) -> String {
    ts.and_then(|t| gtk4::glib::DateTime::from_unix_local(t).ok())
        .and_then(|d| d.format("%d/%m/%Y %H:%M").ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "—".into())
}

/// Tipos que o Files nunca "abre" executando (segurança).
pub fn is_executable_type(ctype: &str) -> bool {
    matches!(
        ctype,
        "application/x-executable"
            | "application/x-pie-executable"
            | "application/x-sharedlib"
            | "application/x-shellscript"
            | "application/x-desktop"
            | "application/x-msdownload"
            | "application/vnd.microsoft.portable-executable"
            | "application/x-perl"
            | "text/x-python"
            | "application/x-python-code"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(name: &str, dir: bool, size: u64, m: i64) -> Entry {
        Entry {
            name: name.into(),
            path: PathBuf::from(name),
            is_dir: dir,
            is_symlink: false,
            link_target: None,
            size,
            modified: Some(m),
            mode: 0o644,
            owner: String::new(),
            content_type: String::new(),
            description: if dir { "pasta".into() } else { "texto".into() },
            icon: None,
            trash: None,
        }
    }

    #[test]
    fn folders_first_then_key() {
        let mut v = vec![
            e("b.txt", false, 10, 3),
            e("Z", true, 0, 1),
            e("a.txt", false, 99, 2),
            e("c", true, 0, 9),
        ];
        sort(&mut v, SortKey::Name);
        assert_eq!(
            v.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(),
            ["c", "Z", "a.txt", "b.txt"]
        );
        sort(&mut v, SortKey::Size);
        assert_eq!(v[2].name, "a.txt");
        sort(&mut v, SortKey::Modified);
        assert_eq!((v[0].name.as_str(), v[2].name.as_str()), ("c", "b.txt"));
    }

    #[test]
    fn formatting() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1536), "1,5 KB");
        assert_eq!(perms(0o755), "rwxr-xr-x (755)");
        assert_eq!(perms(0o600), "rw------- (600)");
        assert!(is_executable_type("application/x-shellscript"));
        assert!(!is_executable_type("text/plain"));
    }

    #[test]
    fn lists_real_directory_with_unicode_and_symlinks() {
        let d = std::env::temp_dir().join(format!("osmf-list-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("pasta")).unwrap();
        std::fs::write(d.join("ação ✓.txt"), "x").unwrap();
        std::fs::write(d.join(".oculto"), "").unwrap();
        std::os::unix::fs::symlink(d.join("pasta"), d.join("atalho")).unwrap();
        let v = list_dir(&d, false).unwrap();
        assert_eq!(v.len(), 3);
        let a = v.iter().find(|x| x.name == "atalho").unwrap();
        assert!(a.is_symlink && a.is_dir);
        assert!(v.iter().any(|x| x.name == "ação ✓.txt" && x.size == 1));
        assert_eq!(list_dir(&d, true).unwrap().len(), 4);
        assert!(list_dir(&d.join("nao-existe"), false).is_err());
        std::fs::remove_dir_all(d).unwrap();
    }
}
