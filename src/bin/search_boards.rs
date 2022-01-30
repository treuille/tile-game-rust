use ndarray::{Array, Array2};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use tile_game::big_set::{BigSet, BloomSet, HashedItemSet};

use tile_game::big_stack::{BigStack, Stack};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
struct Board(Array2<u8>);

type Pt = [usize; 2];

impl Board {
    fn new<I>(iter: I, shape: &(usize, usize)) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        Board(Array::from_iter(iter).into_shape(*shape).unwrap())
    }

    fn permute(self: &Self, pt_1: &Pt, pt_2: &Pt) -> Self {
        let board = &self.0;
        let mut output_board = board.clone();
        output_board[*pt_1] = board[*pt_2];
        output_board[*pt_2] = board[*pt_1];
        Board(output_board)
    }

    fn slide_iter(self: &Self) -> impl Iterator<Item = Self> + '_ {
        // Extract the board and its dimensions.
        let board = &self.0;
        let w = board.shape()[0] as i32;
        let h = board.shape()[1] as i32;

        // The location of the zero in the board.
        let (x1, y1): (i32, i32) = board
            .indexed_iter()
            .find_map(|((i, j), x)| match x {
                0 => Some((i as i32, j as i32)),
                _ => None,
            })
            .unwrap();

        let offsets: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        offsets.into_iter().filter_map(move |(dx, dy)| {
            let x2: i32 = x1 + dx;
            let y2: i32 = y1 + dy;
            if x2 >= 0 && x2 < w && y2 >= 0 && y2 < h {
                let pt1: Pt = [x1 as usize, y1 as usize];
                let pt2: Pt = [x2 as usize, y2 as usize];
                Some(self.permute(&pt1, &pt2))
            } else {
                None
            }
        })
    }
}
fn factorial(x: usize) -> usize {
    (2..=x).fold(1, |x, y| x * y)
}

fn main() {
    // This the width and height of the tile game we're using.
    let (w, h) = (3, 4);
    let n_elts = (w * h) as u8;
    let n_solns = factorial(w * h) / 2;
    println!("Board size: {w}x{h}");
    println!("Anticipated solitions: {n_solns}");

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

    // let mut all_boards = BigSet::<Board>::new(1 << 25);
    let mut all_boards = BloomSet::<Board>::new(1 << 25, n_solns, 0.75);
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
                // if (all_boards.len() + 1) % 10000000 == 0 {
                //     panic!(
                //         "Exiting early at {} with {} to go.",
                //         all_boards.len(),
                //         unprocessed_boards.len()
                //     );
                // }
            }
        }
    }
    all_boards.len()
}

#[test]
fn test_board_perumute() {
    let a1 = Board::new([0, 1, 2, 3], &(2, 2));
    let a2 = Board::new([1, 0, 2, 3], &(2, 2));
    let a3 = a1.permute(&[0, 0], &[0, 1]);
    assert_eq!(a2, a3);
}
