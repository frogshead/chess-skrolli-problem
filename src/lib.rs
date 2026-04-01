use rand::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
pub struct Board {
    pub size: i32,
    pub start: Point,
    pub current: Point,
    pub win: Point,
    pub loose: Point,
    pub visit_counts: Vec<Vec<u64>>,
}

pub struct Game {
    pub board: Board,
    pub stats: GameStats,
    pub moves: Vec<u64>,
}

impl Game {
    pub fn new(size: i32) -> Self {
        let start = Point { x: 0, y: 0 };
        let win = Point { x: size - 1, y: size - 1 };
        let loose = Point { x: size - 1, y: 0 };
        Game {
            board: Board::new(size, start, win, loose),
            stats: GameStats { wins: 0, looses: 0 },
            moves: vec![],
        }
    }

    pub fn run(&mut self, iterations: u64) {
        for _ in 0..iterations {
            self.board.reset();
            let mut moves: u64 = 0;
            loop {
                self.board.next_move();
                moves += 1;
                if self.board.current == self.board.win {
                    self.stats.wins += 1;
                    self.moves.push(moves);
                    break;
                }
                if self.board.current == self.board.loose {
                    self.stats.looses += 1;
                    self.moves.push(moves);
                    break;
                }
            }
        }
    }
}

pub struct GameStats {
    pub wins: u64,
    pub looses: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    NORTH = 0,
    EAST = 1,
    SOUTH = 2,
    WEST = 3,
}

pub const BOARD_SIZE: i32 = 8;

impl Board {
    pub fn new(size: i32, start: Point, win: Point, loose: Point) -> Self {
        Board {
            size,
            start: start.clone(),
            current: start.clone(),
            win,
            loose,
            visit_counts: vec![vec![0u64; size as usize]; size as usize],
        }
    }

    pub fn reset(&mut self) {
        self.current = self.start.clone();
    }

    pub fn next_move(&mut self) {
        let mut rng = rand::rng();
        let valid_moves: Vec<Point> = self.get_valid_moves();

        let p = valid_moves.choose(&mut rng);
        match p {
            Some(pp) => {
                self.current = pp.clone();
                self.visit_counts[self.current.x as usize][self.current.y as usize] += 1;
            }
            None => {
                panic!("No valid moves")
            }
        }
    }

    pub fn get_valid_moves(&self) -> Vec<Point> {
        let mut v = vec![];
        const VARIANTS: [Direction; 4] = [
            Direction::NORTH,
            Direction::EAST,
            Direction::SOUTH,
            Direction::WEST,
        ];
        for dir in VARIANTS.iter() {
            match dir {
                Direction::NORTH => {
                    let p = Point { x: self.current.x - 1, y: self.current.y + 2 };
                    if self.is_valid_point(&p) { v.push(p); }
                    let pp = Point { x: self.current.x + 1, y: self.current.y + 2 };
                    if self.is_valid_point(&pp) { v.push(pp); }
                }
                Direction::EAST => {
                    let p = Point { x: self.current.x + 2, y: self.current.y - 1 };
                    if self.is_valid_point(&p) { v.push(p); }
                    let pp = Point { x: self.current.x + 2, y: self.current.y + 1 };
                    if self.is_valid_point(&pp) { v.push(pp); }
                }
                Direction::SOUTH => {
                    let p = Point { x: self.current.x - 1, y: self.current.y - 2 };
                    if self.is_valid_point(&p) { v.push(p); }
                    let pp = Point { x: self.current.x + 1, y: self.current.y - 2 };
                    if self.is_valid_point(&pp) { v.push(pp); }
                }
                Direction::WEST => {
                    let p = Point { x: self.current.x - 2, y: self.current.y + 1 };
                    if self.is_valid_point(&p) { v.push(p); }
                    let pp = Point { x: self.current.x - 2, y: self.current.y - 1 };
                    if self.is_valid_point(&pp) { v.push(pp); }
                }
            }
        }
        v
    }

    pub fn is_valid_point(&self, p: &Point) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.size && p.y < self.size
    }
}

/// Map a ratio 0.0–1.0 to a heat color: dark blue → cyan → yellow → red.
/// Returns (r, g, b).
pub fn heat_color(ratio: f64) -> (u8, u8, u8) {
    let r = ratio.clamp(0.0, 1.0);
    if r < 0.25 {
        let t = r / 0.25;
        (0, (t * 100.0) as u8, (55.0 + t * 200.0) as u8)
    } else if r < 0.5 {
        let t = (r - 0.25) / 0.25;
        (0, (100.0 + t * 155.0) as u8, (255.0 - t * 255.0) as u8)
    } else if r < 0.75 {
        let t = (r - 0.5) / 0.25;
        ((t * 255.0) as u8, 255, 0)
    } else {
        let t = (r - 0.75) / 0.25;
        (255, (255.0 - t * 255.0) as u8, 0)
    }
}

pub fn format_move_stats(moves: &[u64]) -> String {
    let cnt = moves.len();
    if cnt == 0 {
        return String::new();
    }
    let sum: u64 = moves.iter().sum();
    let mean = sum as f64 / cnt as f64;
    let variance = moves
        .iter()
        .map(|&m| {
            let diff = m as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / cnt as f64;
    let std_dev = variance.sqrt();

    let mut sorted = moves.to_vec();
    sorted.sort_unstable();
    let median = if cnt % 2 == 0 {
        (sorted[cnt / 2 - 1] + sorted[cnt / 2]) as f64 / 2.0
    } else {
        sorted[cnt / 2] as f64
    };

    format!(
        "Move count — avg: {:.1}  median: {:.1}  min: {}  max: {}  std: {:.1}",
        mean,
        median,
        sorted[0],
        sorted[cnt - 1],
        std_dev
    )
}

// ── WebAssembly bindings ───────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmGame {
    board: Board,
    stats: GameStats,
    moves: Vec<u64>,
    move_num: u64,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new(size: i32) -> WasmGame {
        let start = Point { x: 0, y: 0 };
        let win = Point { x: size - 1, y: size - 1 };
        let loose = Point { x: size - 1, y: 0 };
        WasmGame {
            board: Board::new(size, start, win, loose),
            stats: GameStats { wins: 0, looses: 0 },
            moves: vec![],
            move_num: 0,
        }
    }

    /// Advance one knight move for step-by-step animation.
    /// Returns: 0 = ongoing, 1 = win, 2 = loss
    pub fn step(&mut self) -> u8 {
        self.board.next_move();
        self.move_num += 1;
        if self.board.current == self.board.win {
            self.stats.wins += 1;
            self.moves.push(self.move_num);
            1
        } else if self.board.current == self.board.loose {
            self.stats.looses += 1;
            self.moves.push(self.move_num);
            2
        } else {
            0
        }
    }

    /// Reset board position for a new animated game (does NOT clear stats or heatmap).
    pub fn reset_board(&mut self) {
        self.board.reset();
        self.move_num = 0;
    }

    /// Run N complete games headlessly (accumulates into stats and visit_counts).
    pub fn run_simulation(&mut self, iterations: u32) {
        for _ in 0..iterations {
            self.board.reset();
            let mut moves: u64 = 0;
            loop {
                self.board.next_move();
                moves += 1;
                if self.board.current == self.board.win {
                    self.stats.wins += 1;
                    self.moves.push(moves);
                    break;
                }
                if self.board.current == self.board.loose {
                    self.stats.looses += 1;
                    self.moves.push(moves);
                    break;
                }
            }
        }
    }

    /// Returns normalized heatmap as flat f64 array (values 0.0–1.0).
    /// Index: x * size + y (column-major, matching visit_counts[x][y]).
    pub fn get_heatmap_normalized(&self) -> Vec<f64> {
        let size = self.board.size as usize;
        let max_count = self
            .board
            .visit_counts
            .iter()
            .flat_map(|col| col.iter())
            .copied()
            .max()
            .unwrap_or(1)
            .max(1);
        let mut result = vec![0.0f64; size * size];
        for x in 0..size {
            for y in 0..size {
                result[x * size + y] =
                    self.board.visit_counts[x][y] as f64 / max_count as f64;
            }
        }
        result
    }

    pub fn board_size(&self) -> i32 {
        self.board.size
    }
    pub fn current_x(&self) -> i32 {
        self.board.current.x
    }
    pub fn current_y(&self) -> i32 {
        self.board.current.y
    }
    pub fn move_num(&self) -> u32 {
        self.move_num as u32
    }
    pub fn wins(&self) -> u32 {
        self.stats.wins as u32
    }
    pub fn losses(&self) -> u32 {
        self.stats.looses as u32
    }

    pub fn win_probability(&self) -> f64 {
        let total = self.stats.wins + self.stats.looses;
        if total == 0 {
            0.0
        } else {
            self.stats.wins as f64 / total as f64
        }
    }

    pub fn move_stats(&self) -> String {
        format_move_stats(&self.moves)
    }

    /// Full reset including stats, visit_counts, and move history.
    pub fn reset_game(&mut self) {
        self.board.reset();
        self.board.visit_counts =
            vec![vec![0u64; self.board.size as usize]; self.board.size as usize];
        self.stats = GameStats { wins: 0, looses: 0 };
        self.moves.clear();
        self.move_num = 0;
    }
}

/// Compute RGB heat color for a ratio 0.0–1.0. Returns [r, g, b] as Vec<u8>.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn heat_color_js(ratio: f64) -> Vec<u8> {
    let (r, g, b) = heat_color(ratio);
    vec![r, g, b]
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "cells field removed from Board"]
    fn test_board_population() {
        fn populate_board(size: i32) -> Vec<Point> {
            let mut v = vec![];
            for x in 0..size {
                for y in 0..size {
                    v.push(Point { x, y });
                }
            }
            v
        }
        assert_eq!(BOARD_SIZE * BOARD_SIZE, populate_board(BOARD_SIZE).len() as i32);
    }

    #[test]
    fn test_next_moves() {
        let start: Point = Point { x: 0, y: 0 };
        let win: Point = Point { x: 2, y: 2 };
        let loose: Point = Point { x: 2, y: 0 };
        let board = Board::new(3, start, win, loose);
        assert_eq!(
            board.get_valid_moves(),
            vec![Point { x: 1, y: 2 }, Point { x: 2, y: 1 }]
        );
    }
}
