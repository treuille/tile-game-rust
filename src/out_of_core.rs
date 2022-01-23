use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// A set of integers.
trait HashedItemSet {
    type Item: Hash;

    /// Returns true if the set contains this integer.
    fn contains(&self, item: &Self::Item) -> bool;

    /// Inserts a usize into this struct.
    fn insert(&mut self, item: &Self::Item);
}

/// A set of integers held in memory.
#[derive(Debug)]
struct InMemoryHashedItemSet<T: Hash> {
    hashes: HashSet<u64>,
    phantom: PhantomData<T>,
}

impl<T: Hash> InMemoryHashedItemSet<T> {
    /// Contructor
    pub fn new() -> Self {
        Self {
            hashes: HashSet::new(),
            phantom: PhantomData,
        }
    }
}

impl<T: Hash> HashedItemSet for InMemoryHashedItemSet<T>
where
    T: Hash,
{
    type Item = T;

    /// Returns true if the set contains this integer.
    fn contains(&self, item: &Self::Item) -> bool {
        let the_hash = calculate_hash(item);
        self.hashes.contains(&the_hash)
    }

    /// Inserts a usize into this struct.
    fn insert(&mut self, item: &Self::Item) {
        let the_hash: u64 = calculate_hash(item);
        self.hashes.insert(the_hash);
    }
}

// #[cfg(test)]
pub mod test {
    use super::*;

    // #[test]
    pub fn scratchpad() {
        println!("Testing scratchpad.");
        let mut hash_items = InMemoryHashedItemSet::new();
        hash_items.insert(&'a');
        hash_items.insert(&'b');
        hash_items.insert(&'z');
        println!("hash_items: {hash_items:?}");

        // for i in TEST_STR.chars().step_by(2) {
        //     hash_items.insert(&i);
        // }
        for i in "abcd".chars() {
            let contains_i = hash_items.contains(&i);
            println!("{i} in hash_items : {contains_i}");
        }
        assert!(hash_items.contains(&'a'));
        assert!(hash_items.contains(&'b'));
        assert!(!hash_items.contains(&'c'));
        assert!(!hash_items.contains(&'d'));
    }

    #[test]
    pub fn test_in_memory_hashed_item_set() {
        let mut hash_items = InMemoryHashedItemSet::new();

        hash_items.insert(&'a');
        hash_items.insert(&'b');
        hash_items.insert(&'z');

        assert!(hash_items.contains(&'a'), "Should contain a");
        assert!(hash_items.contains(&'b'), "Should contain b");
        assert!(!hash_items.contains(&'c'), "Shouldn't contain c");
        assert!(!hash_items.contains(&'d'), "Shouldn't contain d");
    }
}
