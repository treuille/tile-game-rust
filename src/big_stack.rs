use bincode;
use mktemp::Temp;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fs::File;

// A stack may or pop elements in any particuar order.
pub trait Stack<T> {
    /// Push an item onto the stack.
    fn push(&mut self, item: T);

    /// Pop an item from the stack. Order is not garanteed.
    fn pop(&mut self) -> Option<T>;

    /// Returns the number of elements in this set.
    fn len(&self) -> usize;
}

impl<T> Stack<T> for Vec<T> {
    /// Push an item onto the stack.
    fn push(&mut self, item: T) {
        self.push(item);
    }

    /// Pop an item off the stack.
    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        self.len()
    }
}

/// A big stack which can use the disk to serialize extra items.
pub struct BigStack<T>
where
    T: Debug + Serialize + for<'de> Deserialize<'de>,
{
    /// The in-memory stack.
    stack: VecDeque<T>,

    /// The capacity of the in-memory stack.
    capacity: usize,

    /// The set of temporary files with which we can store extra stack items to disk.
    temp_filenames: Vec<Temp>,
}

impl<T> BigStack<T>
where
    T: Debug + Serialize + for<'de> Deserialize<'de>,
{
    /// Creates a new BigStack. Panics if capacity < 2.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 2, "Capacity cannot be less than 2.");
        BigStack {
            stack: VecDeque::with_capacity(capacity),
            capacity,
            temp_filenames: Vec::new(),
        }
    }

    /// Returns half the capacity of the in=memory stack.
    fn half_capacity(&self) -> usize {
        self.capacity / 2
    }
}

impl<T> Stack<T> for BigStack<T>
where
    T: Debug + Serialize + for<'de> Deserialize<'de>,
{
    /// Push an item onto the stack.
    fn push(&mut self, item: T) {
        if self.stack.len() == self.capacity {
            // If we're at capacity, move this stack into storage.
            let temp_filename = Temp::new_file().unwrap();
            let temp_file = File::create(&temp_filename).unwrap();

            println!("Creating file: {:?}", temp_filename.to_path_buf());

            for item in self.stack.drain(..self.half_capacity()) {
                bincode::serialize_into(&temp_file, &item).unwrap();
            }

            self.temp_filenames.push(temp_filename);
        }
        self.stack.push_back(item);
    }

    /// Pop an item from the stack. Order is not garanteed.
    fn pop(&mut self) -> Option<T> {
        self.stack.pop_back().or_else(|| {
            if let Some(temp_filename) = self.temp_filenames.pop() {
                println!("Opening {:?}", temp_filename.to_path_buf());
                let temp_file = File::open(temp_filename).unwrap();
                while let Ok(item) = bincode::deserialize_from(&temp_file) {
                    self.stack.push_back(item);
                }
                self.stack.pop_back()
            } else {
                None
            }
        })
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        self.stack.len() + self.temp_filenames.len() * self.half_capacity()
    }
}

// #[cfg(test)]
pub mod test {
    use super::*;

    pub fn scratchpad() {
        println!("Finished running test.");
        test_vec_stack();
        test_big_stack();
    }

    fn test_vec_stack() {
        test_stack(&mut Vec::new());
    }

    fn test_big_stack() {
        test_stack(&mut BigStack::new(5));
    }

    fn test_stack(stack: &mut impl Stack<usize>) {
        let n_elts = 10;
        for i in 0..n_elts {
            println!("Pushed {i}");
            assert_eq!(stack.len(), i, "Stack length should be {i}");
            stack.push(i);
        }
        assert_eq!(stack.len(), n_elts);

        for i in (0..n_elts).rev() {
            assert_eq!(stack.pop(), Some(i));
            assert_eq!(stack.len(), i);
            println!("Popped {i}");
        }
    }

    #[test]
    pub fn foo_test() {}
}
