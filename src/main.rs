// use ndarray::prelude::*;
use ndarray::Array;

fn main() {
    // This the width and height of the tile game we're using.
    let (w, h) = (3, 3);
    let n_elts = w * h;
    println!("Board size: {}x{}", w, h);

    let a = Array::<u8, _>::from_iter(0..n_elts);
    assert_eq!(a.ndim(), 1); // get the number of dimensions of array a
    assert_eq!(a.len(), n_elts as usize); // get the number of elements in array a
    assert_eq!(a.shape(), [n_elts as usize]); // get the shape of array a
    assert_eq!(a.is_empty(), false); // check if the array has zero elements

    println!("{:?}", a);
}
