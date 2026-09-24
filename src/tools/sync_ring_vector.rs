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

use crate::tools::ring_buffer::PushRangeOverwriteResult;

/// Thread-safe, bounded ring vector on top of a preallocated `VecDeque`.
/// Unlike `SyncQueue`, `push` fails once `capacity` items are stored instead
/// of growing unbounded, so callers can detect and count dropped items.
#[derive(Debug)]
pub struct SyncRingVector<T> {
    vector: RwLock<VecDeque<T>>,
    capacity: usize,
}

/// Implementation of the SyncRingVector methods.
impl<T> SyncRingVector<T> {
    /// Creates a new SyncRingVector, preallocating storage for `capacity` items.
    pub fn new(capacity: usize) -> Self {
        SyncRingVector {
            vector: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// Adds an item to the back of the vector unless it is already full.
    /// Returns true if the item was pushed, false if it was rejected.
    pub fn push(&self, item: T) -> bool {
        let mut vector_guard = self.vector.write().unwrap();
        if vector_guard.len() >= self.capacity {
            return false;
        }
        vector_guard.push_back(item);
        true
    }

    /// Removes and returns an item from the front of the vector.
    pub fn pop_front(&self) -> Option<T> {
        let mut vector_guard = self.vector.write().unwrap();
        vector_guard.pop_front()
    }

    /// Alias for `pop_front`, provided for queue-compatible container usage.
    pub fn front_pop(&self) -> Option<T> {
        self.pop_front()
    }

    /// Adds an item to the back of the vector, evicting the oldest item once
    /// full instead of rejecting the new one. Returns true if an existing
    /// item was evicted to make room.
    pub fn push_overwrite(&self, item: T) -> bool {
        let mut vector_guard = self.vector.write().unwrap();
        if self.capacity == 0 {
            return false;
        }
        let evicted = if vector_guard.len() >= self.capacity {
            vector_guard.pop_front();
            true
        } else {
            false
        };
        vector_guard.push_back(item);
        evicted
    }

    /// Pushes items from an iterator, stopping once the vector is full.
    /// Returns the number of items actually inserted.
    pub fn push_range<I: IntoIterator<Item = T>>(&self, items: I) -> usize {
        let mut vector_guard = self.vector.write().unwrap();
        let mut inserted = 0;
        for item in items {
            if vector_guard.len() >= self.capacity {
                break;
            }
            vector_guard.push_back(item);
            inserted += 1;
        }
        inserted
    }

    /// Pushes items from an iterator in overwrite mode: once full, each new
    /// item evicts the oldest one instead of being rejected.
    pub fn push_range_overwrite<I: IntoIterator<Item = T>>(
        &self,
        items: I,
    ) -> PushRangeOverwriteResult {
        let mut vector_guard = self.vector.write().unwrap();
        let mut result = PushRangeOverwriteResult::default();
        if self.capacity == 0 {
            return result;
        }
        for item in items {
            if vector_guard.len() >= self.capacity {
                vector_guard.pop_front();
                result.overwritten += 1;
            }
            vector_guard.push_back(item);
            result.inserted += 1;
        }
        result
    }

    /// Removes and returns up to `max_count` items from the front of the vector.
    pub fn pop_range(&self, max_count: usize) -> Vec<T> {
        let mut vector_guard = self.vector.write().unwrap();
        let count = max_count.min(vector_guard.len());
        vector_guard.drain(..count).collect()
    }

    /// Checks if the vector is empty.
    pub fn is_empty(&self) -> bool {
        let vector_guard = self.vector.read().unwrap();
        vector_guard.is_empty()
    }

    /// Checks if the vector reached its capacity.
    pub fn is_full(&self) -> bool {
        let vector_guard = self.vector.read().unwrap();
        vector_guard.len() >= self.capacity
    }

    /// Returns the number of items currently stored in the vector.
    pub fn size(&self) -> usize {
        let vector_guard = self.vector.read().unwrap();
        vector_guard.len()
    }

    /// Returns the maximum number of items the vector can hold.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clears all items from the vector.
    pub fn clear(&self) {
        let mut vector_guard = self.vector.write().unwrap();
        vector_guard.clear();
    }

    /// Returns a copy of the front item of the vector.
    pub fn front(&self) -> Option<T>
    where
        T: Clone,
    {
        let vector_guard = self.vector.read().unwrap();
        vector_guard.front().cloned()
    }

    /// Returns a copy of the back item of the vector.
    pub fn back(&self) -> Option<T>
    where
        T: Clone,
    {
        let vector_guard = self.vector.read().unwrap();
        vector_guard.back().cloned()
    }
}

// Unit tests for SyncRingVector.
#[cfg(test)]
mod tests {
    use super::SyncRingVector;

    // Basic test for push and pop_front operations.
    #[test]
    fn test_push_pop_front() {
        let vector = SyncRingVector::new(4);
        assert!(vector.push(1));
        assert!(vector.push(2));
        assert_eq!(vector.pop_front(), Some(1));
        assert_eq!(vector.pop_front(), Some(2));
        assert_eq!(vector.pop_front(), None);
    }

    // Test that push is rejected once capacity is reached.
    #[test]
    fn test_push_rejected_when_full() {
        let vector = SyncRingVector::new(2);
        assert!(vector.push(1));
        assert!(vector.push(2));
        assert!(!vector.push(3));
        assert_eq!(vector.size(), 2);
        assert!(vector.is_full());
    }

    // Test push succeeds again after popping an item from a full vector.
    #[test]
    fn test_push_after_pop_frees_capacity() {
        let vector = SyncRingVector::new(1);
        assert!(vector.push(1));
        assert!(!vector.push(2));
        assert_eq!(vector.pop_front(), Some(1));
        assert!(vector.push(2));
        assert_eq!(vector.pop_front(), Some(2));
    }

    // Basic test for is_empty method.
    #[test]
    fn test_is_empty() {
        let vector = SyncRingVector::new(2);
        assert!(vector.is_empty());
        vector.push(1);
        assert!(!vector.is_empty());
        vector.pop_front();
        assert!(vector.is_empty());
    }

    // Basic test for size and capacity methods.
    #[test]
    fn test_size_and_capacity() {
        let vector = SyncRingVector::new(3);
        assert_eq!(vector.capacity(), 3);
        assert_eq!(vector.size(), 0);
        vector.push(1);
        vector.push(2);
        assert_eq!(vector.size(), 2);
    }

    // Basic test for clear method.
    #[test]
    fn test_clear() {
        let vector = SyncRingVector::new(2);
        vector.push(1);
        vector.push(2);
        vector.clear();
        assert!(vector.is_empty());
        assert!(!vector.is_full());
    }

    // Basic test for front and back methods.
    #[test]
    fn test_front_back() {
        let vector = SyncRingVector::new(2);
        vector.push(1);
        vector.push(2);
        assert_eq!(vector.front(), Some(1));
        assert_eq!(vector.back(), Some(2));
    }

    // test for front_pop alias
    #[test]
    fn test_front_pop() {
        let vector = SyncRingVector::new(2);
        vector.push(1);
        vector.push(2);
        assert_eq!(vector.front_pop(), Some(1));
        assert_eq!(vector.front_pop(), Some(2));
        assert_eq!(vector.front_pop(), None);
    }

    // test push_overwrite evicts the oldest item once full
    #[test]
    fn test_push_overwrite_evicts_oldest() {
        let vector = SyncRingVector::new(2);
        vector.push(1);
        vector.push(2);
        assert!(vector.push_overwrite(3));
        assert_eq!(vector.pop_front(), Some(2));
        assert_eq!(vector.pop_front(), Some(3));
    }

    #[test]
    fn test_zero_capacity_rejects_overwrite() {
        let vector = SyncRingVector::new(0);
        assert!(!vector.push_overwrite(1));
        assert_eq!(vector.size(), 0);

        let result = vector.push_range_overwrite(vec![2, 3]);
        assert_eq!(result.inserted, 0);
        assert_eq!(result.overwritten, 0);
        assert!(vector.is_empty());
    }

    // test push_range stops at capacity
    #[test]
    fn test_push_range_stops_at_capacity() {
        let vector = SyncRingVector::new(2);
        let inserted = vector.push_range(vec![1, 2, 3]);
        assert_eq!(inserted, 2);
        assert!(vector.is_full());
    }

    // test push_range_overwrite reports eviction count
    #[test]
    fn test_push_range_overwrite_reports_counts() {
        let vector = SyncRingVector::new(2);
        let result = vector.push_range_overwrite(vec![1, 2, 3, 4]);
        assert_eq!(result.inserted, 4);
        assert_eq!(result.overwritten, 2);
        assert_eq!(vector.pop_range(2), vec![3, 4]);
    }

    // Additional test with two threads
    use std::sync::Arc;
    use std::thread;
    #[test]
    fn test_concurrent_access() {
        let vector = Arc::new(SyncRingVector::new(8));
        let vector_for_producer = vector.clone();
        let vector_for_consumer = vector.clone();

        let producer = thread::spawn(move || {
            for i in 0..100 {
                while !vector_for_producer.push(i) {
                    thread::yield_now();
                }
            }
        });

        let consumer = thread::spawn(move || {
            let mut count = 0;
            while count < 100 {
                if vector_for_consumer.pop_front().is_some() {
                    count += 1;
                }
            }
        });

        producer.join().unwrap();
        consumer.join().unwrap();

        assert!(vector.is_empty());
    }
}
