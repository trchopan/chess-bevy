use chess::{Board, ALL_SQUARES};

fn main() {
    let board = Board::default();
    for sq in ALL_SQUARES.iter() {
        println!("{:?} -> {:?}", sq, board.piece_on(*sq));
    }
}
