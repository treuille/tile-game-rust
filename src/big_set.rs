use memmap::MmapMut;
use mktemp::Temp;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A set of items, stored by their hash values.
pub trait HashedItemSet<T: Hash> {
    /// Returns true if the set contains this item (up to hash collisions).
    fn contains(&self, item: &T) -> bool;

    /// Inserts a item into this set.
    fn insert(&mut self, item: &T);

    /// Returns the number of elements in this set.
    fn len(&self) -> usize;
}

/// Calculates the hash for an item.
fn hash(item: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}

/// A set of items, held in hashed form in memory. Used just for testing purposes.
#[derive(Debug)]
pub struct LittleSet<T: Hash> {
    // Where we store the hashes of the elements.
    hashes: HashSet<u64>,

    // 0-sized variable that makes this type behave as if it
    // contained items of type T.
    phantom: PhantomData<T>,
}

impl<T: Hash> LittleSet<T> {
    /// Constructor
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            hashes: HashSet::new(),
            phantom: PhantomData,
        }
    }
}

impl<T: Hash> HashedItemSet<T> for LittleSet<T> {
    /// Returns true if the set contains this item.
    fn contains(&self, item: &T) -> bool {
        self.hashes.contains(&hash(item))
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &T) {
        self.hashes.insert(hash(item));
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        self.hashes.len()
    }
}

/// A big array of u64s backed by a large memory-mapped temporary file.
struct BigU64Array {
    /// The memory map itself.
    mmap: MmapMut,

    /// The filename of the memory map.
    filename: Temp,
}

impl BigU64Array {
    fn new(n_elts: usize) -> io::Result<Self> {
        let filename = Temp::new_file()?;
        let file = OpenOptions::new().read(true).write(true).open(&filename)?;
        file.set_len((n_elts * std::mem::size_of::<u64>()) as u64)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        let array = BigU64Array { filename, mmap };
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
#[cfg(test)]
pub mod test {
    use super::*;

    fn test_hashed_item_set(items: &mut impl HashedItemSet<char>) {
        for (i, c) in ('a'..='z').step_by(2).enumerate() {
            assert_eq!(items.len(), i);
            items.insert(&c);
        }

        for (i, c) in ('a'..='z').enumerate() {
            match i % 2 {
                0 => assert!(items.contains(&c)),
                _ => assert!(!items.contains(&c)),
            }
        }
    }

    #[test]
    pub fn test_little_set() {
        test_hashed_item_set(&mut LittleSet::new());
    }
}
