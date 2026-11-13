//! `osmarcio-core` — lógica compartilhada pelos aplicativos do OSMarcio Linux.
//!
//! Este crate **não depende de GTK**: tudo aqui é lógica pura e pode ser
//! testado em qualquer host (`cargo test -p osmarcio-core`, inclusive no macOS).
//! A interface gráfica de cada app apenas consulta estas regras.

pub mod config;
pub mod sysinfo;
pub mod tabs;
pub mod web;

/// Versão da distribuição/aplicativos (vem do Cargo.toml do workspace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
