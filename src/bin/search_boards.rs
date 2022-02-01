#[allow(unused_imports)]
use tile_game::big_set::{BigSet, BloomSet, HashedItemSet, PartitionSet};

use tile_game::big_stack::{BigStack, Stack};

use tile_game::board::Board;

fn factorial(x: usize) -> usize {
    (2..=x).fold(1, |x, y| x * y)
}

fn main() {
    // This the width and height of the tile game we're using.
    let (w, h) = (3, 3);
    let n_elts = (w * h) as u8;
    let n_solns = factorial(w * h) / 2;
    println!("Board size: {w}x{h}");
    println!("Anticipated solutions: {n_solns}");

    let board = Board::new(0..n_elts, &(w, h));
    let n_solns = find_all_boards_iteratively(board, n_solns);
    println!("There are {} solutions.", n_solns);
}

/// Counts the number of boards which are accessible from the starting position.
///
/// # Arguments
///
/// * `board` - The starting position.
/// * `n_solns` - The expected number of solutions.
fn find_all_boards_iteratively(board: Board, n_solns: usize) -> usize {
    let mut unprocessed_boards: BigStack<Board> = BigStack::new(1 << 25);
    unprocessed_boards.push(board.clone());

    let mut all_boards = PartitionSet::<Board>::new(1 << 18, n_solns, 0.75, 128);
    all_boards.insert(&board);

    while let Some(board) = unprocessed_boards.pop() {
        for permuted_board in board.slide_iter() {
            if !all_boards.contains(&permuted_board) {
                unprocessed_boards.push(permuted_board.clone());
                all_boards.insert(&permuted_board);
                if all_boards.len() % 1000000 == 0 {
                    println!(
                        "Processed {} boards with {} to go.",
                        all_boards.len(),
                        unprocessed_boards.len()
                    );
                }
            }
        }
    }
    all_boards.len()
}
