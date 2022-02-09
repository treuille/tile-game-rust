// use rayon::prelude::*;
// use std::thread;
// use std::time::Duration;

use tile_game::big_set::{BigHashSet, HashedItemSet};

fn main() {
    println!("Benchmarking the new hashtable");

    let elts_to_insert = 100000000;
    let mut set = BigHashSet::new(elts_to_insert);
    for i in 0..elts_to_insert {
        set.insert(&i);
    }

    println!("Inserted {elts_to_insert} items.");
}
