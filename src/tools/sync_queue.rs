//-----------------------------------------------------------------------------//
// Rust Publish/Subscribe Pattern - Spare time development for fun             //
// (c) 2025-2026 Laurent Lardinois https://be.linkedin.com/in/laurentlardinois //
//                                                                             //
// https://github.com/type-one/PublishSubscribeRust                            //
//                                                                             //
// MIT License                                                                 //
//                                                                             //
// This software is provided 'as-is', without any express or implied           //
// warranty.In no event will the authors be held liable for any damages        //
// arising from the use of this software.                                      //
//                                                                             //
// Permission is granted to anyone to use this software for any purpose,       //
// including commercial applications, and to alter itand redistribute it       //
// freely, subject to the following restrictions :                             //
//                                                                             //
// 1. The origin of this software must not be misrepresented; you must not     //
// claim that you wrote the original software.If you use this software         //
// in a product, an acknowledgment in the product documentation would be       //
// appreciated but is not required.                                            //
// 2. Altered source versions must be plainly marked as such, and must not be  //
// misrepresented as being the original software.                              //
// 3. This notice may not be removed or altered from any source distribution.  //
//-----------------------------------------------------------------------------//

use std::collections::VecDeque;
use std::sync::RwLock;
/// Thread-safe queue implementation using standard Rust constructs.
#[derive(Debug)]
pub struct SyncQueue<T> {
    queue: RwLock<VecDeque<T>>,
}

/// Implementation of the SyncQueue methods.
impl<T> SyncQueue<T> {
    /// Creates a new SyncQueue.
    pub fn new() -> Self {
        SyncQueue {
            queue: RwLock::new(VecDeque::new()),
        }
    }

    /// Adds an item to the back of the queue.
    pub fn enqueue(&self, item: T) {
        let mut queue_guard = self.queue.write().unwrap();
        queue_guard.push_back(item);
    }

    /// Removes and returns an item from the front of the queue.
    pub fn dequeue(&self) -> Option<T> {
        let mut queue_guard = self.queue.write().unwrap();
        queue_guard.pop_front()
    }

    /// Alias for `dequeue`, provided for queue-compatible container usage.
    pub fn front_pop(&self) -> Option<T> {
        self.dequeue()
    }

    /// Adds items from an iterator to the back of the queue under a single lock.
    /// Returns the number of items inserted.
    pub fn push_range<I: IntoIterator<Item = T>>(&self, items: I) -> usize {
        let mut queue_guard = self.queue.write().unwrap();
        let mut inserted = 0;
        for item in items {
            queue_guard.push_back(item);
            inserted += 1;
        }
        inserted
    }

    /// Removes and returns up to `max_count` items from the front of the queue
    /// under a single lock.
    pub fn pop_range(&self, max_count: usize) -> Vec<T> {
        let mut queue_guard = self.queue.write().unwrap();
        let count = max_count.min(queue_guard.len());
        queue_guard.drain(..count).collect()
    }

    /// Checks if the queue is empty.
    pub fn is_empty(&self) -> bool {
        let queue_guard = self.queue.read().unwrap();
        queue_guard.is_empty()
    }

    /// Returns the size of the queue.
    pub fn size(&self) -> usize {
        let queue_guard = self.queue.read().unwrap();
        queue_guard.len()
    }

    /// Clears all items from the queue.
    pub fn clear(&self) {
        let mut queue_guard = self.queue.write().unwrap();
        queue_guard.clear();
    }

    /// Returns a reference to the front item of the queue.
    pub fn front(&self) -> Option<T>
    where
        T: Clone,
    {
        let queue_guard = self.queue.read().unwrap();
        queue_guard.front().cloned()
    }

    /// Returns a reference to the back item of the queue.
    pub fn back(&self) -> Option<T>
    where
        T: Clone,
    {
        let queue_guard = self.queue.read().unwrap();
        queue_guard.back().cloned()
    }
}

/// Default implementation for SyncQueue.
impl<T> Default for SyncQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for SyncQueue.
#[cfg(test)]
mod tests {
    use super::SyncQueue;

    // Basic test for enqueue and dequeue operations.
    #[test]
    fn test_enqueue_dequeue() {
        let queue = SyncQueue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), None);
    }

    // Basic test for is_empty method.
    #[test]
    fn test_is_empty() {
        let queue = SyncQueue::new();
        assert!(queue.is_empty());
        queue.enqueue(1);
        assert!(!queue.is_empty());
        queue.dequeue();
        assert!(queue.is_empty());
    }

    // Basic test for size method.
    #[test]
    fn test_size() {
        let queue = SyncQueue::new();
        assert_eq!(queue.size(), 0);
        queue.enqueue(1);
        assert_eq!(queue.size(), 1);
        queue.enqueue(2);
        assert_eq!(queue.size(), 2);
        queue.dequeue();
        assert_eq!(queue.size(), 1);
    }

    // Basic test for clear method.
    #[test]
    fn test_clear() {
        let queue = SyncQueue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.clear();
        assert!(queue.is_empty());
    }

    // Basic test for front and back methods.
    #[test]
    fn test_front_back() {
        let queue = SyncQueue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        assert_eq!(queue.front(), Some(1));
        assert_eq!(queue.back(), Some(2));
    }

    // test for front_pop alias
    #[test]
    fn test_front_pop() {
        let queue = SyncQueue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        assert_eq!(queue.front_pop(), Some(1));
        assert_eq!(queue.front_pop(), Some(2));
        assert_eq!(queue.front_pop(), None);
    }

    // test for push_range and pop_range batch operations
    #[test]
    fn test_push_range_and_pop_range() {
        let queue = SyncQueue::new();
        let inserted = queue.push_range(vec![1, 2, 3]);
        assert_eq!(inserted, 3);
        assert_eq!(queue.pop_range(2), vec![1, 2]);
        assert_eq!(queue.pop_range(5), vec![3]);
    }

    // Additional test with two threads
    use std::sync::Arc;
    use std::thread;
    #[test]
    fn test_concurrent_access() {
        let queue = Arc::new(SyncQueue::new());
        let queue_for_producer = queue.clone();
        let queue_for_consumer = queue.clone();

        let producer = thread::spawn(move || {
            for i in 0..100 {
                queue_for_producer.enqueue(i);
            }
        });

        let consumer = thread::spawn(move || {
            let mut count = 0;
            while count < 100 {
                if queue_for_consumer.dequeue().is_some() {
                    count += 1;
                }
            }
        });

        producer.join().unwrap();
        consumer.join().unwrap();

        assert!(queue.is_empty());
    }

    // test for Default trait
    #[test]
    fn test_default() {
        let queue: SyncQueue<i32> = SyncQueue::default();
        assert_eq!(queue.size(), 0);
    }
}
