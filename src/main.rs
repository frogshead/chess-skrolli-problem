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

use rand::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Board {
    size: i32,
    //cells: Vec<Point>,
    start: Point,
    current: Point,
    win: Point,
    loose: Point,
}

struct Game {
    board: Board,
    stats: GameStats,
    moves: Vec<u64>,
}

impl Game {
    fn new() -> Self {
        Game {
            board: Board::new(BOARD_SIZE, START_POSITION, WIN_POSITION, LOOSE_POSITION),
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
                moves = moves + 1;
                if self.board.current == self.board.win {
                    self.stats.wins += 1;
                    self.moves.push(moves);
                    //println!("Number of moves: {:?}", self.moves);
                    break;
                }
                if self.board.current == self.board.loose {
                    self.stats.looses += 1;
                    self.moves.push(moves);
                    //println!("Number of moves: {:?}", self.moves);
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
const BOARD_SIZE: i32 = 100; // Chess board size BOARD_SIZE x BOARD_SIZE
const START_POSITION: Point = Point { x: 0, y: 0 };
const WIN_POSITION: Point = Point {
    x: BOARD_SIZE - 1,
    y: BOARD_SIZE - 1,
};
const LOOSE_POSITION: Point = Point {
    x: BOARD_SIZE - 1,
    y: 0,
};

impl Board {
    fn new(size: i32, start: Point, win: Point, loose: Point) -> Self {
        Board {
            size,
            //cells: populate_board(size),
            start: start.clone(),
            current: start.clone(),
            win,
            loose,
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
                    let p = Point {
                        x: self.current.x - 1,
                        y: self.current.y + 2,
                    };
                    if self.is_valid_point(&p) {
                        v.push(p);
                    }
                    let pp = Point {
                        x: self.current.x + 1,
                        y: self.current.y + 2,
                    };
                    if self.is_valid_point(&pp) {
                        v.push(pp);
                    }
                }
                Direction::EAST => {
                    let p = Point {
                        x: self.current.x + 2,
                        y: self.current.y - 1,
                    };
                    if self.is_valid_point(&p) {
                        v.push(p);
                    }

                    let pp = Point {
                        x: self.current.x + 2,
                        y: self.current.y + 1,
                    };
                    if self.is_valid_point(&pp) {
                        v.push(pp);
                    }
                }
                Direction::SOUTH => {
                    let p = Point {
                        x: self.current.x - 1,
                        y: self.current.y - 2,
                    };
                    if self.is_valid_point(&p) {
                        v.push(p);
                    }
                    let pp = Point {
                        x: self.current.x + 1,
                        y: self.current.y - 2,
                    };
                    if self.is_valid_point(&pp) {
                        v.push(pp);
                    }
                }
                Direction::WEST => {
                    let p = Point {
                        x: self.current.x - 2,
                        y: self.current.y + 1,
                    };
                    if self.is_valid_point(&p) {
                        v.push(p);
                    }
                    let pp = Point {
                        x: self.current.x - 2,
                        y: self.current.y - 1,
                    };
                    if self.is_valid_point(&pp) {
                        v.push(pp);
                    }
                }
            }
        }
        v
    }

    fn is_valid_point(&self, p: &Point) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.size && p.y < self.size
    }
}

fn populate_board(size: i32) -> Vec<Point> {
    let mut v = vec![];
    for x in 0..size {
        for y in 0..size {
            v.push(Point { x: x, y: y });
        }
    }
    v
}

fn main() {
    let iterations = 100_000;
    let mut game = Game::new();
    game.run(iterations);
    let total = game.stats.wins + game.stats.looses;
    let probability = game.stats.wins as f64 / total as f64;
    println!("Win probability ({iterations} iterations): {probability:}");
    let sum: u64 = game.moves.iter().sum();
    let cnt: u64 = game.moves.len() as u64;
    println!("Average move count: {:?}", sum / cnt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_population() {
        assert_eq!(
            BOARD_SIZE * BOARD_SIZE,
            Board::new(BOARD_SIZE, START_POSITION, WIN_POSITION, LOOSE_POSITION)
                .cells
                .len() as i32
        )
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
