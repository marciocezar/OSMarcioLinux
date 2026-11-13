//! Regras do Campo Minado — lógica pura, sem GTK (testável sem sessão gráfica).
//!
//! * As minas são sorteadas **no primeiro clique**, excluindo a célula
//!   clicada e suas vizinhas (área 3×3). Por isso o número máximo de minas é
//!   `linhas × colunas − 9`: sempre sobra espaço para a área segura, e nenhuma
//!   mina é "redistribuída" depois — a regra é a mesma para todas as partidas.
//! * Abertura de áreas vazias (flood fill) é **iterativa** (pilha), sem
//!   recursão: tabuleiros grandes não estouram a pilha.

use std::collections::VecDeque;

pub const MIN_SIDE: usize = 5;
pub const MAX_ROWS: usize = 24;
pub const MAX_COLS: usize = 40;
/// Área preservada no primeiro clique (a célula e as 8 vizinhas).
pub const SAFE_AREA: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Intermediate,
    Expert,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub rows: usize,
    pub cols: usize,
    pub mines: usize,
}

impl Config {
    pub const EASY: Config = Config {
        rows: 9,
        cols: 9,
        mines: 10,
    };
    pub const INTERMEDIATE: Config = Config {
        rows: 16,
        cols: 16,
        mines: 40,
    };
    /// 30 colunas × 16 linhas.
    pub const EXPERT: Config = Config {
        rows: 16,
        cols: 30,
        mines: 99,
    };

    pub fn preset(d: Difficulty) -> Option<Config> {
        match d {
            Difficulty::Easy => Some(Self::EASY),
            Difficulty::Intermediate => Some(Self::INTERMEDIATE),
            Difficulty::Expert => Some(Self::EXPERT),
            Difficulty::Custom => None,
        }
    }

    pub fn difficulty(&self) -> Difficulty {
        match *self {
            c if c == Self::EASY => Difficulty::Easy,
            c if c == Self::INTERMEDIATE => Difficulty::Intermediate,
            c if c == Self::EXPERT => Difficulty::Expert,
            _ => Difficulty::Custom,
        }
    }

    /// Valida uma configuração personalizada.
    pub fn custom(rows: usize, cols: usize, mines: usize) -> Result<Config, String> {
        if !(MIN_SIDE..=MAX_ROWS).contains(&rows) {
            return Err(format!("linhas devem estar entre {MIN_SIDE} e {MAX_ROWS}"));
        }
        if !(MIN_SIDE..=MAX_COLS).contains(&cols) {
            return Err(format!("colunas devem estar entre {MIN_SIDE} e {MAX_COLS}"));
        }
        let max = rows * cols - SAFE_AREA;
        if mines == 0 || mines > max {
            return Err(format!(
                "minas devem estar entre 1 e {max} (o primeiro clique precisa de uma área 3×3 livre)"
            ));
        }
        Ok(Config { rows, cols, mines })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    NotStarted,
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cell {
    pub mine: bool,
    pub revealed: bool,
    pub flagged: bool,
    /// Minas nas 8 vizinhas.
    pub adjacent: u8,
}

/// Gerador SplitMix64 — suficiente para um jogo, sem dependências externas.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }
    /// Semente do kernel (/dev/urandom), com o relógio como reserva.
    pub fn from_entropy() -> Self {
        use std::io::Read;
        let mut b = [0u8; 8];
        let seed = std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut b))
            .map(|_| u64::from_le_bytes(b))
            .unwrap_or_else(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(1)
            });
        Rng(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Inteiro uniforme em `0..n` (rejeição para evitar viés).
    pub fn below(&mut self, n: usize) -> usize {
        let n = n as u64;
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < zone {
                return (x % n) as usize;
            }
        }
    }
}

/// Resultado de uma ação do jogador (para a interface reagir).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    Revealed(usize),
    Exploded,
    Won,
}

#[derive(Debug, Clone)]
pub struct Game {
    cfg: Config,
    cells: Vec<Cell>,
    state: State,
    flags: usize,
    revealed: usize,
    exploded: Option<(usize, usize)>,
    rng: Rng,
}

impl Game {
    pub fn new(cfg: Config, rng: Rng) -> Self {
        Game {
            cfg,
            cells: vec![Cell::default(); cfg.rows * cfg.cols],
            state: State::NotStarted,
            flags: 0,
            revealed: 0,
            exploded: None,
            rng,
        }
    }

    /// Recomeça com a mesma configuração (minas sorteadas de novo no 1º clique).
    pub fn restart(&mut self) {
        let rng = self.rng.clone();
        *self = Game::new(self.cfg, rng);
    }

    pub fn config(&self) -> Config {
        self.cfg
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn rows(&self) -> usize {
        self.cfg.rows
    }
    pub fn cols(&self) -> usize {
        self.cfg.cols
    }
    pub fn exploded(&self) -> Option<(usize, usize)> {
        self.exploded
    }
    pub fn flags(&self) -> usize {
        self.flags
    }
    /// Contador do jogo: minas − bandeiras (pode ficar negativo).
    pub fn mines_left(&self) -> i64 {
        self.cfg.mines as i64 - self.flags as i64
    }
    pub fn is_over(&self) -> bool {
        matches!(self.state, State::Won | State::Lost)
    }

    fn in_bounds(&self, r: usize, c: usize) -> bool {
        r < self.cfg.rows && c < self.cfg.cols
    }
    fn idx(&self, r: usize, c: usize) -> usize {
        r * self.cfg.cols + c
    }
    pub fn cell(&self, r: usize, c: usize) -> Option<&Cell> {
        self.in_bounds(r, c).then(|| &self.cells[self.idx(r, c)])
    }

    /// Vizinhas válidas (até 8) de (r, c).
    pub fn neighbors(&self, r: usize, c: usize) -> Vec<(usize, usize)> {
        let mut v = Vec::with_capacity(8);
        for dr in [-1i64, 0, 1] {
            for dc in [-1i64, 0, 1] {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let (nr, nc) = (r as i64 + dr, c as i64 + dc);
                if nr >= 0 && nc >= 0 && self.in_bounds(nr as usize, nc as usize) {
                    v.push((nr as usize, nc as usize));
                }
            }
        }
        v
    }

    /// Sorteia as minas fora da área segura em volta de (r, c).
    fn place_mines(&mut self, r: usize, c: usize) {
        let mut safe = self.neighbors(r, c);
        safe.push((r, c));
        let safe: Vec<usize> = safe.into_iter().map(|(a, b)| self.idx(a, b)).collect();
        let mut pool: Vec<usize> = (0..self.cells.len())
            .filter(|i| !safe.contains(i))
            .collect();
        // Fisher–Yates parcial: as primeiras `mines` posições viram minas.
        for i in 0..self.cfg.mines {
            let j = i + self.rng.below(pool.len() - i);
            pool.swap(i, j);
            self.cells[pool[i]].mine = true;
        }
        for rr in 0..self.cfg.rows {
            for cc in 0..self.cfg.cols {
                let n = self
                    .neighbors(rr, cc)
                    .iter()
                    .filter(|&&(a, b)| self.cells[self.idx(a, b)].mine)
                    .count();
                let i = self.idx(rr, cc);
                self.cells[i].adjacent = n as u8;
            }
        }
    }

    /// Clique esquerdo: revela a célula (no 1º clique, sorteia as minas).
    pub fn reveal(&mut self, r: usize, c: usize) -> Outcome {
        if !self.in_bounds(r, c) || self.is_over() {
            return Outcome::Nothing;
        }
        if self.state == State::NotStarted {
            self.place_mines(r, c);
            self.state = State::Playing;
        }
        let i = self.idx(r, c);
        if self.cells[i].flagged || self.cells[i].revealed {
            return Outcome::Nothing;
        }
        if self.cells[i].mine {
            self.cells[i].revealed = true;
            self.exploded = Some((r, c));
            self.state = State::Lost;
            for cell in self.cells.iter_mut().filter(|c| c.mine) {
                cell.revealed = true;
            }
            return Outcome::Exploded;
        }
        // Flood fill iterativo: abre a região sem minas vizinhas e sua borda numerada.
        let mut opened = 0;
        let mut stack = VecDeque::from([(r, c)]);
        while let Some((cr, cc)) = stack.pop_back() {
            let j = self.idx(cr, cc);
            if self.cells[j].revealed || self.cells[j].flagged || self.cells[j].mine {
                continue;
            }
            self.cells[j].revealed = true;
            opened += 1;
            if self.cells[j].adjacent == 0 {
                stack.extend(self.neighbors(cr, cc));
            }
        }
        self.revealed += opened;
        if self.revealed == self.cells.len() - self.cfg.mines {
            self.state = State::Won;
            // Na vitória, todas as minas ficam marcadas.
            for cell in self.cells.iter_mut().filter(|c| c.mine) {
                cell.flagged = true;
            }
            self.flags = self.cfg.mines;
            return Outcome::Won;
        }
        Outcome::Revealed(opened)
    }

    /// Clique direito: marca/desmarca bandeira numa célula oculta.
    pub fn toggle_flag(&mut self, r: usize, c: usize) -> bool {
        if !self.in_bounds(r, c) || self.is_over() {
            return false;
        }
        let i = self.idx(r, c);
        if self.cells[i].revealed {
            return false;
        }
        self.cells[i].flagged = !self.cells[i].flagged;
        if self.cells[i].flagged {
            self.flags += 1;
        } else {
            self.flags -= 1;
        }
        true
    }

    /// Clique num número já revelado: se as bandeiras vizinhas somam o número,
    /// revela as demais vizinhas ("chord"). Uma bandeira errada causa derrota,
    /// como no jogo clássico.
    pub fn chord(&mut self, r: usize, c: usize) -> Outcome {
        let Some(cell) = self.cell(r, c).copied() else {
            return Outcome::Nothing;
        };
        if self.is_over() || !cell.revealed || cell.adjacent == 0 {
            return Outcome::Nothing;
        }
        let nb = self.neighbors(r, c);
        let flagged = nb
            .iter()
            .filter(|&&(a, b)| self.cells[self.idx(a, b)].flagged)
            .count();
        if flagged != cell.adjacent as usize {
            return Outcome::Nothing;
        }
        let mut total = 0;
        for (a, b) in nb {
            match self.reveal(a, b) {
                Outcome::Exploded => return Outcome::Exploded,
                Outcome::Won => return Outcome::Won,
                Outcome::Revealed(n) => total += n,
                Outcome::Nothing => {}
            }
        }
        if total > 0 {
            Outcome::Revealed(total)
        } else {
            Outcome::Nothing
        }
    }

    /// Só para testes: monta um tabuleiro com minas em posições conhecidas.
    #[cfg(test)]
    pub fn with_mines(rows: usize, cols: usize, mines: &[(usize, usize)]) -> Game {
        let mut g = Game::new(
            Config {
                rows,
                cols,
                mines: mines.len(),
            },
            Rng::new(1),
        );
        for &(r, c) in mines {
            let i = g.idx(r, c);
            g.cells[i].mine = true;
        }
        for rr in 0..rows {
            for cc in 0..cols {
                let n = g
                    .neighbors(rr, cc)
                    .iter()
                    .filter(|&&(a, b)| g.cells[g.idx(a, b)].mine)
                    .count();
                let i = g.idx(rr, cc);
                g.cells[i].adjacent = n as u8;
            }
        }
        g.state = State::Playing;
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mine_count(g: &Game) -> usize {
        (0..g.rows())
            .flat_map(|r| (0..g.cols()).map(move |c| (r, c)))
            .filter(|&(r, c)| g.cell(r, c).unwrap().mine)
            .count()
    }

    #[test]
    fn presets_have_classic_sizes() {
        assert_eq!(
            (Config::EASY.rows, Config::EASY.cols, Config::EASY.mines),
            (9, 9, 10)
        );
        assert_eq!(
            (
                Config::INTERMEDIATE.rows,
                Config::INTERMEDIATE.cols,
                Config::INTERMEDIATE.mines
            ),
            (16, 16, 40)
        );
        assert_eq!(
            (
                Config::EXPERT.cols,
                Config::EXPERT.rows,
                Config::EXPERT.mines
            ),
            (30, 16, 99)
        );
        assert_eq!(Config::EXPERT.difficulty(), Difficulty::Expert);
        assert_eq!(
            Config::custom(10, 10, 10).unwrap().difficulty(),
            Difficulty::Custom
        );
    }

    #[test]
    fn board_size_and_exact_mine_count() {
        for (i, cfg) in [Config::EASY, Config::INTERMEDIATE, Config::EXPERT]
            .into_iter()
            .enumerate()
        {
            let mut g = Game::new(cfg, Rng::new(42 + i as u64));
            assert_eq!(g.state(), State::NotStarted);
            assert_eq!(mine_count(&g), 0, "minas só existem depois do 1º clique");
            g.reveal(cfg.rows / 2, cfg.cols / 2);
            assert_eq!(g.cells.len(), cfg.rows * cfg.cols);
            assert_eq!(mine_count(&g), cfg.mines);
        }
    }

    #[test]
    fn first_click_is_always_safe_including_neighbors() {
        for seed in 0..300 {
            // Máximo de minas permitido: o pior caso para a área segura.
            let cfg = Config::custom(5, 5, 25 - SAFE_AREA).unwrap();
            let mut g = Game::new(cfg, Rng::new(seed));
            let (r, c) = ((seed % 5) as usize, (seed / 5 % 5) as usize);
            assert_ne!(g.reveal(r, c), Outcome::Exploded, "semente {seed}");
            assert!(!g.cell(r, c).unwrap().mine);
            for (a, b) in g.neighbors(r, c) {
                assert!(
                    !g.cell(a, b).unwrap().mine,
                    "vizinha minada, semente {seed}"
                );
            }
            assert_eq!(mine_count(&g), 16);
        }
    }

    #[test]
    fn neighbor_counts_and_bounds() {
        let g = Game::with_mines(3, 3, &[(0, 0), (2, 2)]);
        assert_eq!(g.cell(1, 1).unwrap().adjacent, 2);
        assert_eq!(g.cell(0, 1).unwrap().adjacent, 1);
        assert_eq!(g.cell(0, 2).unwrap().adjacent, 0);
        assert_eq!(g.neighbors(0, 0).len(), 3, "canto");
        assert_eq!(g.neighbors(0, 1).len(), 5, "borda");
        assert_eq!(g.neighbors(1, 1).len(), 8, "centro");
        assert!(g.cell(3, 0).is_none() && g.cell(0, 3).is_none());
    }

    #[test]
    fn out_of_bounds_actions_are_ignored() {
        let mut g = Game::with_mines(3, 3, &[(0, 0)]);
        assert_eq!(g.reveal(9, 9), Outcome::Nothing);
        assert!(!g.toggle_flag(3, 0));
        assert_eq!(g.chord(0, 7), Outcome::Nothing);
    }

    #[test]
    fn reveal_number_opens_one_cell() {
        let mut g = Game::with_mines(3, 3, &[(0, 0)]);
        assert_eq!(g.reveal(1, 1), Outcome::Revealed(1));
        assert!(g.cell(1, 1).unwrap().revealed);
        assert!(!g.cell(2, 2).unwrap().revealed);
    }

    #[test]
    fn flood_fill_opens_empty_region_and_border() {
        // Mina no canto: todo o resto se abre a partir do canto oposto → vitória.
        let mut g = Game::with_mines(4, 4, &[(0, 0)]);
        assert_eq!(g.reveal(3, 3), Outcome::Won);
        assert_eq!(g.revealed, 15);
        // Região separada por uma "parede" de números não vaza.
        let mut g = Game::with_mines(5, 5, &[(0, 2), (1, 2), (2, 2), (3, 2), (4, 2)]);
        assert!(matches!(g.reveal(2, 0), Outcome::Revealed(10)));
        assert!(!g.cell(2, 4).unwrap().revealed);
    }

    #[test]
    fn flood_fill_is_iterative_on_largest_board() {
        let cfg = Config::custom(MAX_ROWS, MAX_COLS, 1).unwrap();
        let mut g = Game::new(cfg, Rng::new(7));
        assert!(matches!(
            g.reveal(0, 0),
            Outcome::Revealed(_) | Outcome::Won
        ));
        assert!(g.revealed > 900);
    }

    #[test]
    fn flag_toggle_and_counter() {
        let mut g = Game::with_mines(3, 3, &[(0, 0)]);
        assert!(g.toggle_flag(0, 0));
        assert_eq!((g.flags(), g.mines_left()), (1, 0));
        assert!(g.cell(0, 0).unwrap().flagged);
        assert_eq!(
            g.reveal(0, 0),
            Outcome::Nothing,
            "célula com bandeira não abre"
        );
        assert!(g.toggle_flag(0, 0));
        assert_eq!((g.flags(), g.mines_left()), (0, 1));
        g.reveal(1, 1);
        assert!(!g.toggle_flag(1, 1), "célula revelada não recebe bandeira");
        assert!(g.toggle_flag(2, 2) && g.toggle_flag(2, 1));
        assert_eq!(g.mines_left(), -1, "contador pode ficar negativo");
    }

    #[test]
    fn losing_reveals_mines_and_freezes_board() {
        let mut g = Game::with_mines(3, 3, &[(0, 0), (2, 2)]);
        assert_eq!(g.reveal(0, 0), Outcome::Exploded);
        assert_eq!(g.state(), State::Lost);
        assert_eq!(g.exploded(), Some((0, 0)));
        assert!(g.cell(2, 2).unwrap().revealed, "todas as minas aparecem");
        assert_eq!(g.reveal(1, 1), Outcome::Nothing);
        assert!(!g.toggle_flag(1, 2));
    }

    #[test]
    fn winning_flags_all_mines() {
        let mut g = Game::with_mines(2, 2, &[(0, 0)]);
        assert_eq!(g.reveal(0, 1), Outcome::Revealed(1));
        assert_eq!(g.reveal(1, 0), Outcome::Revealed(1));
        assert_eq!(g.reveal(1, 1), Outcome::Won);
        assert_eq!(g.state(), State::Won);
        assert!(g.cell(0, 0).unwrap().flagged);
        assert_eq!(g.mines_left(), 0);
    }

    #[test]
    fn chord_reveals_when_flags_match() {
        let mut g = Game::with_mines(3, 3, &[(0, 0)]);
        g.reveal(1, 1);
        assert_eq!(g.chord(1, 1), Outcome::Nothing, "sem bandeiras suficientes");
        g.toggle_flag(0, 0);
        assert_eq!(g.chord(1, 1), Outcome::Won);
        // Bandeira errada + chord = derrota.
        let mut g = Game::with_mines(3, 3, &[(0, 0)]);
        g.reveal(1, 1);
        g.toggle_flag(0, 1);
        assert_eq!(g.chord(1, 1), Outcome::Exploded);
    }

    #[test]
    fn restart_clears_everything() {
        let mut g = Game::new(Config::EASY, Rng::new(3));
        g.reveal(4, 4);
        g.toggle_flag(0, 0);
        g.restart();
        assert_eq!(g.state(), State::NotStarted);
        assert_eq!((g.flags(), g.revealed, mine_count(&g)), (0, 0, 0));
        assert_eq!(g.config(), Config::EASY);
    }

    #[test]
    fn invalid_custom_configs_are_rejected() {
        assert!(Config::custom(4, 10, 5).is_err());
        assert!(Config::custom(10, 41, 5).is_err());
        assert!(Config::custom(25, 10, 5).is_err());
        assert!(Config::custom(10, 10, 0).is_err());
        assert!(
            Config::custom(10, 10, 92).is_err(),
            "100 − 9 = 91 é o máximo"
        );
        assert!(Config::custom(10, 10, 91).is_ok());
        assert!(
            Config::custom(5, 5, 16).is_ok(),
            "menor tabuleiro permitido"
        );
    }

    #[test]
    fn rng_is_uniform_enough_and_deterministic() {
        let (mut a, mut b) = (Rng::new(9), Rng::new(9));
        assert_eq!(a.next_u64(), b.next_u64());
        let mut hist = [0usize; 6];
        for _ in 0..6000 {
            hist[a.below(6)] += 1;
        }
        assert!(hist.iter().all(|&h| (800..1200).contains(&h)), "{hist:?}");
    }
}
