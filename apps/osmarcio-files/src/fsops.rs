//! Operações de arquivos do OSMarcio Files — sem interface gráfica (testáveis).
//!
//! Regras de segurança aplicadas aqui (não na UI):
//! * nunca sobrescreve: destino existente ganha nome único "nome (2).ext";
//! * recursão NÃO segue links simbólicos (o link é copiado como link);
//! * mover entre sistemas de arquivos = copiar tudo e só então apagar a origem;
//! * cancelamento cooperativo (`gio::Cancellable`): o arquivo parcial é removido;
//! * diretórios críticos (/, /usr, /etc, a própria home…) não vão para a
//!   lixeira nem são excluídos — e as permissões do Linux continuam valendo:
//!   o app roda como usuário comum e não contorna nada.

use gtk4::gio;
use gtk4::gio::prelude::*;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Progresso de uma operação longa.
#[derive(Debug, Clone, Default)]
pub struct Progress {
    pub done: u64,
    pub total: u64,
    pub current: String,
}

/// Mensagem de erro legível a partir de um erro do GIO/IO.
pub fn describe(e: &gtk4::glib::Error) -> String {
    use gio::IOErrorEnum as E;
    match e.kind::<E>() {
        Some(E::PermissionDenied) => "permissão negada".into(),
        Some(E::NoSpace) => "sem espaço no destino".into(),
        Some(E::NotFound) => "arquivo ou pasta não encontrado (volume removido?)".into(),
        Some(E::Exists) => "o destino já existe".into(),
        Some(E::Cancelled) => "operação cancelada".into(),
        Some(E::ReadOnly) => "sistema de arquivos somente leitura".into(),
        Some(E::NotEmpty) => "a pasta não está vazia".into(),
        _ => e.message().to_string(),
    }
}

fn io(e: std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => "permissão negada".into(),
        std::io::ErrorKind::NotFound => "arquivo ou pasta não encontrado".into(),
        std::io::ErrorKind::AlreadyExists => "o destino já existe".into(),
        _ => e.to_string(),
    }
}

/// Valida um nome de arquivo digitado pelo usuário (sem barras, não vazio, não "."/"..").
pub fn validate_name(name: &str) -> Result<&str, String> {
    let n = name.trim();
    if n.is_empty() {
        Err("o nome não pode ser vazio".into())
    } else if n == "." || n == ".." {
        Err("nome reservado".into())
    } else if n.contains('/') || n.contains('\0') {
        Err("o nome não pode conter \"/\"".into())
    } else if n.len() > 255 {
        Err("nome longo demais (máx. 255 bytes)".into())
    } else {
        Ok(n)
    }
}

/// Destino livre em `dir` para `name`: "x.txt" → "x (2).txt" → "x (3).txt"…
pub fn unique_dest(dir: &Path, name: &OsStr) -> PathBuf {
    let first = dir.join(name);
    if std::fs::symlink_metadata(&first).is_err() {
        return first;
    }
    let p = Path::new(name);
    let (stem, ext) = match (p.file_stem(), p.extension()) {
        // ".bashrc" não tem extensão; "a.tar.gz" vira "a.tar (2).gz" (simples e previsível).
        (Some(s), Some(e)) if !s.is_empty() => (
            s.to_string_lossy().into_owned(),
            format!(".{}", e.to_string_lossy()),
        ),
        _ => (name.to_string_lossy().into_owned(), String::new()),
    };
    (2..)
        .map(|n| dir.join(format!("{stem} ({n}){ext}")))
        .find(|c| std::fs::symlink_metadata(c).is_err())
        .expect("sempre há um nome livre")
}

/// Diretórios que o app recusa enviar à lixeira/excluir/mover, mesmo se o
/// usuário tivesse permissão (proteção contra acidentes).
pub fn is_protected(path: &Path) -> bool {
    const SYSTEM: &[&str] = &[
        "/", "/bin", "/boot", "/dev", "/etc", "/home", "/lib", "/lib64", "/media", "/mnt", "/opt",
        "/proc", "/root", "/run", "/sbin", "/srv", "/sys", "/tmp", "/usr", "/var",
    ];
    let p = path.components().collect::<PathBuf>();
    if SYSTEM.iter().any(|s| p == Path::new(s)) {
        return true;
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        if p == home {
            return true;
        }
        // Pastas XDG da própria home (Documentos, Downloads…).
        if let Some(name) = p
            .strip_prefix(&home)
            .ok()
            .filter(|r| r.components().count() == 1)
            && [".config", ".local", ".cache"]
                .iter()
                .any(|n| name == Path::new(n))
        {
            return true;
        }
    }
    false
}

/// Tamanho total (bytes) sem seguir links simbólicos.
pub fn total_size(paths: &[PathBuf], cancel: &gio::Cancellable) -> u64 {
    let mut total = 0;
    let mut stack: Vec<PathBuf> = paths.to_vec();
    while let Some(p) = stack.pop() {
        if cancel.is_cancelled() {
            break;
        }
        let Ok(md) = std::fs::symlink_metadata(&p) else {
            continue;
        };
        if md.is_dir() {
            if let Ok(rd) = std::fs::read_dir(&p) {
                stack.extend(rd.flatten().map(|e| e.path()));
            }
        } else if md.is_file() {
            // Só arquivos comuns contam bytes (links são recriados, não copiados).
            total += md.len();
        }
    }
    total
}

/// Copia `src` para `dst` (que ainda não existe), recursivamente.
fn copy_one(
    src: &Path,
    dst: &Path,
    cancel: &gio::Cancellable,
    pr: &mut Progress,
    cb: &mut dyn FnMut(&Progress),
) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err("operação cancelada".into());
    }
    let md = std::fs::symlink_metadata(src).map_err(io)?;
    let ft = md.file_type();
    if ft.is_symlink() {
        let target = std::fs::read_link(src).map_err(io)?;
        std::os::unix::fs::symlink(target, dst).map_err(io)?;
    } else if ft.is_dir() {
        std::fs::create_dir(dst).map_err(io)?;
        let mut entries: Vec<_> = std::fs::read_dir(src)
            .map_err(io)?
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for e in entries {
            copy_one(
                &e,
                &dst.join(e.file_name().unwrap_or_default()),
                cancel,
                pr,
                cb,
            )?;
        }
        // Permissões da pasta depois do conteúdo (pasta somente leitura não bloqueia a cópia).
        let _ = std::fs::set_permissions(dst, md.permissions());
    } else if ft.is_file() {
        pr.current = src
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let base = pr.done;
        let (s, d) = (gio::File::for_path(src), gio::File::for_path(dst));
        let mut on_bytes = |cur: i64, _total: i64| {
            pr.done = base + cur.max(0) as u64;
            cb(pr);
        };
        if let Err(e) = s.copy(
            &d,
            gio::FileCopyFlags::NOFOLLOW_SYMLINKS,
            Some(cancel),
            Some(&mut on_bytes),
        ) {
            // Não deixa arquivo parcial para trás.
            let _ = std::fs::remove_file(dst);
            return Err(format!("{}: {}", src.display(), describe(&e)));
        }
        pr.done = base + md.len();
        cb(pr);
    } else {
        // FIFOs, sockets e dispositivos não são copiados.
        return Err(format!(
            "{}: tipo de arquivo especial não suportado",
            src.display()
        ));
    }
    Ok(())
}

/// Copia vários itens para `dest_dir`. Retorna os caminhos criados.
/// Em caso de erro/cancelamento, os itens já copiados por completo permanecem
/// e o item em andamento é desfeito.
pub fn copy_items(
    srcs: &[PathBuf],
    dest_dir: &Path,
    cancel: &gio::Cancellable,
    cb: &mut dyn FnMut(&Progress),
) -> Result<Vec<PathBuf>, String> {
    let mut pr = Progress {
        total: total_size(srcs, cancel),
        ..Default::default()
    };
    let mut made = Vec::new();
    for src in srcs {
        if dest_dir.starts_with(src) {
            return Err(format!(
                "não é possível copiar \"{}\" para dentro dela mesma",
                src.display()
            ));
        }
        let dst = unique_dest(dest_dir, src.file_name().unwrap_or_default());
        if let Err(e) = copy_one(src, &dst, cancel, &mut pr, cb) {
            let _ = remove_tree(&dst);
            return Err(e);
        }
        made.push(dst);
    }
    Ok(made)
}

/// Move vários itens. No mesmo sistema de arquivos é um `rename` atômico; entre
/// sistemas de arquivos copia tudo e só apaga a origem depois da cópia completa.
pub fn move_items(
    srcs: &[PathBuf],
    dest_dir: &Path,
    cancel: &gio::Cancellable,
    cb: &mut dyn FnMut(&Progress),
) -> Result<Vec<PathBuf>, String> {
    let mut made = Vec::new();
    for src in srcs {
        if is_protected(src) {
            return Err(format!(
                "\"{}\" é protegido e não pode ser movido",
                src.display()
            ));
        }
        if dest_dir.starts_with(src) {
            return Err(format!(
                "não é possível mover \"{}\" para dentro dela mesma",
                src.display()
            ));
        }
        if src.parent() == Some(dest_dir) {
            continue; // já está lá
        }
        let dst = unique_dest(dest_dir, src.file_name().unwrap_or_default());
        match std::fs::rename(src, &dst) {
            Ok(()) => made.push(dst),
            Err(e) if e.raw_os_error() == Some(18) => {
                // EXDEV: outro sistema de arquivos.
                let copied = copy_items(std::slice::from_ref(src), dest_dir, cancel, cb)?;
                remove_tree(src).map_err(|e| {
                    format!("cópia concluída, mas a origem não pôde ser removida: {e}")
                })?;
                made.extend(copied);
            }
            Err(e) => return Err(format!("{}: {}", src.display(), io(e))),
        }
    }
    Ok(made)
}

/// Remove definitivamente (sem seguir links). Usado só após confirmação explícita.
pub fn remove_tree(path: &Path) -> Result<(), String> {
    let md = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io(e)),
    };
    if md.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
    .map_err(io)
}

pub fn delete_permanently(paths: &[PathBuf]) -> Result<(), String> {
    for p in paths {
        if is_protected(p) {
            return Err(format!(
                "\"{}\" é protegido e não pode ser excluído",
                p.display()
            ));
        }
        remove_tree(p).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    Ok(())
}

/// Envia à lixeira do usuário (FreeDesktop Trash, implementada pelo GIO).
pub fn trash(paths: &[PathBuf]) -> Result<(), String> {
    for p in paths {
        if is_protected(p) {
            return Err(format!(
                "\"{}\" é protegido e não vai para a lixeira",
                p.display()
            ));
        }
        gio::File::for_path(p)
            .trash(None::<&gio::Cancellable>)
            .map_err(|e| format!("{}: {}", p.display(), describe(&e)))?;
    }
    Ok(())
}

pub fn create_folder(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let n = validate_name(name)?;
    let p = dir.join(n);
    std::fs::create_dir(&p).map_err(io)?;
    Ok(p)
}

/// Cria um arquivo vazio; falha se já existir (nunca trunca um arquivo).
pub fn create_file(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let n = validate_name(name)?;
    let p = dir.join(n);
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&p)
        .map_err(io)?;
    Ok(p)
}

/// Renomeia dentro da mesma pasta; recusa se o novo nome já existir.
pub fn rename(path: &Path, new_name: &str) -> Result<PathBuf, String> {
    let n = validate_name(new_name)?;
    if is_protected(path) {
        return Err("item protegido".into());
    }
    let dst = path.parent().unwrap_or(Path::new("/")).join(n);
    if dst == path {
        return Ok(dst);
    }
    if std::fs::symlink_metadata(&dst).is_ok() {
        return Err(format!("já existe \"{n}\" nesta pasta"));
    }
    std::fs::rename(path, &dst).map_err(io)?;
    Ok(dst)
}

/// Busca por nome (sem diferenciar maiúsculas), sem seguir links simbólicos.
/// `recursive = false` olha só a pasta atual. Limite de resultados para não travar.
pub fn search(
    dir: &Path,
    query: &str,
    recursive: bool,
    show_hidden: bool,
    cancel: &gio::Cancellable,
    limit: usize,
) -> Vec<PathBuf> {
    let q = query.to_lowercase();
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        if cancel.is_cancelled() || out.len() >= limit {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            if name.to_lowercase().contains(&q) {
                out.push(e.path());
            }
            if recursive && e.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(e.path());
            }
        }
    }
    out.sort();
    out.truncate(limit);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp() -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "osmf-test-{}-{}",
            std::process::id(),
            rand_suffix()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }
    fn rand_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
    fn none() -> gio::Cancellable {
        gio::Cancellable::new()
    }

    #[test]
    fn names_are_validated() {
        assert!(validate_name("").is_err());
        assert!(validate_name("..").is_err());
        assert!(validate_name("a/b").is_err());
        assert_eq!(
            validate_name("  Relatório 2026.txt ").unwrap(),
            "Relatório 2026.txt"
        );
        assert_eq!(validate_name("日本語 ✓").unwrap(), "日本語 ✓");
    }

    #[test]
    fn unique_names_never_overwrite() {
        let d = tmp();
        assert_eq!(unique_dest(&d, OsStr::new("a.txt")), d.join("a.txt"));
        fs::write(d.join("a.txt"), "1").unwrap();
        assert_eq!(unique_dest(&d, OsStr::new("a.txt")), d.join("a (2).txt"));
        fs::write(d.join("a (2).txt"), "2").unwrap();
        assert_eq!(unique_dest(&d, OsStr::new("a.txt")), d.join("a (3).txt"));
        fs::write(d.join(".bashrc"), "").unwrap();
        assert_eq!(
            unique_dest(&d, OsStr::new(".bashrc")),
            d.join(".bashrc (2)")
        );
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn critical_paths_are_protected() {
        for p in ["/", "/usr", "/etc/", "/home", "/root"] {
            assert!(is_protected(Path::new(p)), "{p}");
        }
        assert!(!is_protected(Path::new("/tmp/qualquer")));
        if let Some(h) = std::env::var_os("HOME") {
            assert!(is_protected(Path::new(&h)));
            assert!(is_protected(&Path::new(&h).join(".config")));
            assert!(!is_protected(&Path::new(&h).join("Documentos/x")));
        }
        assert!(trash(&[PathBuf::from("/")]).is_err());
        assert!(delete_permanently(&[PathBuf::from("/usr")]).is_err());
    }

    #[test]
    fn copy_file_and_tree_without_following_symlinks() {
        let d = tmp();
        let src = d.join("origem ✓");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("a.txt"), "olá").unwrap();
        fs::write(src.join("sub/b.bin"), vec![7u8; 100_000]).unwrap();
        std::os::unix::fs::symlink("/etc/hostname", src.join("link")).unwrap();
        let dest = d.join("destino");
        fs::create_dir(&dest).unwrap();
        let mut last = Progress::default();
        let made = copy_items(std::slice::from_ref(&src), &dest, &none(), &mut |p| {
            last = p.clone()
        })
        .unwrap();
        let c = &made[0];
        assert_eq!(fs::read_to_string(c.join("a.txt")).unwrap(), "olá");
        assert_eq!(fs::read(c.join("sub/b.bin")).unwrap().len(), 100_000);
        assert!(
            fs::symlink_metadata(c.join("link"))
                .unwrap()
                .file_type()
                .is_symlink(),
            "link copiado como link"
        );
        assert_eq!(last.done, last.total);
        // Segunda cópia: nome único, original intacto.
        let made2 = copy_items(std::slice::from_ref(&src), &dest, &none(), &mut |_| {}).unwrap();
        assert_eq!(made2[0], dest.join("origem ✓ (2)"));
        assert!(src.join("a.txt").exists());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn copy_into_itself_is_refused() {
        let d = tmp();
        fs::create_dir(d.join("p")).unwrap();
        assert!(copy_items(&[d.join("p")], &d.join("p"), &none(), &mut |_| {}).is_err());
        assert!(move_items(&[d.join("p")], &d.join("p"), &none(), &mut |_| {}).is_err());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn cancelled_copy_leaves_no_partial_item() {
        let d = tmp();
        fs::write(d.join("grande"), vec![1u8; 4 << 20]).unwrap();
        fs::create_dir(d.join("dst")).unwrap();
        let c = gio::Cancellable::new();
        c.cancel();
        assert!(copy_items(&[d.join("grande")], &d.join("dst"), &c, &mut |_| {}).is_err());
        assert_eq!(fs::read_dir(d.join("dst")).unwrap().count(), 0);
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn move_rename_create() {
        let d = tmp();
        fs::write(d.join("x.txt"), "x").unwrap();
        fs::create_dir(d.join("pasta")).unwrap();
        let moved = move_items(&[d.join("x.txt")], &d.join("pasta"), &none(), &mut |_| {}).unwrap();
        assert_eq!(moved[0], d.join("pasta/x.txt"));
        assert!(!d.join("x.txt").exists());
        let r = rename(&moved[0], "y.txt").unwrap();
        assert_eq!(fs::read_to_string(&r).unwrap(), "x");
        fs::write(d.join("pasta/z.txt"), "z").unwrap();
        assert!(rename(&r, "z.txt").is_err(), "não sobrescreve");
        assert!(create_folder(&d, "nova").is_ok());
        assert!(create_folder(&d, "nova").is_err());
        assert!(create_file(&d, "vazio.txt").is_ok());
        fs::write(d.join("vazio.txt"), "conteúdo").unwrap();
        assert!(create_file(&d, "vazio.txt").is_err(), "nunca trunca");
        assert_eq!(fs::read_to_string(d.join("vazio.txt")).unwrap(), "conteúdo");
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn search_by_name_with_and_without_recursion() {
        let d = tmp();
        fs::create_dir_all(d.join("a/b")).unwrap();
        fs::write(d.join("Relatorio.txt"), "").unwrap();
        fs::write(d.join("a/b/relatório-final.txt"), "").unwrap();
        fs::write(d.join(".relatorio-oculto"), "").unwrap();
        std::os::unix::fs::symlink(&d, d.join("a/loop")).unwrap();
        let c = none();
        assert_eq!(search(&d, "relat", false, false, &c, 100).len(), 1);
        assert_eq!(
            search(&d, "relat", true, false, &c, 100).len(),
            2,
            "não entra no link em loop"
        );
        assert_eq!(search(&d, "relat", true, true, &c, 100).len(), 3);
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn delete_does_not_follow_symlinks() {
        let d = tmp();
        fs::create_dir(d.join("alvo")).unwrap();
        fs::write(d.join("alvo/importante"), "!").unwrap();
        std::os::unix::fs::symlink(d.join("alvo"), d.join("atalho")).unwrap();
        delete_permanently(&[d.join("atalho")]).unwrap();
        assert!(
            d.join("alvo/importante").exists(),
            "apagar o link não apaga o alvo"
        );
        fs::remove_dir_all(d).unwrap();
    }
}
