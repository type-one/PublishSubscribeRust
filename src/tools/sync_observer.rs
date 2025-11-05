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

use multimap::MultiMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

// https://juanchopanzacpp.wordpress.com/2013/02/24/simple-observer-pattern-implementation-c11/
// http://www.codeproject.com/Articles/328365/Understanding-and-Implementing-Observer-Pattern

/// Trait defining a synchronous observer.
pub trait SyncObserver<Topic, Evt> {
    fn inform(&self, topic: &Topic, event: &Evt, origin: &str);
}

/// Type alias for a synchronous subscription.
pub type SyncSubscription<Topic, Evt> = (Topic, Arc<dyn SyncObserver<Topic, Evt> + Send + Sync>);

/// Type alias for a loose-coupled handler function.
pub type LooseCoupledHandler<Topic, Evt> = dyn Fn(&Topic, &Evt, &str) + Send + Sync;

/// Struct representing a synchronous subject.
pub struct SyncSubject<Topic, Evt> {
    rwlock: RwLock<()>,
    subscribers: MultiMap<Topic, Arc<dyn SyncObserver<Topic, Evt> + Send + Sync>>,
    handlers: MultiMap<Topic, (Arc<LooseCoupledHandler<Topic, Evt>>, String)>,
    name: String,
}

/// Implementation of the SyncSubject methods.
impl<Topic: Eq + Hash + Clone, Evt> SyncSubject<Topic, Evt> {
    /// Creates a new SyncSubject.
    pub fn new(name: &str) -> Self {
        SyncSubject {
            rwlock: RwLock::new(()),
            subscribers: MultiMap::new(),
            handlers: MultiMap::new(),
            name: name.to_string(),
        }
    }

    /// Subscribes an observer to a topic.
    pub fn subscribe(
        &mut self,
        topic: Topic,
        observer: Arc<dyn SyncObserver<Topic, Evt> + Send + Sync>,
    ) {
        let _lock = self.rwlock.write().unwrap();
        self.subscribers.insert(topic, observer);
    }

    /// Unsubscribes a single observer instance from a topic (keeps other observers for the same topic).
    pub fn unsubscribe(
        &mut self,
        topic: &Topic,
        observer: &Arc<dyn SyncObserver<Topic, Evt> + Send + Sync>,
    ) {
        let _lock = self.rwlock.write().unwrap();
        if let Some(observers) = self.subscribers.get_vec(topic) {
            // Keep only those observers that are NOT the one to remove.
            let retained: Vec<Arc<dyn SyncObserver<Topic, Evt> + Send + Sync>> = observers
                .iter()
                .filter(|o| !Arc::ptr_eq(o, observer))
                .cloned()
                .collect();

            // Remove all current entries for the topic and re-insert the retained ones.
            self.subscribers.remove(topic);
            for other_observer in retained {
                self.subscribers.insert(topic.clone(), other_observer);
            }
        }
    }

    /// Subscribes a loose-coupled handler to a topic.
    pub fn subscribe_handler(
        &mut self,
        topic: Topic,
        handler: Arc<LooseCoupledHandler<Topic, Evt>>,
        handler_name: &str,
    ) {
        let _lock = self.rwlock.write().unwrap();
        self.handlers
            .insert(topic, (handler, handler_name.to_string()));
    }

    /// Unsubscribes a loose-coupled handler from a topic by name.
    pub fn unsubscribe_handler(&mut self, topic: &Topic, handler_name: &str) {
        let _lock = self.rwlock.write().unwrap();
        if let Some(handlers) = self.handlers.get_vec(topic) {
            // Keep only those handlers whose name does NOT match handler_name.
            let retained: Vec<(Arc<LooseCoupledHandler<Topic, Evt>>, String)> = handlers
                .iter()
                .filter(|(_, name)| name != handler_name)
                .cloned()
                .collect();

            // Remove all current entries for the topic and re-insert the retained ones.
            self.handlers.remove(topic);
            for (handler, name) in retained {
                self.handlers.insert(topic.clone(), (handler, name));
            }
        }
    }

    /// Publishes an event to a topic with a specified origin.
    pub fn publish_named(&self, topic: &Topic, event: &Evt, origin: &str) {
        let _lock = self.rwlock.read().unwrap();
        if let Some(observers) = self.subscribers.get_vec(topic) {
            for observer in observers {
                observer.inform(topic, event, origin);
            }
        }
        if let Some(handlers) = self.handlers.get_vec(topic) {
            for (handler, _) in handlers {
                handler(topic, event, origin);
            }
        }
    }

    /// Publishes an event to a topic using the subject's name as the origin.
    pub fn publish(&self, topic: &Topic, event: &Evt) {
        self.publish_named(topic, event, &self.name);
    }
}
