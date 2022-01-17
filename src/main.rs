use ndarray::{Array, Array2};

#[derive(Debug, PartialEq)]
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
}

fn main() {
    // This the width and height of the tile game we're using.
    let (w, h) = (3, 3);
    let n_elts = (w * h) as u8;
    println!("Board size: {}x{}", w, h);

    let a = Board::new(0..n_elts, &(w, h));

    println!("{:?}", a);
    println!("It worked!");

    // assert_eq!(a.ndim(), 1); // get the number of dimensions of array a
    // assert_eq!(a.len(), n_elts as usize); // get the number of elements in array a
    // assert_eq!(a.shape(), [n_elts as usize]); // get the shape of array a
    // assert_eq!(a.is_empty(), false); // check if the array has zero elements
}

// fn side_iter(board: Array::<u8,

// fn new(dim: (usize, usize)) -> Self {}

#[test]
fn test_permute() {
    let a1 = Board::new([0, 1, 2, 3], &(2, 2));
    let a2 = Board::new([1, 0, 2, 3], &(2, 2));
    let a3 = a1.permute(&[0, 0], &[0, 1]);
    assert_eq!(a2, a3);
}
