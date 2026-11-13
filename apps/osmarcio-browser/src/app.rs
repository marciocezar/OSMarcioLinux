//! `App` do CEF e configurações globais (perfil, cache, idioma, sandbox).

use cef::*;
use std::path::PathBuf;

wrap_app! {
    pub struct OsmApp;

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(OsmBrowserProcessHandler::new())
        }
    }
}

wrap_browser_process_handler! {
    struct OsmBrowserProcessHandler;

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            // Contexto do CEF pronto (thread de UI): cria a janela.
            crate::ui::create_main_window();
        }
    }
}

fn xdg(base: Option<PathBuf>, rel: &str) -> PathBuf {
    base.unwrap_or_else(|| PathBuf::from("/tmp")).join(rel)
}

/// Perfil persistente (cookies, localStorage, cache HTTP) em
/// `~/.local/share/osmarcio/browser/profile`; log em `~/.cache/osmarcio/browser/cef.log`.
pub fn settings() -> Settings {
    let dirs = directories_base();
    let profile = xdg(dirs.0, "osmarcio/browser/profile");
    let cache = xdg(dirs.1, "osmarcio/browser");
    let _ = std::fs::create_dir_all(&profile);
    let _ = std::fs::create_dir_all(&cache);
    let s = |p: PathBuf| CefString::from(p.to_string_lossy().as_ref());
    Settings {
        // Sandbox do Chromium LIGADO (namespaces do kernel + seccomp-bpf).
        no_sandbox: 0,
        root_cache_path: s(profile.clone()),
        cache_path: s(profile),
        persist_session_cookies: 1,
        locale: CefString::from("pt-BR"),
        log_file: s(cache.join("cef.log")),
        log_severity: LogSeverity::WARNING,
        ..Default::default()
    }
}

/// (data_dir, cache_dir) respeitando XDG_DATA_HOME/XDG_CACHE_HOME.
fn directories_base() -> (Option<PathBuf>, Option<PathBuf>) {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|h| h.join(".local/share")));
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|h| h.join(".cache")));
    (data, cache)
}
