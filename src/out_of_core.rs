use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A set of integers.
trait HashedItemSet<T: Hash> {
    /// Returns true if the set contains this integer.
    fn contains(&self, item: &T) -> bool;

    /// Inserts a usize into this struct.
    fn insert(&mut self, item: &T);

    /// Calculates the hash for an item.
    fn hash(item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
}

/// A set of integers held in memory.
#[derive(Debug)]
struct InMemoryHashedItemSet<T: Hash> {
    // Where we store the hashes of the elements.
    hashes: HashSet<u64>,

    // 0-sized variable that makes this type behave as if it
    // contained items of type T.
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

impl<T: Hash> HashedItemSet<T> for InMemoryHashedItemSet<T> {
    /// Returns true if the set contains this item.
    fn contains(&self, item: &T) -> bool {
        self.hashes.contains(&Self::hash(item))
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &T) {
        self.hashes.insert(Self::hash(item));
    }
}

// struct OutOfCoreHashItemSet<T> {
//     _hashes: HashSet<T>,
// }

// #[cfg(test)]
pub mod test {
    use super::*;

    pub fn scratchpad() {
        let mut hash_items = InMemoryHashedItemSet::new();
        use std::mem;
        println!("Size of hashes: {}", mem::size_of_val(&hash_items.hashes));
        println!(
            "Size of _phantom: {}",
            mem::size_of_val(&hash_items._phantom)
        );

        for c in ('a'..='z').step_by(2) {
            hash_items.insert(&c);
        }

        for (i, c) in ('a'..='z').enumerate() {
            let contains_c = hash_items.contains(&c);
            println!("{i} : '{c}' in hash_items : {contains_c}");
            match i % 2 {
                0 => assert!(hash_items.contains(&c)),
                _ => assert!(!hash_items.contains(&c)),
            }
        }
    }

    #[test]
    pub fn test_in_memory_hashed_item_set() {
        let mut hash_items = InMemoryHashedItemSet::new();

        for c in ('a'..='z').step_by(2) {
            hash_items.insert(&c);
        }

        for (i, c) in ('a'..='z').enumerate() {
            match i % 2 {
                0 => assert!(hash_items.contains(&c)),
                _ => assert!(!hash_items.contains(&c)),
            }
        }
    }
}
