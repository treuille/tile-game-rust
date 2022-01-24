use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::process::Command;

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
    /// Constructor
    #[allow(dead_code)]
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

/// Runs a command on the command line.
fn run_command<S: AsRef<OsStr>>(cmd: S) {
    Command::new("sh")
        .arg::<&str>("-c")
        .arg(cmd)
        .spawn()
        .unwrap();
}

/// Stores a large group of elements by hash, even if they can't fit in main memory.
struct OutOfCoreHashedItemSet<T: Hash> {
    // Where we store the hashes of the elements.
    hashes: HashSet<u64>,

    // Maximum number of elements in memory
    max_elts: usize,

    // 0-sized variable that makes this type behave as if it
    // contained items of type T.
    _phantom: PhantomData<T>,
}

impl<T: Hash> OutOfCoreHashedItemSet<T> {
    /// Contructor
    fn new(max_elts: usize) -> Self {
        run_command(format!("rm -rfv {}/*", Self::CACHE_PATH));
        run_command(format!("mkdir -p {}", Self::CACHE_PATH));

        Self {
            hashes: HashSet::new(),
            max_elts,
            _phantom: PhantomData,
        }
    }

    // Where we store the data
    const CACHE_PATH: &'static str = "cache/item_set";
}

impl<T: Hash> HashedItemSet<T> for OutOfCoreHashedItemSet<T> {
    /// Returns true if the set contains this item.
    fn contains(&self, item: &T) -> bool {
        false
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &T) {
        self.hashes.insert(Self::hash(item));

        // TODO: This is were I need to start implementing the memory map.
        if self.hashes.len() >= self.max_elts {
            panic!("There are now {} elements!", self.hashes.len());
        }
    }
}

// #[cfg(test)]
pub mod test {
    use super::*;

    pub fn scratchpad() {
        let mut hash_items = OutOfCoreHashedItemSet::new(10);
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
