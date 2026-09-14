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

/// Result of a batch push in overwrite mode: how many items were inserted and
/// how many pre-existing items were evicted to make room for them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PushRangeOverwriteResult {
    pub inserted: usize,
    pub overwritten: usize,
}

/// Fixed-capacity ring buffer (single-threaded, not synchronized).
/// `CAPACITY` is set at compile time via a const generic parameter.
pub struct RingBuffer<T, const CAPACITY: usize> {
    slots: Vec<Option<T>>,
    head: usize,
    len: usize,
}

/// Implementation of the RingBuffer methods.
impl<T, const CAPACITY: usize> RingBuffer<T, CAPACITY> {
    /// Creates a new, empty RingBuffer with a fixed capacity of `CAPACITY`.
    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(CAPACITY);
        slots.resize_with(CAPACITY, || None);
        RingBuffer {
            slots,
            head: 0,
            len: 0,
        }
    }

    /// Returns the maximum number of items the ring buffer can hold.
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Returns the number of items currently stored in the ring buffer.
    pub fn size(&self) -> usize {
        self.len
    }

    /// Checks if the ring buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Checks if the ring buffer reached its capacity.
    pub fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    fn slot_index(&self, offset: usize) -> usize {
        (self.head + offset) % CAPACITY
    }

    /// Adds an item to the back of the ring buffer unless it is already full.
    /// Returns true if the item was pushed, false if it was rejected.
    pub fn push(&mut self, item: T) -> bool {
        if self.is_full() {
            return false;
        }
        let index = self.slot_index(self.len);
        self.slots[index] = Some(item);
        self.len += 1;
        true
    }

    /// Adds an item to the back of the ring buffer, evicting the oldest item
    /// once full instead of rejecting the new one. Returns true if an
    /// existing item was evicted to make room.
    pub fn push_overwrite(&mut self, item: T) -> bool {
        if !self.is_full() {
            self.push(item);
            return false;
        }
        self.slots[self.head] = None;
        self.head = (self.head + 1) % CAPACITY;
        self.len -= 1;
        self.push(item);
        true
    }

    /// Pushes items from an iterator, stopping once the ring buffer is full.
    /// Returns the number of items actually inserted.
    pub fn push_range<I: IntoIterator<Item = T>>(&mut self, items: I) -> usize {
        let mut inserted = 0;
        for item in items {
            if !self.push(item) {
                break;
            }
            inserted += 1;
        }
        inserted
    }

    /// Pushes items from an iterator in overwrite mode: once full, each new
    /// item evicts the oldest one instead of being rejected.
    pub fn push_range_overwrite<I: IntoIterator<Item = T>>(
        &mut self,
        items: I,
    ) -> PushRangeOverwriteResult {
        let mut result = PushRangeOverwriteResult::default();
        for item in items {
            if self.push_overwrite(item) {
                result.overwritten += 1;
            }
            result.inserted += 1;
        }
        result
    }

    /// Removes and returns the item at the front of the ring buffer.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let item = self.slots[self.head].take();
        self.head = (self.head + 1) % CAPACITY;
        self.len -= 1;
        item
    }

    /// Alias for `pop`, provided for queue-compatible container usage.
    pub fn front_pop(&mut self) -> Option<T> {
        self.pop()
    }

    /// Removes and returns up to `max_count` items from the front of the ring buffer.
    pub fn pop_range(&mut self, max_count: usize) -> Vec<T> {
        let mut popped = Vec::with_capacity(max_count.min(self.len));
        while popped.len() < max_count {
            match self.pop() {
                Some(item) => popped.push(item),
                None => break,
            }
        }
        popped
    }

    /// Returns a reference to the front item of the ring buffer.
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.slots[self.head].as_ref()
        }
    }

    /// Returns a reference to the back item of the ring buffer.
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.slots[self.slot_index(self.len - 1)].as_ref()
        }
    }

    /// Clears all items from the ring buffer.
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
        self.head = 0;
        self.len = 0;
    }
}

/// Default implementation for RingBuffer.
impl<T, const CAPACITY: usize> Default for RingBuffer<T, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for RingBuffer.
#[cfg(test)]
mod tests {
    use super::RingBuffer;

    // basic test for push and pop operations
    #[test]
    fn test_push_pop() {
        let mut buffer: RingBuffer<i32, 4> = RingBuffer::new();
        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), None);
    }

    // test for front_pop alias
    #[test]
    fn test_front_pop() {
        let mut buffer: RingBuffer<i32, 4> = RingBuffer::new();
        buffer.push(1);
        buffer.push(2);
        assert_eq!(buffer.front_pop(), Some(1));
        assert_eq!(buffer.front_pop(), Some(2));
        assert_eq!(buffer.front_pop(), None);
    }

    // test reject-on-full mode
    #[test]
    fn test_push_rejected_when_full() {
        let mut buffer: RingBuffer<i32, 2> = RingBuffer::new();
        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert!(!buffer.push(3));
        assert_eq!(buffer.size(), 2);
        assert!(buffer.is_full());
    }

    // test overwrite-on-full mode evicts the oldest item
    #[test]
    fn test_push_overwrite_evicts_oldest() {
        let mut buffer: RingBuffer<i32, 3> = RingBuffer::new();
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        assert!(buffer.push_overwrite(4));
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), Some(3));
        assert_eq!(buffer.pop(), Some(4));
    }

    // test push_range stops at capacity
    #[test]
    fn test_push_range_stops_at_capacity() {
        let mut buffer: RingBuffer<i32, 3> = RingBuffer::new();
        let inserted = buffer.push_range(vec![1, 2, 3, 4, 5]);
        assert_eq!(inserted, 3);
        assert!(buffer.is_full());
    }

    // test push_range_overwrite reports eviction count
    #[test]
    fn test_push_range_overwrite_reports_counts() {
        let mut buffer: RingBuffer<i32, 3> = RingBuffer::new();
        let result = buffer.push_range_overwrite(vec![1, 2, 3, 4, 5]);
        assert_eq!(result.inserted, 5);
        assert_eq!(result.overwritten, 2);
        assert_eq!(buffer.pop_range(3), vec![3, 4, 5]);
    }

    // test wrap-around behavior after repeated push/pop cycles
    #[test]
    fn test_wrap_around() {
        let mut buffer: RingBuffer<i32, 2> = RingBuffer::new();
        for value in 0..10 {
            buffer.push(value);
            assert_eq!(buffer.pop(), Some(value));
        }
        assert!(buffer.is_empty());
    }

    // basic test for front and back methods
    #[test]
    fn test_front_back() {
        let mut buffer: RingBuffer<i32, 4> = RingBuffer::new();
        buffer.push(1);
        buffer.push(2);
        assert_eq!(buffer.front(), Some(&1));
        assert_eq!(buffer.back(), Some(&2));
    }

    // basic test for clear method
    #[test]
    fn test_clear() {
        let mut buffer: RingBuffer<i32, 4> = RingBuffer::new();
        buffer.push(1);
        buffer.push(2);
        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.size(), 0);
    }

    // test for Default trait
    #[test]
    fn test_default() {
        let buffer: RingBuffer<i32, 4> = RingBuffer::default();
        assert_eq!(buffer.capacity(), 4);
        assert_eq!(buffer.size(), 0);
    }
}
