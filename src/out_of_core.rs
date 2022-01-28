use memmap::MmapMut;
use mktemp::Temp;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

// temp_file is cleaned from the fs here
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
    phantom: PhantomData<T>,
}

impl<T: Hash> InMemoryHashedItemSet<T> {
    /// Constructor
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            hashes: HashSet::new(),
            phantom: PhantomData,
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

/// Stores a large group of elements by hash, even if they can't fit in main memory.
struct OutOfCoreHashedItemSet<T: Hash> {
    /// In memory store the hashes of the elements.
    hash_cache: HashSet<u64>,

    /// The maximum number of elements in the cache until transferred into the store.
    cache_size: usize,

    /// Memmap-backed store of hashes of the elements.
    hash_store: Option<BigU64Array>,

    /// 0-sized variable that makes this type behave as if it
    /// contained i tems of type T.
    phantom: PhantomData<T>,
}

impl<T: Hash> OutOfCoreHashedItemSet<T> {
    /// Contructor
    fn new(cache_size: usize) -> Self {
        Self {
            hash_cache: HashSet::with_capacity(cache_size),
            cache_size,
            hash_store: None,
            phantom: PhantomData,
        }
    }
}

impl<T: Hash> HashedItemSet<T> for OutOfCoreHashedItemSet<T> {
    ///  Returns true if the set contains this item.
    fn contains(&self, item: &T) -> bool {
        let item_hash = Self::hash(item);
        if self.hash_cache.contains(&item_hash) {
            return true;
        } else if let Some(hash_store) = &self.hash_store {
            return hash_store.binary_search(&item_hash).is_ok();
        }
        false
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &T) {
        let item_hash = Self::hash(item);
        self.hash_cache.insert(item_hash);
        if self.hash_cache.len() == self.cache_size {
            println!("Maximum cache size {} reached.", self.cache_size);
            let old_cache = mem::replace(
                &mut self.hash_cache,
                HashSet::with_capacity(self.cache_size),
            );
            let mut new_store: BigU64Array;
            match &self.hash_store {
                Some(old_store) => {
                    let old_store_len = old_store.len();
                    let new_store_len = old_store_len + self.cache_size;
                    new_store = BigU64Array::new(new_store_len).unwrap();
                    new_store[..old_store_len].copy_from_slice(old_store);
                    new_store[old_store_len..]
                        .copy_from_slice(&old_cache.into_iter().collect::<Vec<_>>());
                }
                None => {
                    new_store = BigU64Array::new(self.cache_size).unwrap();
                    new_store.copy_from_slice(&old_cache.into_iter().collect::<Vec<_>>());
                }
            }

            new_store.sort();
            println!("cache size: {}", self.hash_cache.len());
            println!("store size: {}", new_store.len());
            self.hash_store = Some(new_store);
        }
    }
}

/// A big array of u64s backed by a large memory-mapped temporary file.
struct BigU64Array {
    // The memory map itself.
    mmap: MmapMut,

    // The file that holds the memory map.
    file: File,

    // The filename of the memory map.
    filename: Temp,
}

impl BigU64Array {
    fn new(n_elts: usize) -> io::Result<Self> {
        let filename = Temp::new_file()?;
        let file = OpenOptions::new().read(true).write(true).open(&filename)?;
        file.set_len((n_elts * std::mem::size_of::<u64>()) as u64)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        let array = BigU64Array {
            filename,
            file,
            mmap,
        };
        assert_eq!(array.len(), n_elts);
        println!(
            "Created BigU64Array of size {} in {:?} ",
            n_elts,
            array.filename.to_path_buf()
        );
        Ok(array)
    }
}

impl Deref for BigU64Array {
    type Target = [u64];

    fn deref(&self) -> &Self::Target {
        let (align_left, u64_array, align_right) = unsafe { self.mmap.align_to::<u64>() };
        assert_eq!(align_left.len(), 0);
        assert_eq!(align_right.len(), 0);
        u64_array
    }
}
impl DerefMut for BigU64Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let (align_left, u64_array, align_right) = unsafe { self.mmap.align_to_mut::<u64>() };
        assert_eq!(align_left.len(), 0);
        assert_eq!(align_right.len(), 0);
        u64_array
    }
}

impl Drop for BigU64Array {
    fn drop(&mut self) {
        println!("Dropping array backed by {:?}", self.filename.to_path_buf());
    }
}

// #[cfg(test)]
pub mod test {
    use super::*;

    pub fn scratchpad() {
        let mut hash_items = OutOfCoreHashedItemSet::new(5);
        // use std::mem;
        // println!("Size of hashes: {}", mem::size_of_val(&hash_items.hashes));
        // println!("Size of phantom: {}", mem::size_of_val(&hash_items.phantom));

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
