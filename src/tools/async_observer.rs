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

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::tools::sync_object::SyncObject;
use crate::tools::sync_observer::Observer;
use crate::tools::sync_priority_queue::SyncPriorityQueue;
use crate::tools::sync_queue::SyncQueue;
use crate::tools::sync_ring_buffer::SyncRingBuffer;
use crate::tools::sync_vector::SyncVector;

/// Type alias for an event entry.
pub type EventEntry<Topic, Evt> = (Topic, Evt, String);

/// Pluggable event storage for AsyncObserver. Implemented by an unbounded
/// `SyncQueue` (never rejects an entry), a bounded `SyncVector` or
/// fixed-capacity `SyncRingBuffer` (both reject entries once full so
/// overflow can be observed), or a `SyncPriorityQueue` (delivers entries in
/// priority order instead of FIFO order).
trait EventStore<T>: Send + Sync {
    /// Stores an entry. Returns false when the store rejected it (full).
    fn push(&self, entry: T) -> bool;
    fn dequeue(&self) -> Option<T>;
    fn is_empty(&self) -> bool;
    fn size(&self) -> usize;
}

impl<T: Send + Sync> EventStore<T> for SyncQueue<T> {
    fn push(&self, entry: T) -> bool {
        self.enqueue(entry);
        true
    }

    fn dequeue(&self) -> Option<T> {
        SyncQueue::dequeue(self)
    }

    fn is_empty(&self) -> bool {
        SyncQueue::is_empty(self)
    }

    fn size(&self) -> usize {
        SyncQueue::size(self)
    }
}

impl<T: Send + Sync> EventStore<T> for SyncVector<T> {
    fn push(&self, entry: T) -> bool {
        SyncVector::push(self, entry)
    }

    fn dequeue(&self) -> Option<T> {
        self.pop_front()
    }

    fn is_empty(&self) -> bool {
        SyncVector::is_empty(self)
    }

    fn size(&self) -> usize {
        SyncVector::size(self)
    }
}

impl<T: Ord + Send + Sync> EventStore<T> for SyncPriorityQueue<T> {
    fn push(&self, entry: T) -> bool {
        SyncPriorityQueue::push(self, entry);
        true
    }

    fn dequeue(&self) -> Option<T> {
        self.top_pop()
    }

    fn is_empty(&self) -> bool {
        SyncPriorityQueue::is_empty(self)
    }

    fn size(&self) -> usize {
        SyncPriorityQueue::size(self)
    }
}

impl<T: Send + Sync, const CAPACITY: usize> EventStore<T> for SyncRingBuffer<T, CAPACITY> {
    fn push(&self, entry: T) -> bool {
        SyncRingBuffer::push(self, entry)
    }

    fn dequeue(&self) -> Option<T> {
        self.pop()
    }

    fn is_empty(&self) -> bool {
        SyncRingBuffer::is_empty(self)
    }

    fn size(&self) -> usize {
        SyncRingBuffer::size(self)
    }
}

/// Struct representing an asynchronous observer.
pub struct AsyncObserver<Topic, Evt> {
    wakeable_sync_object: Arc<SyncObject>,
    event_queue: Arc<dyn EventStore<EventEntry<Topic, Evt>>>,
    overflow_count: Arc<AtomicUsize>,
}

// Topic and Event must be Send + Sync + 'static to be safely shared across threads.
// It means that they can be transferred across thread boundaries (Send),
// can be referenced from multiple threads simultaneously (Sync), and does not
// contain any non-static references ('static - static lifetime - valid for the
// entire duration of the program).

/// Implementation of the AsyncObserver methods.
impl<Topic: Send + Sync + 'static, Evt: Send + Sync + 'static> AsyncObserver<Topic, Evt> {
    /// Creates a new AsyncObserver backed by an unbounded `SyncQueue`.
    pub fn new() -> Self {
        AsyncObserver {
            wakeable_sync_object: Arc::new(SyncObject::new()),
            event_queue: Arc::new(SyncQueue::new()),
            overflow_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Creates a new AsyncObserver backed by a bounded `SyncVector`. Once the
    /// vector reaches `queue_capacity`, further events are dropped and counted
    /// as overflow.
    pub fn with_capacity(queue_capacity: usize) -> Self {
        AsyncObserver {
            wakeable_sync_object: Arc::new(SyncObject::new()),
            event_queue: Arc::new(SyncVector::new(queue_capacity)),
            overflow_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Creates a new AsyncObserver backed by a fixed-capacity `SyncRingBuffer`.
    /// Once the ring buffer reaches its compile-time `CAPACITY`, further
    /// events are dropped and counted as overflow.
    pub fn with_ring_buffer_capacity<const CAPACITY: usize>() -> Self {
        AsyncObserver {
            wakeable_sync_object: Arc::new(SyncObject::new()),
            event_queue: Arc::new(SyncRingBuffer::<EventEntry<Topic, Evt>, CAPACITY>::new()),
            overflow_count: Arc::new(AtomicUsize::new(0)),
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

    /// Returns true if at least one event was dropped due to queue overflow.
    pub fn has_queue_overflow(&self) -> bool {
        self.overflow_count.load(Ordering::Relaxed) != 0
    }

    /// Returns the number of events dropped due to queue overflow so far.
    pub fn queue_overflow_count(&self) -> usize {
        self.overflow_count.load(Ordering::Relaxed)
    }

    /// Returns the number of dropped events and resets the overflow counter to zero.
    pub fn consume_queue_overflow_count(&self) -> usize {
        self.overflow_count.swap(0, Ordering::Relaxed)
    }
}

/// Creates AsyncObserver backed by a `SyncPriorityQueue`, delivering events in
/// priority order (lowest first) instead of FIFO order. Requires the event
/// entry (topic, event, origin) to be `Ord`, which in turn requires `Topic`
/// and `Evt` to be `Ord`.
impl<Topic, Evt> AsyncObserver<Topic, Evt>
where
    Topic: Send + Sync + Ord + 'static,
    Evt: Send + Sync + Ord + 'static,
{
    /// Creates a new AsyncObserver backed by a `SyncPriorityQueue`.
    pub fn with_priority() -> Self {
        AsyncObserver {
            wakeable_sync_object: Arc::new(SyncObject::new()),
            event_queue: Arc::new(SyncPriorityQueue::new()),
            overflow_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl<Topic: Send + Sync + 'static, Evt: Send + Sync + 'static> Default
    for AsyncObserver<Topic, Evt>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Topic, Evt> Drop for AsyncObserver<Topic, Evt> {
    fn drop(&mut self) {
        self.wakeable_sync_object.signal_all();
    }
}

/// Implementation of the SyncObserver trait for AsyncObserver.
impl<Topic: Send + Sync + Clone + 'static, Evt: Send + Sync + Clone + 'static> Observer<Topic, Evt>
    for AsyncObserver<Topic, Evt>
{
    /// Informs the observer of an event.
    fn inform(&self, topic: &Topic, event: &Evt, origin: &str) {
        let record = ((*topic).clone(), (*event).clone(), origin.to_string());

        if self.event_queue.push(record) {
            self.wakeable_sync_object.signal();
        } else {
            self.overflow_count.fetch_add(1, Ordering::Relaxed);
        }
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

    // basic test for inform and pop
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

    // basic test for wait_for_events
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

    // test for inform from multiple threads
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
        for (i, event) in all_events.iter().enumerate().take(5) {
            assert_eq!(
                *event,
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

    // test queue overflow detection with a bounded observer
    #[test]
    fn test_async_observer_queue_overflow() {
        let observer: AsyncObserver<String, String> = AsyncObserver::with_capacity(2);
        observer.inform(&"topic".to_string(), &"event-1".to_string(), "producer");
        observer.inform(&"topic".to_string(), &"event-2".to_string(), "producer");
        observer.inform(
            &"topic".to_string(),
            &"event-3-dropped".to_string(),
            "producer",
        );

        assert!(observer.has_queue_overflow());
        assert_eq!(observer.queue_overflow_count(), 1);

        assert_eq!(observer.consume_queue_overflow_count(), 1);
        assert!(!observer.has_queue_overflow());

        let events = observer.pop_all_events();
        assert_eq!(events.len(), 2);
    }

    // test queue overflow detection with a fixed-capacity SyncRingBuffer-backed observer
    #[test]
    fn test_async_observer_ring_buffer_overflow() {
        let observer: AsyncObserver<String, String> =
            AsyncObserver::with_ring_buffer_capacity::<2>();
        observer.inform(&"topic".to_string(), &"event-1".to_string(), "producer");
        observer.inform(&"topic".to_string(), &"event-2".to_string(), "producer");
        observer.inform(
            &"topic".to_string(),
            &"event-3-dropped".to_string(),
            "producer",
        );

        assert!(observer.has_queue_overflow());
        assert_eq!(observer.queue_overflow_count(), 1);

        assert_eq!(observer.consume_queue_overflow_count(), 1);
        assert!(!observer.has_queue_overflow());

        let events = observer.pop_all_events();
        assert_eq!(events.len(), 2);
    }

    // test priority-ordered delivery with a SyncPriorityQueue-backed observer
    #[test]
    fn test_async_observer_priority_order() {
        let observer: AsyncObserver<String, i32> = AsyncObserver::with_priority();
        observer.inform(&"topic".to_string(), &5, "producer");
        observer.inform(&"topic".to_string(), &1, "producer");
        observer.inform(&"topic".to_string(), &3, "producer");

        assert_eq!(observer.pop_first_event().unwrap().1, 1);
        assert_eq!(observer.pop_first_event().unwrap().1, 3);
        assert_eq!(observer.pop_first_event().unwrap().1, 5);
    }

    // test drop with other thread waiting
    #[test]
    fn test_drop_with_waiting_thread() {
        let observer: Arc<AsyncObserver<String, i32>> = Arc::new(AsyncObserver::new());
        let observer_for_waiter = observer.clone();
        let waiter = thread::spawn(move || {
            observer_for_waiter.wait_for_events(1000);
        });
        drop(observer);
        waiter.join().unwrap();
    }
}
