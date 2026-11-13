//! Preferências do terminal, persistidas em `~/.config/osmarcio/terminal.toml`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    /// Fonte no formato Pango, ex.: "Monospace 11".
    pub font: String,
    /// Shell; vazio = `$SHELL` do usuário, ou /bin/bash.
    pub shell: String,
    /// Linhas de histórico rolável (scrollback).
    pub scrollback_lines: i64,
    /// "block" | "ibeam" | "underline"
    pub cursor_shape: String,
    pub cursor_blink: bool,
    /// Tema: "osmarcio-dark" | "osmarcio-light"
    pub theme: String,
    /// true  => Ctrl+T / Ctrl+W / Ctrl+Tab (padrão do OSMarcio);
    /// false => Ctrl+Shift+T / Ctrl+Shift+W, deixando Ctrl+W/Ctrl+T para o Bash.
    pub plain_ctrl_tab_shortcuts: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font: "Monospace 11".into(),
            shell: String::new(),
            scrollback_lines: 10_000,
            cursor_shape: "block".into(),
            cursor_blink: true,
            theme: "osmarcio-dark".into(),
            plain_ctrl_tab_shortcuts: true,
        }
    }
}

impl TerminalConfig {
    pub const APP: &'static str = "terminal";

    /// Shell a executar: config > $SHELL > /bin/bash.
    pub fn resolve_shell(&self, env_shell: Option<String>) -> String {
        if !self.shell.trim().is_empty() {
            return self.shell.trim().to_string();
        }
        env_shell
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "/bin/bash".into())
    }
}

/// Paleta de cores (fg, bg, 16 cores ANSI) em hex.
pub struct Palette {
    pub fg: &'static str,
    pub bg: &'static str,
    pub ansi: [&'static str; 16],
}

pub fn palette(theme: &str) -> Palette {
    match theme {
        "osmarcio-light" => Palette {
            fg: "#1e2430",
            bg: "#f7f8fa",
            ansi: [
                "#1e2430", "#c0392b", "#2e8b57", "#b7791f", "#2b6cb0", "#8e44ad", "#138d90",
                "#d5d9e0", "#5c6370", "#e74c3c", "#38a169", "#d69e2e", "#3182ce", "#9f7aea",
                "#0bc5ea", "#ffffff",
            ],
        },
        _ => Palette {
            fg: "#e6e9ef",
            bg: "#141821",
            ansi: [
                "#1d2230", "#ff6b6b", "#5fd38d", "#f6c560", "#5aa9ff", "#c792ea", "#4dd0e1",
                "#d0d6e0", "#5c6680", "#ff8787", "#7ee2a8", "#ffd88a", "#82c0ff", "#dab4ff",
                "#84e3ef", "#ffffff",
            ],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_resolution_order() {
        let mut c = TerminalConfig::default();
        assert_eq!(c.resolve_shell(None), "/bin/bash");
        assert_eq!(c.resolve_shell(Some("/usr/bin/zsh".into())), "/usr/bin/zsh");
        assert_eq!(c.resolve_shell(Some("  ".into())), "/bin/bash");
        c.shell = "/bin/sh".into();
        assert_eq!(c.resolve_shell(Some("/usr/bin/zsh".into())), "/bin/sh");
    }

    #[test]
    fn config_toml_roundtrip() {
        let c = TerminalConfig::default();
        let s = toml::to_string(&c).unwrap();
        assert_eq!(toml::from_str::<TerminalConfig>(&s).unwrap(), c);
    }

    #[test]
    fn palettes_are_complete_hex() {
        for t in ["osmarcio-dark", "osmarcio-light", "desconhecido"] {
            let p = palette(t);
            for c in p.ansi.iter().chain([&p.fg, &p.bg]) {
                assert!(c.len() == 7 && c.starts_with('#'), "{t}: {c}");
            }
        }
    }
}
