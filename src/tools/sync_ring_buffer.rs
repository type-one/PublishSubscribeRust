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

use std::sync::RwLock;

use crate::tools::ring_buffer::{PushRangeOverwriteResult, RingBuffer};

/// Thread-safe, fixed-capacity ring buffer built on top of `RingBuffer`.
/// `CAPACITY` is set at compile time via a const generic parameter.
pub struct SyncRingBuffer<T, const CAPACITY: usize> {
    buffer: RwLock<RingBuffer<T, CAPACITY>>,
}

/// Implementation of the SyncRingBuffer methods.
impl<T, const CAPACITY: usize> SyncRingBuffer<T, CAPACITY> {
    /// Creates a new, empty SyncRingBuffer with a fixed capacity of `CAPACITY`.
    pub fn new() -> Self {
        SyncRingBuffer {
            buffer: RwLock::new(RingBuffer::new()),
        }
    }

    /// Returns the maximum number of items the ring buffer can hold.
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Returns the number of items currently stored in the ring buffer.
    pub fn size(&self) -> usize {
        let buffer_guard = self.buffer.read().unwrap();
        buffer_guard.size()
    }

    /// Checks if the ring buffer is empty.
    pub fn is_empty(&self) -> bool {
        let buffer_guard = self.buffer.read().unwrap();
        buffer_guard.is_empty()
    }

    /// Checks if the ring buffer reached its capacity.
    pub fn is_full(&self) -> bool {
        let buffer_guard = self.buffer.read().unwrap();
        buffer_guard.is_full()
    }

    /// Adds an item to the back of the ring buffer unless it is already full.
    /// Returns true if the item was pushed, false if it was rejected.
    pub fn push(&self, item: T) -> bool {
        let mut buffer_guard = self.buffer.write().unwrap();
        buffer_guard.push(item)
    }

    /// Adds an item to the back of the ring buffer, evicting the oldest item
    /// once full instead of rejecting the new one. Returns true if an
    /// existing item was evicted to make room.
    pub fn push_overwrite(&self, item: T) -> bool {
        let mut buffer_guard = self.buffer.write().unwrap();
        buffer_guard.push_overwrite(item)
    }

    /// Pushes items from an iterator, stopping once the ring buffer is full.
    /// Returns the number of items actually inserted.
    pub fn push_range<I: IntoIterator<Item = T>>(&self, items: I) -> usize {
        let mut buffer_guard = self.buffer.write().unwrap();
        buffer_guard.push_range(items)
    }

    /// Pushes items from an iterator in overwrite mode: once full, each new
    /// item evicts the oldest one instead of being rejected.
    pub fn push_range_overwrite<I: IntoIterator<Item = T>>(
        &self,
        items: I,
    ) -> PushRangeOverwriteResult {
        let mut buffer_guard = self.buffer.write().unwrap();
        buffer_guard.push_range_overwrite(items)
    }

    /// Removes and returns the item at the front of the ring buffer.
    pub fn pop(&self) -> Option<T> {
        let mut buffer_guard = self.buffer.write().unwrap();
        buffer_guard.pop()
    }

    /// Alias for `pop`, provided for queue-compatible container usage.
    pub fn front_pop(&self) -> Option<T> {
        self.pop()
    }

    /// Removes and returns up to `max_count` items from the front of the ring buffer.
    pub fn pop_range(&self, max_count: usize) -> Vec<T> {
        let mut buffer_guard = self.buffer.write().unwrap();
        buffer_guard.pop_range(max_count)
    }

    /// Returns a copy of the front item of the ring buffer.
    pub fn front(&self) -> Option<T>
    where
        T: Clone,
    {
        let buffer_guard = self.buffer.read().unwrap();
        buffer_guard.front().cloned()
    }

    /// Returns a copy of the back item of the ring buffer.
    pub fn back(&self) -> Option<T>
    where
        T: Clone,
    {
        let buffer_guard = self.buffer.read().unwrap();
        buffer_guard.back().cloned()
    }

    /// Clears all items from the ring buffer.
    pub fn clear(&self) {
        let mut buffer_guard = self.buffer.write().unwrap();
        buffer_guard.clear();
    }
}

/// Default implementation for SyncRingBuffer.
impl<T, const CAPACITY: usize> Default for SyncRingBuffer<T, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for SyncRingBuffer.
#[cfg(test)]
mod tests {
    use super::SyncRingBuffer;

    // basic test for push and pop operations
    #[test]
    fn test_push_pop() {
        let buffer: SyncRingBuffer<i32, 4> = SyncRingBuffer::new();
        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), None);
    }

    // test reject-on-full mode
    #[test]
    fn test_push_rejected_when_full() {
        let buffer: SyncRingBuffer<i32, 2> = SyncRingBuffer::new();
        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert!(!buffer.push(3));
        assert!(buffer.is_full());
    }

    // test overwrite-on-full mode evicts the oldest item
    #[test]
    fn test_push_overwrite_evicts_oldest() {
        let buffer: SyncRingBuffer<i32, 3> = SyncRingBuffer::new();
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        assert!(buffer.push_overwrite(4));
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), Some(3));
        assert_eq!(buffer.pop(), Some(4));
    }

    // test push_range_overwrite reports eviction count
    #[test]
    fn test_push_range_overwrite_reports_counts() {
        let buffer: SyncRingBuffer<i32, 3> = SyncRingBuffer::new();
        let result = buffer.push_range_overwrite(vec![1, 2, 3, 4, 5]);
        assert_eq!(result.inserted, 5);
        assert_eq!(result.overwritten, 2);
        assert_eq!(buffer.pop_range(3), vec![3, 4, 5]);
    }

    // basic test for front/back and front_pop alias
    #[test]
    fn test_front_back_and_front_pop() {
        let buffer: SyncRingBuffer<i32, 4> = SyncRingBuffer::new();
        buffer.push(1);
        buffer.push(2);
        assert_eq!(buffer.front(), Some(1));
        assert_eq!(buffer.back(), Some(2));
        assert_eq!(buffer.front_pop(), Some(1));
    }

    // basic test for clear method
    #[test]
    fn test_clear() {
        let buffer: SyncRingBuffer<i32, 4> = SyncRingBuffer::new();
        buffer.push(1);
        buffer.push(2);
        buffer.clear();
        assert!(buffer.is_empty());
    }

    // test for Default trait
    #[test]
    fn test_default() {
        let buffer: SyncRingBuffer<i32, 4> = SyncRingBuffer::default();
        assert_eq!(buffer.capacity(), 4);
        assert_eq!(buffer.size(), 0);
    }

    // Additional test with two threads
    use std::sync::Arc;
    use std::thread;
    #[test]
    fn test_concurrent_access() {
        let buffer = Arc::new(SyncRingBuffer::<i32, 8>::new());
        let buffer_for_producer = buffer.clone();
        let buffer_for_consumer = buffer.clone();

        let producer = thread::spawn(move || {
            for i in 0..100 {
                while !buffer_for_producer.push(i) {
                    thread::yield_now();
                }
            }
        });

        let consumer = thread::spawn(move || {
            let mut count = 0;
            while count < 100 {
                if buffer_for_consumer.pop().is_some() {
                    count += 1;
                }
            }
        });

        producer.join().unwrap();
        consumer.join().unwrap();

        assert!(buffer.is_empty());
    }
}
