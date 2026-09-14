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

use crate::tools::time_list::{TimeEntry, TimeList};

/// Thread-safe adapter over `TimeList`: a chronological list keyed by
/// timestamp, always exposing the earliest entry first.
pub struct SyncTimeList<Timestamp: Ord, Value> {
    list: RwLock<TimeList<Timestamp, Value>>,
}

/// Implementation of the SyncTimeList methods.
impl<Timestamp: Ord, Value> SyncTimeList<Timestamp, Value> {
    /// Creates a new, empty SyncTimeList.
    pub fn new() -> Self {
        SyncTimeList {
            list: RwLock::new(TimeList::new()),
        }
    }

    /// Adds a (timestamp, value) entry to the list.
    pub fn push(&self, timestamp: Timestamp, value: Value) {
        let mut list_guard = self.list.write().unwrap();
        list_guard.push(timestamp, value);
    }

    /// Returns a copy of the earliest entry without removing it.
    pub fn top(&self) -> Option<TimeEntry<Timestamp, Value>>
    where
        Timestamp: Clone,
        Value: Clone,
    {
        let list_guard = self.list.read().unwrap();
        list_guard.top()
    }

    /// Removes the earliest entry, if any, without returning it.
    pub fn pop(&self) {
        let mut list_guard = self.list.write().unwrap();
        list_guard.pop();
    }

    /// Removes and returns the earliest entry.
    pub fn top_pop(&self) -> Option<TimeEntry<Timestamp, Value>> {
        let mut list_guard = self.list.write().unwrap();
        list_guard.top_pop()
    }

    /// Checks if the list is empty.
    pub fn is_empty(&self) -> bool {
        let list_guard = self.list.read().unwrap();
        list_guard.is_empty()
    }

    /// Returns the number of entries in the list.
    pub fn size(&self) -> usize {
        let list_guard = self.list.read().unwrap();
        list_guard.size()
    }

    /// Clears all entries from the list.
    pub fn clear(&self) {
        let mut list_guard = self.list.write().unwrap();
        list_guard.clear();
    }

    /// Returns all entries sorted from earliest to latest timestamp, without
    /// draining the list.
    pub fn snapshot_sorted(&self) -> Vec<TimeEntry<Timestamp, Value>>
    where
        Timestamp: Clone,
        Value: Clone,
    {
        let list_guard = self.list.read().unwrap();
        list_guard.snapshot_sorted()
    }
}

/// Default implementation for SyncTimeList.
impl<Timestamp: Ord, Value> Default for SyncTimeList<Timestamp, Value> {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for SyncTimeList.
#[cfg(test)]
mod tests {
    use super::SyncTimeList;

    // basic test for chronological pop order
    #[test]
    fn test_chronological_pop_order() {
        let list = SyncTimeList::new();
        list.push(30, 30);
        list.push(10, 10);
        list.push(40, 40);
        list.push(20, 20);

        assert_eq!(list.top_pop(), Some((10, 10)));
        assert_eq!(list.top_pop(), Some((20, 20)));
        assert_eq!(list.top_pop(), Some((30, 30)));
        assert_eq!(list.top_pop(), Some((40, 40)));
        assert_eq!(list.top_pop(), None);
    }

    // basic test for top (peek without removing)
    #[test]
    fn test_top_does_not_remove() {
        let list = SyncTimeList::new();
        list.push(2, "two");
        list.push(1, "one");
        assert_eq!(list.top(), Some((1, "one")));
        assert_eq!(list.size(), 2);
    }

    // basic test for is_empty
    #[test]
    fn test_is_empty() {
        let list = SyncTimeList::new();
        assert!(list.is_empty());
        list.push(1, "one");
        assert!(!list.is_empty());
    }

    // basic test for clear
    #[test]
    fn test_clear() {
        let list = SyncTimeList::new();
        list.push(1, "one");
        list.push(2, "two");
        list.clear();
        assert!(list.is_empty());
    }

    // basic test for snapshot_sorted not draining the list
    #[test]
    fn test_snapshot_sorted_does_not_drain() {
        let list = SyncTimeList::new();
        list.push(30, "thirty");
        list.push(10, "ten");
        list.push(20, "twenty");

        let snapshot = list.snapshot_sorted();
        assert_eq!(snapshot, vec![(10, "ten"), (20, "twenty"), (30, "thirty")]);
        assert_eq!(list.size(), 3);
    }

    // test for Default trait
    #[test]
    fn test_default() {
        let list: SyncTimeList<i64, i32> = SyncTimeList::default();
        assert_eq!(list.size(), 0);
    }

    // Additional test with two threads
    use std::sync::Arc;
    use std::thread;
    #[test]
    fn test_concurrent_access() {
        let list = Arc::new(SyncTimeList::new());
        let list_for_producer = list.clone();
        let list_for_consumer = list.clone();

        let producer = thread::spawn(move || {
            for i in 0..100 {
                list_for_producer.push(i, i);
            }
        });
        producer.join().unwrap();

        let consumer = thread::spawn(move || {
            let mut previous: Option<i64> = None;
            let mut ordered = true;
            let mut count = 0;
            while let Some((timestamp, _)) = list_for_consumer.top_pop() {
                if let Some(prev) = previous {
                    if timestamp < prev {
                        ordered = false;
                    }
                }
                previous = Some(timestamp);
                count += 1;
            }
            (ordered, count)
        });

        let (ordered, count) = consumer.join().unwrap();
        assert!(ordered);
        assert_eq!(count, 100);
    }
}
