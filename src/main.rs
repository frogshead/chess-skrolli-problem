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

use std::io::{self, Write as IoWrite};
use std::thread;
use std::time::Duration;

use chess_skrolli_problem::{
    format_move_stats, heat_color, Board, Game, GameStats, Point, BOARD_SIZE,
};
use clap::Parser;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};

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

    stdout.queue(Print("    "))?;
    if compact {
        for x in 0..size as usize {
            if x % 10 == 0 {
                stdout.queue(Print(format!("{:<20}", x)))?;
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
            let (r, g, b) = heat_color(ratio);
            let bg = Color::Rgb { r, g, b };
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

    stdout.queue(Print("\nLegend: "))?;
    let steps = 20usize;
    for i in 0..steps {
        let ratio = i as f64 / (steps - 1) as f64;
        let (r, g, b) = heat_color(ratio);
        stdout
            .queue(SetBackgroundColor(Color::Rgb { r, g, b }))?
            .queue(Print("  "))?
            .queue(ResetColor)?;
    }
    stdout.queue(Print(" cold → hot\n\n"))?;
    stdout.flush()?;

    let total = game.stats.wins + game.stats.looses;
    let probability = game.stats.wins as f64 / total as f64;
    println!("Win probability ({} iterations): {:.10}", iterations, probability);
    println!("{}", format_move_stats(&game.moves));

    Ok(())
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
        println!("{}", format_move_stats(&game.moves));
    }
}
