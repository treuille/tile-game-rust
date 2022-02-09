use std::collections::HashSet;
use tile_game::big_set::{BigHashSet, HashedItemSet};

fn main() {
    println!("Benchmarking the new hashtable");

    let elts_to_insert = 20000000;
    if true {
        let mut set = BigHashSet::new(elts_to_insert, 0.6);
        for i in 0..elts_to_insert {
            set.insert(&i);
        }
    } else {
        let mut set = HashSet::with_capacity(elts_to_insert);
        for i in 0..elts_to_insert {
            set.insert(i);
        }
    }

    println!("Inserted {elts_to_insert} items.");
}
