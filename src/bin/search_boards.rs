use rayon::prelude::*;

#[allow(unused_imports)]
use tile_game::big_set::{BigSet, BloomSet, HashedItemSet, PartitionSet};

use tile_game::big_stack::{BigStack, Stack};

use tile_game::board::Board;

use std::mem;
use std::sync::{Arc, Mutex};

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
    // let n_solns = find_all_boards_iteratively(board, n_solns);
    let n_solns = find_all_boards_in_parallel(board);
    println!("There are {} solutions.", n_solns);
}

/// Counts the number of boards which are accessible from the starting position.
fn find_all_boards_in_parallel(board: Board) -> usize {
    let cache_size = 50000; // 1 << 25;
    let local_board_queue_len = 1 << 10;

    let mut unprocessed_boards: BigStack<Board> = BigStack::new(cache_size);
    unprocessed_boards.push(board.clone());

    let all_boards: Arc<Mutex<BigSet<Board>>> = Arc::new(Mutex::new(BigSet::new(1 << 19)));

    while unprocessed_boards.len() != 0 {
        let next_unprocessed_boards: Arc<Mutex<BigStack<Board>>> =
            Arc::new(Mutex::new(BigStack::new(cache_size)));

        loop {
            // Take up to `local_board_queue_len` elements from `local_board_queue`.
            let local_board_queue: Vec<Board> = (0..local_board_queue_len)
                .map_while(|_| unprocessed_boards.pop())
                .collect();
            if local_board_queue.len() == 0 {
                break;
            }
            local_board_queue.par_iter().for_each(|board| {
                let mut all_boards = all_boards.lock().unwrap();
                if !all_boards.contains(&board) {
                    all_boards.insert(&board);
                    // if all_boards.len() % 1000000 == 0 {
                    //     println!(
                    //         "Processed {} boards with {} to go.",
                    //         all_boards.len(),
                    //         unprocessed_boards.len() + next_unprocessed_boards.len()
                    //     );
                    // }
                    let mut next_unprocessed_boards = next_unprocessed_boards.lock().unwrap();
                    for permuted_board in board.slide_iter() {
                        next_unprocessed_boards.push(permuted_board);
                    }
                }
            })
        }
        {
            let mut next_unprocessed_boards = next_unprocessed_boards.lock().unwrap();
            mem::swap(&mut unprocessed_boards, &mut next_unprocessed_boards);
        }
        // println!(
        //     "Finished a rung {} boards with ({}, {}) to go.",
        //     all_boards.len(),
        //     unprocessed_boards.len(),
        //     next_unprocessed_boards.len()
        // );
    }

    let boards_found = {
        let all_boards = all_boards.lock().unwrap();
        all_boards.len()
    };
    boards_found
}

/// Counts the number of boards which are accessible from the starting position.
///
/// # Arguments
///
/// * `board` - The starting position.
/// * `n_solns` - The expected number of solutions.
#[allow(dead_code)]
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
