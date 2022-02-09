use memmap::MmapMut;
use mktemp::Temp;
use primal_sieve::Primes;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::{io, mem};

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

pub struct BigHashSet<T: Hash> {
    /// The memmapped hashes
    hashes: BigU64Array,

    /// How many hashes are stored to disk.
    stored_hashes: usize,

    /// Maximum load
    max_load: f64,

    /// 0-sized variable that makes this type behave as if it
    /// contained items of type T.
    phantom: PhantomData<T>,
}

impl<T> BigHashSet<T>
where
    T: Hash,
{
    pub fn new(n_elts: usize, max_load: f64) -> Self {
        let capacity = ((n_elts as f64) / max_load) as usize;
        let prime_capacity = Primes::all().find(|p| p >= &capacity).unwrap();
        BigHashSet {
            hashes: BigU64Array::new_zeroed(prime_capacity).unwrap(),
            stored_hashes: 0,
            max_load,
            phantom: PhantomData,
        }
    }

    /// Inserts a item into this set.
    fn insert_hash(&mut self, hash: u64) {
        let max_elts = ((self.hashes.len() as f64) * self.max_load) as usize;
        if self.stored_hashes + 1 > max_elts {
            // We need to increase the length of the array and rehash everything.
            let mut new_self = Self::new(self.hashes.len() * 2, self.max_load);
            self.hashes
                .iter()
                .filter(|&h| *h != 0)
                .for_each(|h| new_self.insert_hash(*h));
            mem::swap(self, &mut new_self);
        }

        // Find an insert the item in the first empty index we find.
        let empty_index = self
            .probe(hash)
            .find(|i| self.hashes[*i] == 0)
            .expect("Couldn't find an empty location.");
        self.hashes[empty_index] = hash;
        self.stored_hashes += 1;
    }

    // /// Runs an alternating quadratic probe through the hashtable indices
    // fn probe(&self, hash: u64) -> impl Iterator<Item = usize> + '_ {
    //     let hash = usize::try_from(hash).unwrap();
    //     let hash_len = self.hashes.len();
    //     (0..hash_len).map(move |i| {
    //         (match i % 2 {
    //             0 => (hash + i * i / 4),
    //             _ => usize::checked_sub(hash + hash_len, i * (i + 1) / 4).unwrap(),
    //         }) % hash_len
    //     })
    // }

    // /// Runs a linear probe through the hashtable indices
    // fn probe(&self, hash: u64) -> impl Iterator<Item = usize> + '_ {
    //     let hash = usize::try_from(hash).unwrap();
    //     let hash_len = self.hashes.len();
    //     (0..hash_len).map(move |i| (hash + i) % hash_len)
    // }

    /// Runs an alternating quadratic probe through the hashtable indices
    fn probe(&self, hash: u64) -> impl Iterator<Item = usize> + '_ {
        let hash = usize::try_from(hash).unwrap();
        let hash_len = self.hashes.len();
        (0..hash_len).map(move |i| match i % 2 {
            0 => (hash + i / 2),
            _ => usize::checked_sub(hash + hash_len, (i + 2) / 2).unwrap(),
        } % hash_len)
    }

    // /// Runs a quadratic probe through the hashtable indices
    // fn probe(&self, hash: u64) -> impl Iterator<Item = usize> + '_ {
    //     let hash_len = self.hashes.len();
    //     (0..hash_len).map(move |i| ((hash as usize) + i * i) % hash_len)
    // }
}

impl<T> HashedItemSet<T> for BigHashSet<T>
where
    T: Hash,
{
    /// Returns true if the set contains this item (up to hash collisions).
    fn contains(&self, item: &T) -> bool {
        let hash: u64 = hash(item);
        self.probe(hash)
            .find_map(|i| {
                if self.hashes[i] == hash {
                    Some(true)
                } else if self.hashes[i] == 0 {
                    Some(false)
                } else {
                    None
                }
            })
            .expect("Exhausted probe without finding the item.")
    }

    /// Inserts a item into this set.
    fn insert(&mut self, item: &T) {
        self.insert_hash(hash(item));
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        self.stored_hashes
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

    fn new_zeroed(n_elts: usize) -> io::Result<Self> {
        let mut array = Self::new(n_elts)?;
        for element in array.iter_mut() {
            *element = 0;
        }
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
    pub fn test_big_hash_set() {
        test_hashed_item_set(&mut BigHashSet::new(25, 0.5));
    }
}
