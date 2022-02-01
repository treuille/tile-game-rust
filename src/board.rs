use ndarray::{Array, Array2};
use serde::{Deserialize, Serialize};

/// A point on the board.
pub type Pt = [usize; 2];

/// A board reprenting a `w` by `h` grid of sliding tiles.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board(Array2<u8>);

impl Board {
    pub fn new<I>(iter: I, shape: &(usize, usize)) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        Board(Array::from_iter(iter).into_shape(*shape).unwrap())
    }

    pub fn slide_iter(self: &Self) -> impl Iterator<Item = Self> + '_ {
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

    fn permute(self: &Self, pt_1: &Pt, pt_2: &Pt) -> Self {
        let board = &self.0;
        let mut output_board = board.clone();
        output_board[*pt_1] = board[*pt_2];
        output_board[*pt_2] = board[*pt_1];
        Board(output_board)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_board_permute() {
        let a1 = Board::new([0, 1, 2, 3], &(2, 2));
        let a2 = Board::new([1, 0, 2, 3], &(2, 2));
        let a3 = a1.permute(&[0, 0], &[0, 1]);
        assert_eq!(a2, a3);
    }
}
