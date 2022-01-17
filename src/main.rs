// use ndarray::prelude::*;
use ndarray::{Array, Array2};

type Board = Array2<u8>;
type Pt = [usize; 2];

fn main() {
    // This the width and height of the tile game we're using.
    let (w, h) = (3, 3);
    let n_elts = (w * h) as u8;
    println!("Board size: {}x{}", w, h);

    let a: Board = Array::from_iter(0..n_elts).into_shape((w, h)).unwrap();

    println!("{:?}", a);

    let a1: Board = Array::from_iter([0, 1, 2, 3]).into_shape((2, 2)).unwrap();
    let a2: Board = Array::from_iter([1, 0, 2, 3]).into_shape((2, 2)).unwrap();
    let a3: Board = permute(&a1, &[0, 0], &[0, 1]);
    println!("{}", a1);
    println!("{}", a2);
    println!("{}", a3);
    assert_eq!(a2, a3);
    println!("All tests passed!");
    // assert_eq!(a.ndim(), 1); // get the number of dimensions of array a
    // assert_eq!(a.len(), n_elts as usize); // get the number of elements in array a
    // assert_eq!(a.shape(), [n_elts as usize]); // get the shape of array a
    // assert_eq!(a.is_empty(), false); // check if the array has zero elements
}

// fn side_iter(board: Array::<u8,

// fn new(dim: (usize, usize)) -> Self {}

fn permute(board: &Board, pt_1: &Pt, pt_2: &Pt) -> Board {
    let mut output_board = board.clone();
    output_board[*pt_1] = board[*pt_2];
    output_board[*pt_2] = board[*pt_1];
    output_board
}

#[test]
fn test_permute() {
    let a1: Board = Array::from_iter([0, 1, 2, 3]).into_shape((2, 2)).unwrap();
    println!("{}", a1);
}
