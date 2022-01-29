use mktemp::Temp;
use std::collections::HashSet;

/// A queue may or pop elements in any particuar order.
pub trait UnorderedQueue<T> {
    /// Push an item onto the queue.
    fn enqueue(&mut self, item: T);

    /// Pop an item from the queue. Order is not garanteed.
    fn dequeue(&mut self) -> Option<T>;

    /// Returns the number of elements in this set.
    fn len(&self) -> usize;
}

impl<T> UnorderedQueue<T> for Vec<T> {
    /// Push an item onto the queue.
    fn enqueue(&mut self, item: T) {
        self.push(item);
    }

    /// Pop an item off the queue.
    fn dequeue(&mut self) -> Option<T> {
        self.pop()
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        self.len()
    }
}

/// A big queue which can use the disk to serialize extra items.
struct BigQueue<T> {
    /// The in-memory queue.
    queue: Vec<T>,

    /// The set of temporary files with which we can store extra queue items to disk.
    temp_files: Vec<Temp>,
}

impl<T> BigQueue<T> {
    /// Creates a new BigQueue. Panics if capacity < 2.
    fn new(capacity: usize) -> Self {
        assert!(capacity >= 2, "Capacity cannot be less than 2.");
        Self {
            queue: Vec::with_capacity(capacity),
            temp_files: Vec::new(),
        }
    }
}

// impl<T> UnorderedQueue<T> for BigQueue<T> {
//     /// Push an item onto the queue.
//     fn enqueue(&mut self, item: T) {
//         // Don't push a new element on the queue if we're at capacity.
//         if self.queue.len() == self.queue.capacity() {}
//         self.queue(item);
//     }

//     /// Pop an item from the queue. Order is not garanteed.
//     fn dequeue(&mut self) -> Option<T> {
//         self.dequeue()
//     }

//     /// Returns the number of elements in this set.
//     fn len(&self) -> usize {
//         self.len()
//     }
// }

// #[cfg(test)]
pub mod test {
    use super::*;

    pub fn scratchpad() {
        println!("Finished running test.");
        test_queue(&mut Vec::new());
    }

    fn test_queue(queue: &mut impl UnorderedQueue<usize>) {
        let n_elts = 10;
        for i in 0..n_elts {
            println!("enqueued {i}");
            assert_eq!(queue.len(), i);
            queue.enqueue(i);
        }
        assert_eq!(queue.len(), n_elts);

        while let Some(i) = queue.dequeue() {
            assert!(i < n_elts);
            println!("dequeued {i}");
        }
        assert_eq!(queue.len(), 0);
    }

    #[test]
    pub fn foo_test() {}
}
