//! OSMarcio Files — gerenciador de arquivos em Rust (GTK4 + GIO).
//!
//! `osmarcio-files [PASTA]` · também recebe pastas via associação MIME
//! (`inode/directory`). Não roda como root: o app só vê e altera o que as
//! permissões do usuário permitem.

mod dialogs;
mod fsops;
mod listing;
mod trash;
mod ui;

use gtk4::prelude::*;
use gtk4::{gio, glib};
use std::path::PathBuf;

const APP_ID: &str = "org.osmarcio.Files";

fn is_root() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .map(|l| l.split_whitespace().nth(2) == Some("0"))
        })
        .unwrap_or(false)
}

fn main() -> glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    if is_root() {
        eprintln!(
            "osmarcio-files: não execute o gerenciador de arquivos como root. Use sua conta de usuário; para tarefas administrativas, use o terminal com sudo."
        );
        return glib::ExitCode::FAILURE;
    }
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE | gio::ApplicationFlags::HANDLES_OPEN)
        .build();
    app.connect_activate(|app| {
        ui::Files::new(
            app,
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/")),
        );
    });
    app.connect_open(|app, files, _| {
        for f in files {
            let p = f.path().unwrap_or_else(|| PathBuf::from("/"));
            let dir = if p.is_dir() {
                p
            } else {
                p.parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("/"))
            };
            ui::Files::new(app, dir);
        }
    });
    app.run()
}
