//-----------------------------------------------------------------------------//
// Rust Publish/Subscribe Pattern - Spare time development for fun             //
// (c) 2025 Laurent Lardinois https://be.linkedin.com/in/laurentlardinois      //
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

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::RwLock;

/// Thread-safe priority queue implementation on top of `BinaryHeap`.
/// Default behavior is a min-heap (lowest value popped first), matching the
/// C++ `sync_priority_queue` default of `std::greater<T>`.
#[derive(Debug)]
pub struct SyncPriorityQueue<T> {
    heap: RwLock<BinaryHeap<Reverse<T>>>,
}

/// Implementation of the SyncPriorityQueue methods.
impl<T: Ord> SyncPriorityQueue<T> {
    /// Creates a new, empty SyncPriorityQueue.
    pub fn new() -> Self {
        SyncPriorityQueue {
            heap: RwLock::new(BinaryHeap::new()),
        }
    }

    /// Adds an item to the priority queue.
    pub fn push(&self, item: T) {
        let mut heap_guard = self.heap.write().unwrap();
        heap_guard.push(Reverse(item));
    }

    /// Removes and returns the highest-priority (lowest-valued) item.
    pub fn top_pop(&self) -> Option<T> {
        let mut heap_guard = self.heap.write().unwrap();
        heap_guard.pop().map(|Reverse(item)| item)
    }

    /// Returns a copy of the highest-priority (lowest-valued) item.
    pub fn top(&self) -> Option<T>
    where
        T: Clone,
    {
        let heap_guard = self.heap.read().unwrap();
        heap_guard.peek().map(|Reverse(item)| item.clone())
    }

    // Queue-compatible aliases so the priority queue can be used wherever a
    // FIFO-style container is expected (e.g. as a pluggable AsyncObserver store).

    /// Alias for `top`, provided for queue-compatible container usage.
    pub fn front(&self) -> Option<T>
    where
        T: Clone,
    {
        self.top()
    }

    /// Alias for `top_pop`, provided for queue-compatible container usage.
    pub fn front_pop(&self) -> Option<T> {
        self.top_pop()
    }

    /// Checks if the priority queue is empty.
    pub fn is_empty(&self) -> bool {
        let heap_guard = self.heap.read().unwrap();
        heap_guard.is_empty()
    }

    /// Returns the number of items in the priority queue.
    pub fn size(&self) -> usize {
        let heap_guard = self.heap.read().unwrap();
        heap_guard.len()
    }

    /// Clears all items from the priority queue.
    pub fn clear(&self) {
        let mut heap_guard = self.heap.write().unwrap();
        heap_guard.clear();
    }
}

/// Default implementation for SyncPriorityQueue.
impl<T: Ord> Default for SyncPriorityQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for SyncPriorityQueue.
#[cfg(test)]
mod tests {
    use super::SyncPriorityQueue;

    // basic test for min-heap pop order
    #[test]
    fn test_min_heap_pop_order() {
        let queue = SyncPriorityQueue::new();
        queue.push(5);
        queue.push(1);
        queue.push(3);
        assert_eq!(queue.top_pop(), Some(1));
        assert_eq!(queue.top_pop(), Some(3));
        assert_eq!(queue.top_pop(), Some(5));
        assert_eq!(queue.top_pop(), None);
    }

    // basic test for top (peek without removing)
    #[test]
    fn test_top_does_not_remove() {
        let queue = SyncPriorityQueue::new();
        queue.push(2);
        queue.push(1);
        assert_eq!(queue.top(), Some(1));
        assert_eq!(queue.size(), 2);
    }

    // test for front/front_pop queue-compatible aliases
    #[test]
    fn test_front_aliases() {
        let queue = SyncPriorityQueue::new();
        queue.push(9);
        queue.push(4);
        assert_eq!(queue.front(), Some(4));
        assert_eq!(queue.front_pop(), Some(4));
        assert_eq!(queue.front_pop(), Some(9));
    }

    // basic test for is_empty method
    #[test]
    fn test_is_empty() {
        let queue = SyncPriorityQueue::new();
        assert!(queue.is_empty());
        queue.push(1);
        assert!(!queue.is_empty());
    }

    // basic test for size method
    #[test]
    fn test_size() {
        let queue = SyncPriorityQueue::new();
        assert_eq!(queue.size(), 0);
        queue.push(1);
        queue.push(2);
        assert_eq!(queue.size(), 2);
    }

    // basic test for clear method
    #[test]
    fn test_clear() {
        let queue = SyncPriorityQueue::new();
        queue.push(1);
        queue.push(2);
        queue.clear();
        assert!(queue.is_empty());
    }

    // test for Default trait
    #[test]
    fn test_default() {
        let queue: SyncPriorityQueue<i32> = SyncPriorityQueue::default();
        assert_eq!(queue.size(), 0);
    }

    // Additional test with two threads
    use std::sync::Arc;
    use std::thread;
    #[test]
    fn test_concurrent_access() {
        let queue = Arc::new(SyncPriorityQueue::new());
        let queue_for_producer = queue.clone();
        let queue_for_consumer = queue.clone();

        let producer = thread::spawn(move || {
            for i in 0..100 {
                queue_for_producer.push(i);
            }
        });
        producer.join().unwrap();

        let consumer = thread::spawn(move || {
            let mut previous: Option<i32> = None;
            let mut ordered = true;
            let mut count = 0;
            while let Some(value) = queue_for_consumer.top_pop() {
                if let Some(prev) = previous {
                    if value < prev {
                        ordered = false;
                    }
                }
                previous = Some(value);
                count += 1;
            }
            (ordered, count)
        });

        let (ordered, count) = consumer.join().unwrap();
        assert!(ordered);
        assert_eq!(count, 100);
    }
}
