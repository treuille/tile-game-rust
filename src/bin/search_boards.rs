use rayon::prelude::*;

#[allow(unused_imports)]
use tile_game::big_set::{
    BigSet, BloomSet, HashedItemSet, InteriorMutableSet, ParallelSet, PartitionSet,
};

use tile_game::big_stack::{BigStack, Stack};

use tile_game::board::Board;

use std::sync::{Arc, Mutex};
use std::{iter, mem};

fn factorial(x: usize) -> usize {
    (2..=x).fold(1, |x, y| x * y)
}

fn main() {
    // This the width and height of the tile game we're using.
    let (w, h) = (3, 4);
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
    let stack_cache_size = 1 << 25;
    let mut unprocessed_boards: BigStack<Board> = BigStack::new(stack_cache_size);
    unprocessed_boards.push(board.clone());

    // let all_boards: Arc<Mutex<BigSet<Board>>> = Arc::new(Mutex::new(BigSet::new(1 << 19)));
    let set_cache = 1 << 22;
    let n_partitions = 8;
    let all_boards: Arc<ParallelSet<Board>> =
        { Arc::new(ParallelSet::new(set_cache, n_partitions)) };
    // ParallelSet::new(cache_size: usize, n_partitions: usize)

    while unprocessed_boards.len() != 0 {
        let next_unprocessed_boards: Arc<Mutex<BigStack<Board>>> =
            Arc::new(Mutex::new(BigStack::new(stack_cache_size)));

        loop {
            // Take up to `parallel_buffer_len` elements from `parallel_buffer`.
            let parallel_buffer_len = 1 << 10;
            let parallel_buffer: Vec<Board> = iter::repeat(())
                .take(parallel_buffer_len)
                .map_while(|_| unprocessed_boards.pop())
                .collect();
            if parallel_buffer.len() == 0 {
                break;
            }
            parallel_buffer.par_iter().for_each(|board| {
                // let mut all_boards = all_boards.lock().unwrap();
                // fn insert_check(&self, item: &T) -> bool {
                if !all_boards.insert_check(&board) {
                    let mut next_unprocessed_boards = next_unprocessed_boards.lock().unwrap();
                    for permuted_board in board.slide_iter() {
                        next_unprocessed_boards.push(permuted_board);
                    }
                    if all_boards.len() % 1000000 == 0 {
                        println!(
                            "Processed {} boards with {} to go.",
                            all_boards.len(),
                            unprocessed_boards.len() + next_unprocessed_boards.len()
                        );
                    }
                }
            })
        }
        {
            let mut next_unprocessed_boards = next_unprocessed_boards.lock().unwrap();
            mem::swap(&mut unprocessed_boards, &mut next_unprocessed_boards);
        }
    }

    all_boards.len()
    // let boards_found = {
    //     let all_boards = all_boards.lock().unwrap();
    //     all_boards.len()
    // };
    // boards_found
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
