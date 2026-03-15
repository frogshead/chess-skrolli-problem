#[derive(Debug)]
struct Point {
    x: u8,
    y: u8,
}

struct GameStats {
    wins: u64,
    looses: u64,
}
enum Direction {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}
const BOARD_SIZE: u8 = 3; // Chess board size BOARD_SIZE x BOARD_SIZE
const START_POSITION: Point = Point { x: 0, y: 0 };

fn main() {
    println!("START: {:?}", START_POSITION);
}
