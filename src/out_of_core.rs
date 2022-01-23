use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub fn say_hello() {
    println!("Hello, say_hello.");
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// A set of integers.
pub trait IntSet {
    /// Returns true if the set contains this integer.
    fn contains(&self, item: usize) -> bool;

    /// Inserts a usize into this struct.
    fn insert(&mut self, item: usize);
}

/// A set of integers held in memory.
pub struct InMemoryIntSet(HashSet<usize>);

impl InMemoryIntSet {
    /// Contructor
    pub fn new() -> Self {
        Self(HashSet::<usize>::new())
    }
}

impl IntSet for InMemoryIntSet {
    /// Returns true if the set contains this integer.
    fn contains(&self, item: usize) -> bool {
        false
    }

    /// Inserts a usize into this struct.
    fn insert(&mut self, item: usize) {}
}

// #[cfg(test)]
pub mod test {
    use super::*;

    // #[test]
    pub fn scratchpad() {
        println!("Testing scratchpad.");
        for i in 0..40 {
            let hash_i = calculate_hash(&i);
            println!("{i} -> {hash_i}");
        }
        println!("Finished testing scratchpad!");
    }
}
