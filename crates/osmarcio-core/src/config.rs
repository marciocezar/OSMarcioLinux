//! Configuração persistente em padrão XDG:
//! `$XDG_CONFIG_HOME/osmarcio/<app>.toml` (normalmente `~/.config/osmarcio/`).
//!
//! Gravação segura: escreve num arquivo temporário no mesmo diretório e faz
//! `rename` atômico, para nunca deixar um arquivo de configuração pela metade.

use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("não foi possível determinar o diretório de configuração (HOME ausente?)")]
    NoConfigDir,
    #[error("erro de E/S em {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("configuração inválida em {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("falha ao serializar configuração: {0}")]
    Serialize(#[from] toml::ser::Error),
}

/// Diretório `~/.config/osmarcio` (respeita `XDG_CONFIG_HOME`).
pub fn config_dir() -> Result<PathBuf, ConfigError> {
    directories::BaseDirs::new()
        .map(|b| b.config_dir().join("osmarcio"))
        .ok_or(ConfigError::NoConfigDir)
}

/// Caminho do arquivo de configuração de um app.
pub fn config_path(app: &str) -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join(format!("{app}.toml")))
}

/// Carrega a configuração; se o arquivo não existir, retorna `T::default()`.
pub fn load_from<T: DeserializeOwned + Default>(path: &Path) -> Result<T, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(s) => toml::from_str(&s).map_err(|source| ConfigError::Parse {
            path: path.into(),
            source,
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(source) => Err(ConfigError::Io {
            path: path.into(),
            source,
        }),
    }
}

/// Grava a configuração de forma atômica (tmp + rename).
pub fn save_to<T: Serialize>(path: &Path, value: &T) -> Result<(), ConfigError> {
    let io = |source| ConfigError::Io {
        path: path.into(),
        source,
    };
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(io)?;
    let text = toml::to_string_pretty(value)?;
    let tmp = dir.join(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("config"),
        std::process::id()
    ));
    std::fs::write(&tmp, text).map_err(io)?;
    std::fs::rename(&tmp, path).map_err(io)
}

/// Carrega a configuração de `app` no local XDG padrão.
/// Em caso de arquivo corrompido, registra um aviso e usa os padrões —
/// um app de desktop não deve deixar de abrir por causa disso.
pub fn load_or_default<T: DeserializeOwned + Default>(app: &str) -> T {
    match config_path(app).and_then(|p| load_from(&p)) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("{e}; usando configuração padrão");
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Demo {
        font: String,
        size: u32,
    }

    #[test]
    fn missing_file_gives_default() {
        let d = tempfile::tempdir().unwrap();
        let c: Demo = load_from(&d.path().join("nao-existe.toml")).unwrap();
        assert_eq!(c, Demo::default());
    }

    #[test]
    fn roundtrip_creates_dirs_and_leaves_no_tmp() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a/b/demo.toml");
        let v = Demo {
            font: "Monospace".into(),
            size: 12,
        };
        save_to(&p, &v).unwrap();
        assert_eq!(load_from::<Demo>(&p).unwrap(), v);
        let entries: Vec<_> = std::fs::read_dir(p.parent().unwrap()).unwrap().collect();
        assert_eq!(entries.len(), 1, "arquivo temporário não foi removido");
    }

    #[test]
    fn partial_file_fills_defaults() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("demo.toml");
        std::fs::write(&p, "size = 9\n").unwrap();
        assert_eq!(
            load_from::<Demo>(&p).unwrap(),
            Demo {
                font: String::new(),
                size: 9
            }
        );
    }

    #[test]
    fn invalid_file_is_an_error() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("demo.toml");
        std::fs::write(&p, "size = \"muitos\"\n").unwrap();
        assert!(matches!(
            load_from::<Demo>(&p),
            Err(ConfigError::Parse { .. })
        ));
    }
}
