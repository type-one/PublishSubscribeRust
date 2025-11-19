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

use std::sync::Arc;

use crate::tools::sync_object::SyncObject;
use crate::tools::sync_observer::Observer;
use crate::tools::sync_queue::SyncQueue;

/// Type alias for an event entry.
pub type EventEntry<Topic, Evt> = (Topic, Evt, String);

/// Struct representing an asynchronous observer.
pub struct AsyncObserver<Topic, Evt> {
    wakeable_sync_object: Arc<SyncObject>,
    event_queue: Arc<SyncQueue<EventEntry<Topic, Evt>>>,
}

// Topic and Event must be Send + Sync + 'static to be safely shared across threads.
// It means that they can be transferred across thread boundaries (Send),
// can be referenced from multiple threads simultaneously (Sync), and does not
// contain any non-static references ('static - static lifetime - valid for the
// entire duration of the program).

/// Implementation of the AsyncObserver methods.
impl<Topic: Send + Sync + 'static, Evt: Send + Sync + 'static> AsyncObserver<Topic, Evt> {
    /// Creates a new AsyncObserver.
    pub fn new() -> Self {
        AsyncObserver {
            wakeable_sync_object: Arc::new(SyncObject::new()),
            event_queue: Arc::new(SyncQueue::new()),
        }
    }

    /// Pops all events from the event queue.
    pub fn pop_all_events(&self) -> Vec<(Topic, Evt, String)> {
        let mut events = Vec::new();
        while let Some(event) = self.event_queue.dequeue() {
            events.push(event);
        }
        events
    }

    /// Pops the first event from the event queue.
    pub fn pop_first_event(&self) -> Option<(Topic, Evt, String)> {
        self.event_queue.dequeue()
    }

    /// Pops the last event from the event queue.
    pub fn pop_last_event(&self) -> Option<(Topic, Evt, String)> {
        let mut last_event = None;
        while let Some(event) = self.event_queue.dequeue() {
            last_event = Some(event);
        }
        last_event
    }

    /// Checks if there are events in the event queue.
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Returns the number of events in the event queue.
    pub fn number_of_events(&self) -> usize {
        self.event_queue.size()
    }

    /// Waits for events with a timeout in milliseconds.
    pub fn wait_for_events(&self, timeout_ms: u64) {
        self.wakeable_sync_object
            .wait_for_signal_timeout(timeout_ms)
    }
}

impl<Topic: Send + Sync + 'static, Evt: Send + Sync + 'static> Default
    for AsyncObserver<Topic, Evt>
{
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of the SyncObserver trait for AsyncObserver.
impl<Topic: Send + Sync + Clone + 'static, Evt: Send + Sync + Clone + 'static> Observer<Topic, Evt>
    for AsyncObserver<Topic, Evt>
{
    /// Informs the observer of an event.
    fn inform(&self, topic: &Topic, event: &Evt, origin: &str) {
        let record = ((*topic).clone(), (*event).clone(), origin.to_string());

        self.event_queue.enqueue(record);
        self.wakeable_sync_object.signal();
    }
}

// Unit tests for AsyncObserver.
#[cfg(test)]
mod tests {
    use super::AsyncObserver;
    use crate::tools::sync_observer::Observer;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_async_observer_inform_and_pop() {
        let observer: AsyncObserver<String, i32> = AsyncObserver::new();
        observer.inform(&"topic1".to_string(), &42, "origin1");
        observer.inform(&"topic2".to_string(), &84, "origin2");
        assert_eq!(
            observer.pop_first_event(),
            Some(("topic1".to_string(), 42, "origin1".to_string()))
        );
        assert_eq!(
            observer.pop_last_event(),
            Some(("topic2".to_string(), 84, "origin2".to_string()))
        );
    }

    #[test]
    fn test_async_observer_wait_for_events() {
        let observer: Arc<AsyncObserver<String, i32>> = Arc::new(AsyncObserver::new());
        let child_observer = observer.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            child_observer.inform(&"topic_wait".to_string(), &123, "origin_wait");
        });
        observer.wait_for_events(500);
        assert_eq!(
            observer.pop_first_event(),
            Some(("topic_wait".to_string(), 123, "origin_wait".to_string()))
        );
    }

    #[test]
    fn test_async_observer_number_of_events() {
        let observer: AsyncObserver<String, i32> = AsyncObserver::new();
        assert_eq!(observer.number_of_events(), 0);
        observer.inform(&"topic1".to_string(), &1, "origin1");
        observer.inform(&"topic2".to_string(), &2, "origin2");
        assert_eq!(observer.number_of_events(), 2);
        observer.pop_first_event();
        assert_eq!(observer.number_of_events(), 1);
    }

    // test for Default trait
    #[test]
    fn test_default_trait() {
        let observer: AsyncObserver<String, i32> = AsyncObserver::default();
        assert_eq!(observer.number_of_events(), 0);
    }

    // Additional test with two threads
    #[test]
    fn test_concurrent_inform_and_pop() {
        let observer: Arc<AsyncObserver<String, i32>> = Arc::new(AsyncObserver::new());
        let observer_for_informer = observer.clone();
        let observer_for_popper = observer.clone();
        let informer = thread::spawn(move || {
            for i in 0..100 {
                observer_for_informer.inform(&format!("topic{}", i), &i, &format!("origin{}", i));
            }
        });
        let popper = thread::spawn(move || {
            for _ in 0..100 {
                observer_for_popper.pop_first_event();
            }
        });

        informer.join().unwrap();
        popper.join().unwrap();
    }

    // test pop_all_events
    #[test]
    fn test_pop_all_events() {
        let observer: AsyncObserver<String, i32> = AsyncObserver::new();
        for i in 0..5 {
            observer.inform(&format!("topic{}", i), &i, &format!("origin{}", i));
        }
        let all_events = observer.pop_all_events();
        assert_eq!(all_events.len(), 5);
        for i in 0..5 {
            assert_eq!(
                all_events[i],
                (format!("topic{}", i), i as i32, format!("origin{}", i))
            );
        }
    }

    // test has events
    #[test]
    fn test_has_events() {
        let observer: AsyncObserver<String, i32> = AsyncObserver::new();
        assert!(!observer.has_events());
        observer.inform(&"topic1".to_string(), &1, "origin1");
        assert!(observer.has_events());
    }
}
