use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A set of integers.
trait HashedItemSet {
    type Item: Hash;

    /// Returns true if the set contains this integer.
    fn contains(&self, item: &Self::Item) -> bool;

    /// Inserts a usize into this struct.
    fn insert(&mut self, item: &Self::Item);

    /// Calculates the hash for an item.
    fn hash(item: &Self::Item) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
}

/// A set of integers held in memory.
#[derive(Debug)]
struct InMemoryHashedItemSet<T: Hash> {
    hashes: HashSet<u64>,
    _phantom: PhantomData<T>,
}

impl<T: Hash> InMemoryHashedItemSet<T> {
    /// Contructor
    pub fn new() -> Self {
        Self {
            hashes: HashSet::new(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Hash> HashedItemSet for InMemoryHashedItemSet<T>
where
    T: Hash,
{
    type Item = T;

    /// Returns true if the set contains this item.
    fn contains(&self, item: &Self::Item) -> bool {
        self.hashes.contains(&Self::hash(item))
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &Self::Item) {
        self.hashes.insert(Self::hash(item));
    }
}

struct OutOfCoreHashItemSet<T> {
    _hashes: HashSet<T>,
}

// #[cfg(test)]
pub mod test {
    use super::*;

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
