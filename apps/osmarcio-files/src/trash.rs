//! Lixeira do usuário segundo a especificação FreeDesktop Trash:
//! `$XDG_DATA_HOME/Trash/{files,info}`. O envio é feito pelo GIO
//! (`fsops::trash`); listar e restaurar são feitos aqui, lendo os
//! `.trashinfo`, para não depender do gvfs (que a imagem não instala).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct Trashed {
    /// Nome dentro de Trash/files.
    pub id: String,
    /// Arquivo/pasta dentro da lixeira.
    pub path: PathBuf,
    /// Onde estava antes.
    pub original: PathBuf,
    pub deleted: String,
}

pub fn trash_dir() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(data.join("Trash"))
}

/// Decodifica %XX de um caminho do .trashinfo.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn parse_trashinfo(text: &str) -> Option<(PathBuf, String)> {
    let mut path = None;
    let mut date = String::new();
    let mut in_section = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_section = l == "[Trash Info]";
        } else if in_section {
            if let Some(v) = l.strip_prefix("Path=") {
                path = Some(PathBuf::from(percent_decode(v)));
            } else if let Some(v) = l.strip_prefix("DeletionDate=") {
                date = v.replace('T', " ");
            }
        }
    }
    path.filter(|p| p.is_absolute()).map(|p| (p, date))
}

pub fn list_in(trash: &Path) -> Vec<Trashed> {
    let Ok(rd) = std::fs::read_dir(trash.join("info")) else {
        return Vec::new();
    };
    let mut v: Vec<Trashed> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(".trashinfo")?.to_string();
            let (original, deleted) = parse_trashinfo(&std::fs::read_to_string(e.path()).ok()?)?;
            let path = trash.join("files").join(&id);
            std::fs::symlink_metadata(&path).ok()?;
            Some(Trashed {
                id,
                path,
                original,
                deleted,
            })
        })
        .collect();
    v.sort_by(|a, b| b.deleted.cmp(&a.deleted));
    v
}

pub fn list() -> Vec<Trashed> {
    trash_dir().map(|t| list_in(&t)).unwrap_or_default()
}

/// Restaura para o local original. Se já existir algo lá, usa um nome único
/// (nunca sobrescreve); se a pasta original não existir mais, ela é recriada.
pub fn restore_in(trash: &Path, item: &Trashed) -> Result<PathBuf, String> {
    // O id vem do nome do arquivo .trashinfo: não pode escapar de Trash/files.
    if item.id.contains('/') || item.id == ".." || item.id == "." {
        return Err("entrada da lixeira inválida".into());
    }
    let src = trash.join("files").join(&item.id);
    let parent = item.original.parent().ok_or("caminho original inválido")?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("não foi possível recriar {}: {e}", parent.display()))?;
    let dst = crate::fsops::unique_dest(
        parent,
        item.original
            .file_name()
            .ok_or("caminho original inválido")?,
    );
    std::fs::rename(&src, &dst).map_err(|e| format!("{}: {e}", dst.display()))?;
    let _ = std::fs::remove_file(trash.join("info").join(format!("{}.trashinfo", item.id)));
    Ok(dst)
}

pub fn restore(item: &Trashed) -> Result<PathBuf, String> {
    restore_in(&trash_dir().ok_or("lixeira indisponível")?, item)
}

/// Exclusão definitiva de itens da lixeira (só após confirmação na UI).
pub fn purge_in(trash: &Path, items: &[Trashed]) -> Result<(), String> {
    for it in items {
        if it.id.contains('/') || it.id == ".." {
            return Err("entrada da lixeira inválida".into());
        }
        crate::fsops::remove_tree(&trash.join("files").join(&it.id))?;
        let _ = std::fs::remove_file(trash.join("info").join(format!("{}.trashinfo", it.id)));
    }
    Ok(())
}

pub fn purge(items: &[Trashed]) -> Result<(), String> {
    purge_in(&trash_dir().ok_or("lixeira indisponível")?, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fake_trash() -> PathBuf {
        let t = std::env::temp_dir().join(format!(
            "osmf-trash-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(t.join("files")).unwrap();
        fs::create_dir_all(t.join("info")).unwrap();
        t
    }

    #[test]
    fn parses_trashinfo_with_percent_encoding() {
        let (p, d) = parse_trashinfo("[Trash Info]\nPath=/tmp/Relat%C3%B3rio%20final.txt\nDeletionDate=2026-11-13T10:00:00\n").unwrap();
        assert_eq!(p, PathBuf::from("/tmp/Relatório final.txt"));
        assert_eq!(d, "2026-11-13 10:00:00");
        assert!(parse_trashinfo("[Trash Info]\nPath=relativo\n").is_none());
        assert!(parse_trashinfo("lixo").is_none());
    }

    #[test]
    fn list_restore_and_purge() {
        let t = fake_trash();
        let home = t.join("casa");
        fs::write(t.join("files/doc.txt"), "dados").unwrap();
        fs::write(
            t.join("info/doc.txt.trashinfo"),
            format!(
                "[Trash Info]\nPath={}/docs/doc.txt\nDeletionDate=2026-11-13T09:00:00\n",
                home.display()
            ),
        )
        .unwrap();
        fs::write(t.join("files/velho"), "x").unwrap();
        fs::write(
            t.join("info/velho.trashinfo"),
            format!(
                "[Trash Info]\nPath={}/velho\nDeletionDate=2026-01-01T00:00:00\n",
                home.display()
            ),
        )
        .unwrap();
        let items = list_in(&t);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "doc.txt", "mais recente primeiro");
        // Restaura recriando a pasta original.
        let back = restore_in(&t, &items[0]).unwrap();
        assert_eq!(back, home.join("docs/doc.txt"));
        assert_eq!(fs::read_to_string(&back).unwrap(), "dados");
        assert!(!t.join("info/doc.txt.trashinfo").exists());
        // Restaurar de novo um item com o mesmo nome não sobrescreve.
        fs::write(home.join("velho"), "novo").unwrap();
        let back2 = restore_in(&t, &list_in(&t)[0]).unwrap();
        assert_eq!(back2, home.join("velho (2)"));
        assert_eq!(fs::read_to_string(home.join("velho")).unwrap(), "novo");
        // Esvaziar.
        fs::write(t.join("files/lixo"), "").unwrap();
        fs::write(
            t.join("info/lixo.trashinfo"),
            "[Trash Info]\nPath=/tmp/lixo\nDeletionDate=2026-11-13T11:00:00\n",
        )
        .unwrap();
        purge_in(&t, &list_in(&t)).unwrap();
        assert!(list_in(&t).is_empty());
        fs::remove_dir_all(t).unwrap();
    }

    #[test]
    fn rejects_traversal_ids() {
        let t = fake_trash();
        let evil = Trashed {
            id: "../../etc".into(),
            path: t.clone(),
            original: PathBuf::from("/tmp/x"),
            deleted: String::new(),
        };
        assert!(restore_in(&t, &evil).is_err());
        assert!(purge_in(&t, &[evil]).is_err());
        fs::remove_dir_all(t).unwrap();
    }
}
