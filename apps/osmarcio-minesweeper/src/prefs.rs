//! Preferências e recordes em `~/.config/osmarcio/minesweeper.toml` (XDG),
//! gravados de forma atômica por `osmarcio_core::config`.

use crate::game::{Config, Difficulty};
use serde::{Deserialize, Serialize};

pub const APP: &str = "minesweeper";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Prefs {
    pub rows: usize,
    pub cols: usize,
    pub mines: usize,
    /// Melhor tempo (segundos) por dificuldade padrão.
    pub best_easy: Option<u64>,
    pub best_intermediate: Option<u64>,
    pub best_expert: Option<u64>,
}

impl Default for Prefs {
    fn default() -> Self {
        let c = Config::EASY;
        Prefs {
            rows: c.rows,
            cols: c.cols,
            mines: c.mines,
            best_easy: None,
            best_intermediate: None,
            best_expert: None,
        }
    }
}

impl Prefs {
    /// Configuração salva; se o arquivo foi editado com valores inválidos, volta ao Fácil.
    pub fn config(&self) -> Config {
        [Config::EASY, Config::INTERMEDIATE, Config::EXPERT]
            .into_iter()
            .find(|c| (c.rows, c.cols, c.mines) == (self.rows, self.cols, self.mines))
            .or_else(|| Config::custom(self.rows, self.cols, self.mines).ok())
            .unwrap_or(Config::EASY)
    }

    pub fn set_config(&mut self, c: Config) {
        (self.rows, self.cols, self.mines) = (c.rows, c.cols, c.mines);
    }

    fn slot(&mut self, d: Difficulty) -> Option<&mut Option<u64>> {
        match d {
            Difficulty::Easy => Some(&mut self.best_easy),
            Difficulty::Intermediate => Some(&mut self.best_intermediate),
            Difficulty::Expert => Some(&mut self.best_expert),
            Difficulty::Custom => None,
        }
    }

    /// Registra um tempo de vitória; retorna true se for novo recorde.
    /// Partidas personalizadas não têm recorde.
    pub fn record(&mut self, d: Difficulty, secs: u64) -> bool {
        match self.slot(d) {
            Some(best) if best.is_none_or(|b| secs < b) => {
                *best = Some(secs);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_only_improve_and_ignore_custom() {
        let mut p = Prefs::default();
        assert!(p.record(Difficulty::Easy, 50));
        assert!(!p.record(Difficulty::Easy, 60));
        assert!(p.record(Difficulty::Easy, 40));
        assert_eq!(p.best_easy, Some(40));
        assert!(!p.record(Difficulty::Custom, 1));
    }

    #[test]
    fn invalid_saved_config_falls_back_to_easy() {
        let mut p = Prefs::default();
        p.set_config(Config::EXPERT);
        assert_eq!(p.config(), Config::EXPERT);
        (p.rows, p.cols, p.mines) = (3, 3, 50);
        assert_eq!(p.config(), Config::EASY);
    }

    #[test]
    fn prefs_roundtrip_toml() {
        let p = Prefs {
            best_expert: Some(321),
            ..Default::default()
        };
        let t = toml::to_string(&p).unwrap();
        assert_eq!(toml::from_str::<Prefs>(&t).unwrap(), p);
        assert_eq!(toml::from_str::<Prefs>("").unwrap(), Prefs::default());
    }
}
