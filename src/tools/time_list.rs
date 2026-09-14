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

use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

/// Type alias for a (timestamp, value) entry.
pub type TimeEntry<Timestamp, Value> = (Timestamp, Value);

// Wraps an entry so ordering only compares the timestamp, ignoring the value,
// matching the C++ time_list entry_compare (values need not be Ord).
struct Entry<Timestamp, Value> {
    timestamp: Timestamp,
    value: Value,
}

impl<Timestamp: PartialEq, Value> PartialEq for Entry<Timestamp, Value> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl<Timestamp: Eq, Value> Eq for Entry<Timestamp, Value> {}

impl<Timestamp: PartialOrd, Value> PartialOrd for Entry<Timestamp, Value> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl<Timestamp: Ord, Value> Ord for Entry<Timestamp, Value> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

/// Non-thread-safe chronological list: stores (timestamp, value) entries and
/// always exposes the earliest timestamp first. `Timestamp` must be `Ord`
/// (e.g. an integral tick count or `std::time::Instant`); `Value` has no
/// ordering requirement.
pub struct TimeList<Timestamp: Ord, Value> {
    heap: BinaryHeap<Reverse<Entry<Timestamp, Value>>>,
}

/// Implementation of the TimeList methods.
impl<Timestamp: Ord, Value> TimeList<Timestamp, Value> {
    /// Creates a new, empty TimeList.
    pub fn new() -> Self {
        TimeList {
            heap: BinaryHeap::new(),
        }
    }

    /// Adds a (timestamp, value) entry to the list.
    pub fn push(&mut self, timestamp: Timestamp, value: Value) {
        self.heap.push(Reverse(Entry { timestamp, value }));
    }

    /// Returns a copy of the earliest entry without removing it.
    pub fn top(&self) -> Option<TimeEntry<Timestamp, Value>>
    where
        Timestamp: Clone,
        Value: Clone,
    {
        self.heap
            .peek()
            .map(|Reverse(entry)| (entry.timestamp.clone(), entry.value.clone()))
    }

    /// Removes the earliest entry, if any, without returning it.
    pub fn pop(&mut self) {
        self.heap.pop();
    }

    /// Removes and returns the earliest entry.
    pub fn top_pop(&mut self) -> Option<TimeEntry<Timestamp, Value>> {
        self.heap
            .pop()
            .map(|Reverse(entry)| (entry.timestamp, entry.value))
    }

    /// Checks if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Returns the number of entries in the list.
    pub fn size(&self) -> usize {
        self.heap.len()
    }

    /// Clears all entries from the list.
    pub fn clear(&mut self) {
        self.heap.clear();
    }

    /// Returns all entries sorted from earliest to latest timestamp, without
    /// draining the list.
    pub fn snapshot_sorted(&self) -> Vec<TimeEntry<Timestamp, Value>>
    where
        Timestamp: Clone,
        Value: Clone,
    {
        let mut sorted: Vec<TimeEntry<Timestamp, Value>> = self
            .heap
            .iter()
            .map(|Reverse(entry)| (entry.timestamp.clone(), entry.value.clone()))
            .collect();
        sorted.sort_by(|left, right| left.0.cmp(&right.0));
        sorted
    }
}

/// Default implementation for TimeList.
impl<Timestamp: Ord, Value> Default for TimeList<Timestamp, Value> {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for TimeList.
#[cfg(test)]
mod tests {
    use super::TimeList;

    // basic test for chronological pop order
    #[test]
    fn test_chronological_pop_order() {
        let mut list = TimeList::new();
        list.push(300, "three hundred");
        list.push(100, "one hundred");
        list.push(200, "two hundred");

        assert_eq!(list.top_pop(), Some((100, "one hundred")));
        assert_eq!(list.top_pop(), Some((200, "two hundred")));
        assert_eq!(list.top_pop(), Some((300, "three hundred")));
        assert_eq!(list.top_pop(), None);
    }

    // basic test for top (peek without removing)
    #[test]
    fn test_top_does_not_remove() {
        let mut list = TimeList::new();
        list.push(2, "two");
        list.push(1, "one");
        assert_eq!(list.top(), Some((1, "one")));
        assert_eq!(list.size(), 2);
    }

    // basic test for pop (discard without returning)
    #[test]
    fn test_pop_discards_earliest() {
        let mut list = TimeList::new();
        list.push(2, "two");
        list.push(1, "one");
        list.pop();
        assert_eq!(list.top_pop(), Some((2, "two")));
    }

    // basic test for is_empty
    #[test]
    fn test_is_empty() {
        let mut list = TimeList::new();
        assert!(list.is_empty());
        list.push(1, "one");
        assert!(!list.is_empty());
    }

    // basic test for size
    #[test]
    fn test_size() {
        let mut list = TimeList::new();
        assert_eq!(list.size(), 0);
        list.push(1, "one");
        list.push(2, "two");
        assert_eq!(list.size(), 2);
    }

    // basic test for clear
    #[test]
    fn test_clear() {
        let mut list = TimeList::new();
        list.push(1, "one");
        list.push(2, "two");
        list.clear();
        assert!(list.is_empty());
    }

    // basic test for snapshot_sorted not draining the list
    #[test]
    fn test_snapshot_sorted_does_not_drain() {
        let mut list = TimeList::new();
        list.push(300, "three hundred");
        list.push(100, "one hundred");
        list.push(200, "two hundred");

        let snapshot = list.snapshot_sorted();
        assert_eq!(
            snapshot,
            vec![(100, "one hundred"), (200, "two hundred"), (300, "three hundred")]
        );
        assert_eq!(list.size(), 3);
    }

    // test for Default trait
    #[test]
    fn test_default() {
        let list: TimeList<i64, i32> = TimeList::default();
        assert_eq!(list.size(), 0);
    }

    // chrono-style coverage: Instant timestamps drain in chronological order
    #[test]
    fn test_instant_timestamp_order() {
        use std::time::{Duration, Instant};

        let base_time = Instant::now();
        let mut list = TimeList::new();
        list.push(base_time + Duration::from_millis(300), 3);
        list.push(base_time + Duration::from_millis(100), 1);
        list.push(base_time + Duration::from_millis(200), 2);

        assert_eq!(list.top_pop().map(|(_, value)| value), Some(1));
        assert_eq!(list.top_pop().map(|(_, value)| value), Some(2));
        assert_eq!(list.top_pop().map(|(_, value)| value), Some(3));
    }
}
