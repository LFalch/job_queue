use std::collections::{BinaryHeap, VecDeque};

pub trait Queue<T> {
    fn enqueue(&mut self, item: T);
    fn dequeue(&mut self) -> Option<T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Queue<T> for VecDeque<T> {
    fn enqueue(&mut self, item: T) {
        self.push_back(item);
    }
    fn dequeue(&mut self) -> Option<T> {
        self.pop_front()
    }
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T: Ord> Queue<T> for BinaryHeap<T> {
    fn enqueue(&mut self, item: T) {
        self.push(item);
    }
    fn dequeue(&mut self) -> Option<T> {
        self.pop()
    }
    fn len(&self) -> usize {
        self.len()
    }
}
