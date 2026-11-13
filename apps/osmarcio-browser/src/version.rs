//! Versões reais do OSMarcio Browser, num só lugar (tela Sobre, `--version`,
//! logs). Nada é fixo no código: cada campo vem de uma fonte verificável.
//!
//! | Campo            | Fonte                                                   |
//! |------------------|---------------------------------------------------------|
//! | Browser          | `Cargo.toml` do workspace (`CARGO_PKG_VERSION`)         |
//! | Chromium / CEF   | `cef_version_info()` do **libcef.so carregado** (runtime) |
//! | CEF compilado    | constantes do `cef-dll-sys` (versão contra a qual o binário foi gerado) |
//! | cef-rs           | `Cargo.lock` (build.rs)                                  |
//! | Arquitetura      | `std::env::consts::ARCH`                                 |
//! | Sistema          | `/etc/os-release`                                        |

use cef::sys;
use std::os::raw::c_int;

// `cef_version_info` é exportado pelo libcef.so mas não tem binding no cef-rs
// 156.4.0. Assinatura de include/cef_version_info.h: int cef_version_info(int entry).
#[allow(unsafe_code)]
mod ffi {
    use std::os::raw::c_int;
    unsafe extern "C" {
        pub safe fn cef_version_info(entry: c_int) -> c_int;
    }
}

fn runtime(entry: c_int) -> c_int {
    ffi::cef_version_info(entry)
}

pub struct VersionInfo {
    pub browser: String,
    pub chromium: String,
    pub cef: String,
    pub cef_compiled: String,
    pub cef_rs: String,
    pub arch: String,
    pub os: String,
    pub os_version: String,
}

impl VersionInfo {
    pub fn collect() -> Self {
        // Entradas: 0..2 = CEF major/minor/patch, 3 = commit, 4..7 = Chrome major/minor/build/patch.
        let cef = format!("{}.{}.{}", runtime(0), runtime(1), runtime(2));
        let chromium = format!(
            "{}.{}.{}.{}",
            runtime(4),
            runtime(5),
            runtime(6),
            runtime(7)
        );
        let compiled = String::from_utf8_lossy(sys::CEF_VERSION)
            .trim_end_matches('\0')
            .to_string();
        let osr = std::fs::read_to_string("/etc/os-release")
            .map(|t| osmarcio_core::sysinfo::parse_os_release(&t))
            .unwrap_or_default();
        let na = || "Não disponível".to_string();
        Self {
            browser: env!("CARGO_PKG_VERSION").to_string(),
            chromium,
            cef,
            cef_compiled: compiled,
            cef_rs: env!("OSMB_CEF_RS_VERSION").to_string(),
            arch: std::env::consts::ARCH.to_string(),
            os: osr.get("NAME").cloned().unwrap_or_else(na),
            os_version: osr.get("VERSION_ID").cloned().unwrap_or_else(na),
        }
    }

    /// O libcef.so carregado é o mesmo contra o qual o binário foi compilado?
    pub fn runtime_matches_build(&self) -> bool {
        self.cef_compiled.starts_with(&format!("{}+", self.cef))
            && self
                .cef_compiled
                .ends_with(&format!("chromium-{}", self.chromium))
    }

    /// Arquitetura no padrão Debian (nome dos pacotes).
    pub fn deb_arch(&self) -> &str {
        match self.arch.as_str() {
            "aarch64" => "arm64",
            "x86_64" => "amd64",
            a => a,
        }
    }

    pub fn rows(&self) -> Vec<(&'static str, String)> {
        vec![
            ("Versão do Browser", self.browser.clone()),
            ("Chromium Engine", self.chromium.clone()),
            ("CEF Runtime", self.cef.clone()),
            (
                "CEF (compilado)",
                format!(
                    "{}{}",
                    self.cef_compiled,
                    if self.runtime_matches_build() {
                        ""
                    } else {
                        "  ⚠ difere do runtime"
                    }
                ),
            ),
            ("Bindings Rust (cef-rs)", self.cef_rs.clone()),
            ("Arquitetura", self.arch.clone()),
            ("Sistema", self.os.clone()),
            ("Versão do Sistema", self.os_version.clone()),
        ]
    }

    pub fn as_text(&self) -> String {
        let mut s = String::from("OSMarcio Browser\n");
        for (k, v) in self.rows() {
            s.push_str(&format!("{k:<24}{v}\n"));
        }
        s
    }
}
