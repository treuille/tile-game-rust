use bloomfilter::Bloom;
use memmap::MmapMut;
use mktemp::Temp;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::{cmp, io};

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

/// Stores a large group of elements by hash, even if they can't fit in main memory.
pub struct BigSet<T: Hash> {
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

impl<T: Hash> BigSet<T> {
    /// Contructor
    pub fn new(cache_size: usize) -> Self {
        Self {
            hash_cache: HashSet::with_capacity(cache_size),
            cache_size,
            hash_store: None,
            phantom: PhantomData,
        }
    }
}

impl<T: Hash> HashedItemSet<T> for BigSet<T> {
    ///  Returns true if the set contains this item.
    fn contains(&self, item: &T) -> bool {
        let item_hash = hash(item);
        if self.hash_cache.contains(&item_hash) {
            return true;
        } else if let Some(hash_store) = &self.hash_store {
            return hash_store.binary_search(&item_hash).is_ok();
        }
        false
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &T) {
        let item_hash = hash(item);
        self.hash_cache.insert(item_hash);
        if self.hash_cache.len() == self.cache_size {
            println!("Maximum cache size {} reached.", self.cache_size);
            let old_store_len = self.hash_store.as_ref().map_or(0, |s| s.len());
            let mut new_store = BigU64Array::new(old_store_len + self.cache_size).unwrap();
            if let Some(old_store) = &self.hash_store {
                new_store[..old_store_len].copy_from_slice(&old_store);
            }
            for (i, item_hash) in self.hash_cache.drain().enumerate() {
                new_store[old_store_len + i] = item_hash;
            }
            new_store.sort();
            println!("cache size: {}", self.hash_cache.len());
            println!("store size: {}", new_store.len());
            self.hash_store = Some(new_store);
        }
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        match &self.hash_store {
            Some(hash_store) => self.hash_cache.len() + hash_store.len(),
            None => self.hash_cache.len(),
        }
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

/// Like BigSet, but puts all the items behind a bloom filter for efficiency.
pub struct BloomSet<T: Hash> {
    /// The bloom filter to avoid some searches.
    bloom_filter: Bloom<T>,

    /// The mmmap-backed store of hashed items.
    big_set: BigSet<T>,
}

impl<T: Hash> BloomSet<T> {
    /// Constructs a new BloomSet
    ///
    /// # Arguments
    ///
    /// * `cache_size` - The maximum number of hashed items to store in memory.
    /// * `items_count` - The expected number total items stored.
    /// * `fp_p` - The desired number of false positives in the bloom filter.  
    pub fn new(cache_size: usize, items_count: usize, fp_p: f64) -> Self {
        // println!(
        //     "Bitmap size: {}",
        //     Bloom::<T>::compute_bitmap_size(items_count, fp_p)
        // );
        Self {
            bloom_filter: Bloom::new_for_fp_rate(items_count, fp_p),
            big_set: BigSet::new(cache_size),
        }
    }
}

impl<T: Hash> HashedItemSet<T> for BloomSet<T> {
    ///  Returns true if the set contains this item.
    fn contains(&self, item: &T) -> bool {
        if self.bloom_filter.check(item) {
            // There could be a false positive so we have to check explicitly.
            self.big_set.contains(item)
        } else {
            false
        }
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &T) {
        self.bloom_filter.set(item);
        self.big_set.insert(item)
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        self.big_set.len()
    }
}

/// Like BigSet, but puts all the items behind a bloom filter for efficiency.
pub struct PartitionSet<T: Hash> {
    /// The number of partions
    n_partitions: usize,

    /// The bloom filter to avoid some searches.
    partitions: Vec<BloomSet<T>>,
}

impl<T: Hash> PartitionSet<T> {
    /// Constructs a new PartitionSet
    ///
    /// # Arguments
    ///
    /// * `cache_size` - The maximum number of hashed items to store in memory.
    /// * `items_count` - The expected number total items stored.
    /// * `fp_p` - The desired number of false positives in the bloom filter.  
    /// * `n_partitions` - The number of memory mapped partitions for this set.
    pub fn new(cache_size: usize, items_count: usize, fp_p: f64, n_partitions: usize) -> Self {
        assert!(n_partitions > 0, "Must have at least one partition.");
        let items_per_partition = cmp::max(items_count / n_partitions, 1);
        let partitions = (0..n_partitions)
            .map(|_| BloomSet::new(cache_size, items_per_partition, fp_p))
            .collect::<Vec<_>>();
        Self {
            n_partitions,
            partitions,
        }
    }
}

impl<T: Hash> HashedItemSet<T> for PartitionSet<T> {
    ///  Returns true if the set contains this item.
    fn contains(&self, item: &T) -> bool {
        let partition = (hash(item) as usize) % self.n_partitions;
        self.partitions[partition].contains(item)
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &T) {
        let partition = (hash(item) as usize) % self.n_partitions;
        self.partitions[partition].insert(item)
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        self.partitions.iter().map(|p| p.len()).sum()
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

    #[test]
    pub fn test_big_set() {
        test_hashed_item_set(&mut BigSet::new(3));
    }

    #[test]
    pub fn test_bloom_set() {
        test_hashed_item_set(&mut BloomSet::new(3, 26, 0.5));
    }

    #[test]
    pub fn test_partition_set() {
        test_hashed_item_set(&mut PartitionSet::new(3, 26, 0.5, 2));
    }
}
