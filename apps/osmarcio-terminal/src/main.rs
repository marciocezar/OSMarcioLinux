//! OSMarcio Terminal — terminal gráfico em Rust (GTK4 + VTE).
//!
//! A emulação de terminal (sequências ANSI/VT, PTY, scrollback) é feita pela
//! biblioteca VTE — a mesma do GNOME Terminal. Este app cuida de janelas,
//! abas, atalhos, preferências e integração com o desktop.

mod config;
mod window;

use gtk4::prelude::*;
use gtk4::{gio, glib};

pub const APP_ID: &str = "org.osmarcio.Terminal";

fn main() -> glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("OSMarcio Terminal {}", osmarcio_core::VERSION);

    // NON_UNIQUE: cada invocação abre um processo/janela independente,
    // o que simplifica o modelo (um processo = uma janela com N abas).
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(|app| {
        let cfg: config::TerminalConfig =
            osmarcio_core::config::load_or_default(config::TerminalConfig::APP);
        window::TermWindow::new(app, cfg).present();
    });
    app.run()
}
