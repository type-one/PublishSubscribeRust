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
use crate::tools::sync_observer::SyncObserver;
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
            wakeable_sync_object: Arc::new(SyncObject::new(false)),
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
impl<Topic: Send + Sync + Clone + 'static, Evt: Send + Sync + Clone + 'static>
    SyncObserver<Topic, Evt> for AsyncObserver<Topic, Evt>
{
    /// Informs the observer of an event.
    fn inform(&self, topic: &Topic, event: &Evt, origin: &str) {
        let record = ((*topic).clone(), (*event).clone(), origin.to_string());

        self.event_queue.enqueue(record);
        self.wakeable_sync_object.signal();
    }
}
