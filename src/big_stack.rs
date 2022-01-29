/// A generic stack type.
pub trait Stack<T> {
    /// Returns true if the set contains this item (up to hash collisions).
    fn push(&mut self, item: T);

    /// Inserts a item into this set.
    fn pop(&mut self) -> Option<T>;

    /// Returns the number of elements in this set.
    fn len(&self) -> usize;
}

impl<T> Stack<T> for Vec<T> {
    /// Returns true if the set contains this item (up to hash collisions).
    fn push(&mut self, item: T) {
        self.push(item);
    }

    /// Inserts a item into this set.
    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    /// Returns the number of elements in this set.
    fn len(&self) -> usize {
        self.len()
    }
}

// #[cfg(test)]
pub mod test {
    use super::*;

    pub fn scratchpad() {
        println!("Finished running test.");
        test_stack(&mut Vec::new());
    }

    fn test_stack(stack: &mut impl Stack<usize>) {
        for i in 0..10 {
            println!("pushing {i}");
            stack.push(i);
        }

        for i in (0..10).rev() {
            assert_eq!(stack.pop(), Some(i));
            println!("popped {i}");
        }
    }

    #[test]
    pub fn foo_test() {}
}
