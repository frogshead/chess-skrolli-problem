// Shakkilaudan vasemmassa alakulmassa on ratsu. Joka vuorol-
// la ratsu valitsee satunnaisen siirron laillisten siirtojen joukosta,
// jokaisen yhtä todennäköisesti. Jos ratsu astuu oikeaan yläkul-
// maan, peli päättyy ja ratsu voittaa. Jos ratsu astuu oikeaan ala-
// kulmaan, peli päättyy ja ratsu häviää. Mikä on ratsun todennä-
// köisyys voittaa peli, kun laudan koko on:
// a) 3 × 3? b) 100 × 100?
// Anna vastaus 10 desimaalin tarkkuudella.
//
// Sha1 tiiviste oikeista vastauksista
// 2a: 33%
// 2a: 8cf1a37987d0c3f177e7eaa8ef15efe2c317b858
// 2b: 50%
// 2b: 215d1cff21a98027d115c6812f841f936d781b05
//
//

use std::io::{self, Write as IoWrite};
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use rand::prelude::*;

#[derive(Parser)]
#[command(about = "Knight random walk simulation")]
struct Cli {
    /// Animate a single game step-by-step
    #[arg(long)]
    visualize: bool,

    /// Run N iterations and show a visit-frequency heatmap
    #[arg(long)]
    heatmap: bool,

    /// Number of simulation iterations
    #[arg(long, default_value_t = 100_000)]
    iterations: u64,

    /// Delay between moves in milliseconds (visualize mode)
    #[arg(long, default_value_t = 200)]
    delay: u64,

    /// Board size (overrides the compiled-in BOARD_SIZE constant)
    #[arg(long)]
    board_size: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Board {
    size: i32,
    start: Point,
    current: Point,
    win: Point,
    loose: Point,
    visit_counts: Vec<Vec<u64>>,
}

struct Game {
    board: Board,
    stats: GameStats,
    moves: Vec<u64>,
}

impl Game {
    fn new(size: i32) -> Self {
        let start = Point { x: 0, y: 0 };
        let win = Point { x: size - 1, y: size - 1 };
        let loose = Point { x: size - 1, y: 0 };
        Game {
            board: Board::new(size, start, win, loose),
            stats: GameStats { wins: 0, looses: 0 },
            moves: vec![],
        }
    }

    fn run(&mut self, iterations: u64) {
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

struct GameStats {
    wins: u64,
    looses: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    NORTH = 0,
    EAST = 1,
    SOUTH = 2,
    WEST = 3,
}

const BOARD_SIZE: i32 = 8;

impl Board {
    fn new(size: i32, start: Point, win: Point, loose: Point) -> Self {
        Board {
            size,
            start: start.clone(),
            current: start.clone(),
            win,
            loose,
            visit_counts: vec![vec![0u64; size as usize]; size as usize],
        }
    }

    fn reset(&mut self) {
        self.current = self.start.clone();
    }

    fn next_move(&mut self) {
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

    fn get_valid_moves(&self) -> Vec<Point> {
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

    fn is_valid_point(&self, p: &Point) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.size && p.y < self.size
    }
}

// ── Visualization helpers ──────────────────────────────────────────────────

/// Map a ratio 0.0–1.0 to a heat color: dark blue → blue → cyan → yellow → red.
fn heat_color(ratio: f64) -> Color {
    let r = ratio.clamp(0.0, 1.0);
    if r < 0.25 {
        let t = r / 0.25;
        Color::Rgb { r: 0, g: (t * 100.0) as u8, b: (55.0 + t * 200.0) as u8 }
    } else if r < 0.5 {
        let t = (r - 0.25) / 0.25;
        Color::Rgb { r: 0, g: (100.0 + t * 155.0) as u8, b: (255.0 - t * 255.0) as u8 }
    } else if r < 0.75 {
        let t = (r - 0.5) / 0.25;
        Color::Rgb { r: (t * 255.0) as u8, g: 255, b: 0 }
    } else {
        let t = (r - 0.75) / 0.25;
        Color::Rgb { r: 255, g: (255.0 - t * 255.0) as u8, b: 0 }
    }
}

// Above this size, switch to compact single-char-per-cell rendering.
const COMPACT_THRESHOLD: usize = 20;

/// Print the board in step-by-step visualization mode.
fn print_board_state(
    stdout: &mut io::Stdout,
    board: &Board,
    move_num: usize,
) -> io::Result<()> {
    let size = board.size as usize;
    let compact = size > COMPACT_THRESHOLD;
    stdout.execute(terminal::Clear(ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;

    if compact {
        // Column header: show every 10th column
        stdout.queue(Print("    "))?;
        for x in 0..size {
            if x % 10 == 0 {
                stdout.queue(Print(format!("{:<10}", x)))?;
            }
        }
        stdout.queue(Print("\n"))?;
    } else {
        stdout.queue(Print("    "))?;
        for x in 0..size {
            stdout.queue(Print(format!("{:2} ", x)))?;
        }
        stdout.queue(Print("\n"))?;
    }

    // Rows printed top (high y) to bottom (low y)
    for row in (0..size).rev() {
        stdout.queue(Print(format!("{:3} ", row)))?;
        for col in 0..size {
            let p = Point { x: col as i32, y: row as i32 };
            let is_knight = p == board.current;
            let is_win = p == board.win;
            let is_loose = p == board.loose;
            let is_start = p == board.start;

            if compact {
                if is_knight {
                    stdout.queue(SetForegroundColor(Color::Yellow))?.queue(Print("♞"))?.queue(ResetColor)?;
                } else if is_win {
                    stdout.queue(SetForegroundColor(Color::Green))?.queue(Print("★"))?.queue(ResetColor)?;
                } else if is_loose {
                    stdout.queue(SetForegroundColor(Color::Red))?.queue(Print("✗"))?.queue(ResetColor)?;
                } else if is_start {
                    stdout.queue(SetForegroundColor(Color::DarkGrey))?.queue(Print("S"))?.queue(ResetColor)?;
                } else {
                    stdout.queue(Print("."))?;
                }
            } else if is_knight {
                stdout.queue(SetForegroundColor(Color::Yellow))?.queue(Print(" ♞ "))?.queue(ResetColor)?;
            } else if is_win {
                stdout.queue(SetForegroundColor(Color::Green))?.queue(Print(" ★ "))?.queue(ResetColor)?;
            } else if is_loose {
                stdout.queue(SetForegroundColor(Color::Red))?.queue(Print(" ✗ "))?.queue(ResetColor)?;
            } else if is_start {
                stdout.queue(SetForegroundColor(Color::DarkGrey))?.queue(Print(" S "))?.queue(ResetColor)?;
            } else {
                stdout.queue(Print(" . "))?;
            }
        }
        stdout.queue(Print("\n"))?;
    }

    stdout.queue(Print(format!("\nMove: {}\n", move_num)))?;
    stdout.queue(Print("WIN=★  LOSE=✗  START=S  KNIGHT=♞\n"))?;
    stdout.flush()?;
    Ok(())
}

/// Run step-by-step animation of a single game.
fn run_visualize(size: i32, delay_ms: u64) -> io::Result<()> {
    let mut stdout = io::stdout();
    let start = Point { x: 0, y: 0 };
    let win = Point { x: size - 1, y: size - 1 };
    let loose = Point { x: size - 1, y: 0 };
    let mut board = Board::new(size, start, win, loose);

    stdout.execute(cursor::Hide)?;

    print_board_state(&mut stdout, &board, 0)?;
    thread::sleep(Duration::from_millis(delay_ms));

    let mut move_num = 0;
    let result = loop {
        board.next_move();
        move_num += 1;
        print_board_state(&mut stdout, &board, move_num)?;
        thread::sleep(Duration::from_millis(delay_ms));

        if board.current == board.win {
            break true;
        }
        if board.current == board.loose {
            break false;
        }
    };

    stdout.execute(cursor::Show)?;

    let outcome = if result { "WIN ★" } else { "LOSE ✗" };
    println!("\nGame over: {} after {} moves\n", outcome, move_num);
    Ok(())
}

/// Show a heatmap of visit frequencies after running N iterations.
fn run_heatmap(size: i32, iterations: u64) -> io::Result<()> {
    let mut stdout = io::stdout();
    let start = Point { x: 0, y: 0 };
    let win = Point { x: size - 1, y: size - 1 };
    let loose = Point { x: size - 1, y: 0 };
    let mut game = Game {
        board: Board::new(size, start, win, loose),
        stats: GameStats { wins: 0, looses: 0 },
        moves: vec![],
    };

    println!("Running {} iterations...", iterations);
    game.run(iterations);

    // Find max count for normalization
    let max_count = game
        .board
        .visit_counts
        .iter()
        .flat_map(|col| col.iter())
        .copied()
        .max()
        .unwrap_or(1)
        .max(1);

    let compact = size as usize > COMPACT_THRESHOLD;

    if compact {
        println!("\nHeatmap (color blocks — blue=cold, red=hot; needs ≥{} cols wide terminal):\n", size as usize * 2 + 4);
    } else {
        println!("\nHeatmap (visits per cell — blue=cold, red=hot):\n");
    }

    // Column header
    stdout.queue(Print("    "))?;
    if compact {
        // Show label every 10 columns; each cell is 2 chars wide
        for x in 0..size as usize {
            if x % 10 == 0 {
                stdout.queue(Print(format!("{:<20}", x)))?; // 10 cols × 2 chars
            }
        }
    } else {
        for x in 0..size as usize {
            stdout.queue(Print(format!("{:^7}", x)))?;
        }
    }
    stdout.queue(Print("\n"))?;

    for row in (0..size as usize).rev() {
        stdout.queue(Print(format!("{:3} ", row)))?;
        for col in 0..size as usize {
            let count = game.board.visit_counts[col][row];
            let ratio = count as f64 / max_count as f64;
            let bg = heat_color(ratio);
            if compact {
                stdout
                    .queue(SetBackgroundColor(bg))?
                    .queue(Print("  "))?
                    .queue(ResetColor)?;
            } else {
                stdout
                    .queue(SetBackgroundColor(bg))?
                    .queue(SetForegroundColor(Color::Black))?
                    .queue(Print(format!("{:^7}", count)))?
                    .queue(ResetColor)?;
            }
        }
        stdout.queue(Print("\n"))?;
    }

    // Legend
    stdout.queue(Print("\nLegend: "))?;
    let steps = 20usize;
    for i in 0..steps {
        let r = i as f64 / (steps - 1) as f64;
        stdout
            .queue(SetBackgroundColor(heat_color(r)))?
            .queue(Print("  "))?
            .queue(ResetColor)?;
    }
    stdout.queue(Print(" cold → hot\n\n"))?;
    stdout.flush()?;

    let total = game.stats.wins + game.stats.looses;
    let probability = game.stats.wins as f64 / total as f64;
    println!("Win probability ({} iterations): {:.10}", iterations, probability);
    print_move_stats(&game.moves);

    Ok(())
}

fn print_move_stats(moves: &[u64]) {
    let cnt = moves.len();
    if cnt == 0 {
        return;
    }
    let sum: u64 = moves.iter().sum();
    let mean = sum as f64 / cnt as f64;
    let variance = moves.iter().map(|&m| {
        let diff = m as f64 - mean;
        diff * diff
    }).sum::<f64>() / cnt as f64;
    let std_dev = variance.sqrt();

    let mut sorted = moves.to_vec();
    sorted.sort_unstable();
    let median = if cnt % 2 == 0 {
        (sorted[cnt / 2 - 1] + sorted[cnt / 2]) as f64 / 2.0
    } else {
        sorted[cnt / 2] as f64
    };

    println!("Move count — avg: {:.1}  median: {:.1}  min: {}  max: {}  std: {:.1}",
        mean, median, sorted[0], sorted[cnt - 1], std_dev);
}

fn main() {
    let cli = Cli::parse();
    let size = cli.board_size.unwrap_or(BOARD_SIZE);

    if cli.visualize {
        run_visualize(size, cli.delay).expect("visualization failed");
    } else if cli.heatmap {
        run_heatmap(size, cli.iterations).expect("heatmap failed");
    } else {
        let iterations = cli.iterations;
        let mut game = Game::new(size);
        game.run(iterations);
        let total = game.stats.wins + game.stats.looses;
        let probability = game.stats.wins as f64 / total as f64;
        println!("Win probability ({iterations} iterations): {probability:.10}");
        print_move_stats(&game.moves);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn populate_board(size: i32) -> Vec<Point> {
        let mut v = vec![];
        for x in 0..size {
            for y in 0..size {
                v.push(Point { x, y });
            }
        }
        v
    }

    #[test]
    #[ignore = "cells field removed from Board"]
    fn test_board_population() {
        // This test referenced board.cells which is no longer part of Board.
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
