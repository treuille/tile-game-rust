use std::collections::HashSet;

pub fn say_hello() {
    println!("Hello, say_hello.");
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

impl IntSet for InMemoryIntSet {
    /// Returns true if the set contains this integer.
    fn contains(&self, item: usize) -> bool {
        false
    }

    /// Inserts a usize into this struct.
    fn insert(&mut self, item: usize) {}
}
