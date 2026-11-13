//! OSMarcio Minesweeper (Campo Minado): regras (`game`) e preferências
//! (`prefs`) ficam na biblioteca, separadas da interface GTK (`main.rs`).
//! `cargo test -p osmarcio-minesweeper --lib` roda sem sessão gráfica.
pub mod game;
pub mod prefs;
