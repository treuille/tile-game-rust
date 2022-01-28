use memmap::MmapMut;
use mktemp::Temp;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Command;

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

    // The memory map we use to back this data structure.
    mmap: BigU64Array,

    // 0-sized variable that makes this type behave as if it
    // contained i tems of type T.
    phantom: PhantomData<T>,
}

impl<T: Hash> OutOfCoreHashedItemSet<T> {
    /// Contructor
    fn new(max_elts: usize) -> io::Result<Self> {
        // run_command(format!("rm -rfv {}/*", Self::CACHE_PATH));
        // run_command(format!("mkdir -p {}", Self::CACHE_PATH));

        let mmap = BigU64Array::new(100)?;
        println!("The deref has {} elts", mmap.len());

        Ok(Self {
            hashes: HashSet::new(),
            mmap: mmap,
            phantom: PhantomData,
        })
    }

    // // Where we store the data.
    // const CACHE_PATH: &'static str = "cache/item_set";
}

impl<T: Hash> HashedItemSet<T> for OutOfCoreHashedItemSet<T> {
    ///  jReturns true if the set contains this item.
    fn contains(&self, _item: &T) -> bool {
        false
    }

    /// Inserts an item into the set
    fn insert(&mut self, item: &T) {
        let mut path_buf = PathBuf::new();
        {
            let temp_file = Temp::new_file().unwrap();
            path_buf.push(&temp_file);
            println!("{:?} exists: {}", *temp_file, temp_file.exists());
            println!("{:?} exists: {}", path_buf, path_buf.exists());
            println!("{:?} exists: {}", path_buf, path_buf.exists());
        }
        println!("{:?} exists: {}", path_buf, path_buf.exists());
        panic!("All done!");
        // println!("{:?} exists: {}", path_buf, Path::exists(path_buf));
        // self.hashes.insert(Self::hash(item));

        // // TODO: This is were I need to start implementing the memory map.
        // if self.hashes.len() >= self.max_elts {
        //     println!("There are now {} elements!", self.hashes.len());
        //     // Create the file
        //     let mut path = PathBuf::from_str(Self::CACHE_PATH).unwrap();
        //     path.push(format!("{:05}.dat", self.maps.len()));
        //     let file = OpenOptions::new()
        //         .read(true)
        //         .write(true)
        //         .create(true)
        //         .open(&path)
        //         .unwrap();
        //     println!(
        //         "Creating a file of length {}",
        //         self.max_elts * std::mem::size_of::<u64>()
        //     );
        //     file.set_len((self.max_elts * std::mem::size_of::<u64>()) as u64)
        //         .unwrap();
        //     println!("Creating cache file: {path:?}");
        //     panic!("adding the memory map");
        // }
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
            n_elts, array.filename.to_path_buf()
        );
        Ok(array)
    }
}

impl Deref for BigU64Array {
    type Target = [u64];

    fn deref(&self) -> &Self::Target {
        let (align_left, u64_array, align_right) = unsafe { self.mmap.align_to::<u64>() };
        println!("align_left.len(): {}", align_left.len());
        println!("u64_array.len(): {}", u64_array.len());
        println!("align_right.len(): {}", align_right.len());
        assert_eq!(align_left.len(), 0);
        assert_eq!(align_right.len(), 0);
        u64_array
    }
}

// #[cfg(test)]
pub mod test {
    use super::*;

    pub fn scratchpad() {
        let mut hash_items = OutOfCoreHashedItemSet::new(10).unwrap();
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
