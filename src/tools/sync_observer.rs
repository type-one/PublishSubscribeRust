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

/// Trait defining an observer.
pub trait Observer<Topic, Evt> {
    fn inform(&self, topic: &Topic, event: &Evt, origin: &str);
}

/// Type alias for a subscription.
pub type Subscription<Topic, Evt> = (Topic, Arc<dyn Observer<Topic, Evt> + Send + Sync>);

/// Type alias for a loose-coupled handler function.
pub type LooseCoupledHandler<Topic, Evt> = dyn Fn(&Topic, &Evt, &str) + Send + Sync;

pub trait SubjectTrait<Topic: Eq + Hash + Clone, Evt> {
    fn subscribe(&mut self, topic: &Topic, observer: Arc<dyn Observer<Topic, Evt> + Send + Sync>);
    fn unsubscribe(
        &mut self,
        topic: &Topic,
        observer: &Arc<dyn Observer<Topic, Evt> + Send + Sync>,
    );
    fn subscribe_handler(
        &mut self,
        topic: &Topic,
        handler: Arc<LooseCoupledHandler<Topic, Evt>>,
        handler_name: &str,
    );
    fn unsubscribe_handler(&mut self, topic: &Topic, handler_name: &str);
    fn publish_named(&self, topic: &Topic, event: &Evt, origin: &str);
    fn publish(&self, topic: &Topic, event: &Evt);
}

/// Struct representing an observable subject.
pub struct Subject<Topic, Evt> {
    rwlock: RwLock<()>,
    subscribers: MultiMap<Topic, Arc<dyn Observer<Topic, Evt> + Send + Sync>>,
    handlers: MultiMap<Topic, (Arc<LooseCoupledHandler<Topic, Evt>>, String)>,
    name: String,
}

/// Implementation of the Subject methods.
impl<Topic: Eq + Hash + Clone, Evt> Subject<Topic, Evt> {
    /// Creates a new SyncSubject.
    pub fn new(name: &str) -> Self {
        Subject {
            rwlock: RwLock::new(()),
            subscribers: MultiMap::new(),
            handlers: MultiMap::new(),
            name: name.to_string(),
        }
    }
}

impl<Topic: Eq + Hash + Clone, Evt> SubjectTrait<Topic, Evt> for Subject<Topic, Evt> {
    /// Subscribes an observer to a topic.
    fn subscribe(&mut self, topic: &Topic, observer: Arc<dyn Observer<Topic, Evt> + Send + Sync>) {
        let _lock = self.rwlock.write().unwrap();
        self.subscribers.insert(topic.clone(), observer);
    }

    /// Unsubscribes a single observer instance from a topic (keeps other observers for the same topic).
    fn unsubscribe(
        &mut self,
        topic: &Topic,
        observer: &Arc<dyn Observer<Topic, Evt> + Send + Sync>,
    ) {
        let _lock = self.rwlock.write().unwrap();
        if let Some(observers) = self.subscribers.get_vec(topic) {
            // Keep only those observers that are NOT the one to remove.
            let retained: Vec<Arc<dyn Observer<Topic, Evt> + Send + Sync>> = observers
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
    fn subscribe_handler(
        &mut self,
        topic: &Topic,
        handler: Arc<LooseCoupledHandler<Topic, Evt>>,
        handler_name: &str,
    ) {
        let _lock = self.rwlock.write().unwrap();
        self.handlers
            .insert(topic.clone(), (handler, handler_name.to_string()));
    }

    /// Unsubscribes a loose-coupled handler from a topic by name.
    fn unsubscribe_handler(&mut self, topic: &Topic, handler_name: &str) {
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
    fn publish_named(&self, topic: &Topic, event: &Evt, origin: &str) {
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
    fn publish(&self, topic: &Topic, event: &Evt) {
        self.publish_named(topic, event, &self.name);
    }
}

// Unit tests for Subject.
#[cfg(test)]
mod tests {
    use super::{Observer, Subject, SubjectTrait};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    struct TestTopic {
        id: u32,
    }
    #[derive(Debug, PartialEq, Clone)]
    struct TestEvent {
        message: String,
    }
    struct TestObserver {
        received: Arc<Mutex<Vec<(TestTopic, TestEvent, String)>>>,
    }
    impl Observer<TestTopic, TestEvent> for TestObserver {
        fn inform(&self, topic: &TestTopic, event: &TestEvent, origin: &str) {
            let mut guard = self.received.lock().unwrap();
            guard.push((topic.clone(), event.clone(), origin.to_string()));
        }
    }

    // basic test for subscribe and publish
    #[test]
    fn test_subscribe_publish() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let observer = Arc::new(TestObserver {
            received: received.clone(),
        });
        let mut subject: Subject<TestTopic, TestEvent> = Subject::new("TestSubject");
        let topic = TestTopic { id: 1 };
        let event = TestEvent {
            message: "Hello".to_string(),
        };
        subject.subscribe(&topic, observer.clone());
        subject.publish(&topic, &event);
        let guard = received.lock().unwrap();
        assert_eq!(
            guard.as_slice(),
            &[(topic.clone(), event.clone(), "TestSubject".to_string())]
        );
    }

    // basic test for unsubscribe
    #[test]
    fn test_unsubscribe() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let observer: Arc<dyn Observer<TestTopic, TestEvent> + Send + Sync> =
            Arc::new(TestObserver {
                received: received.clone(),
            });
        let mut subject: Subject<TestTopic, TestEvent> = Subject::new("TestSubject");
        let topic = TestTopic { id: 1 };
        let event = TestEvent {
            message: "Hello".to_string(),
        };
        subject.subscribe(&topic, observer.clone());
        subject.unsubscribe(&topic, &observer);
        subject.publish(&topic, &event);
        let guard = received.lock().unwrap();
        assert!(guard.is_empty());
    }

    // basic test for subscribe_handler and publish
    #[test]
    fn test_subscribe_handler_publish() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let handler_received = received.clone();
        let handler = Arc::new(move |topic: &TestTopic, event: &TestEvent, origin: &str| {
            let mut guard = handler_received.lock().unwrap();
            guard.push((topic.clone(), event.clone(), origin.to_string()));
        });
        let mut subject: Subject<TestTopic, TestEvent> = Subject::new("TestSubject");
        let topic = TestTopic { id: 1 };
        let event = TestEvent {
            message: "Hello".to_string(),
        };
        subject.subscribe_handler(&topic, handler.clone(), "TestHandler");
        subject.publish(&topic, &event);
        let guard = received.lock().unwrap();
        assert_eq!(
            guard.as_slice(),
            &[(topic.clone(), event.clone(), "TestSubject".to_string())]
        );
    }

    // basic test for unsubscribe_handler
    #[test]
    fn test_unsubscribe_handler() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let handler_received = received.clone();
        let handler = Arc::new(move |topic: &TestTopic, event: &TestEvent, origin: &str| {
            let mut guard = handler_received.lock().unwrap();
            guard.push((topic.clone(), event.clone(), origin.to_string()));
        });
        let mut subject: Subject<TestTopic, TestEvent> = Subject::new("TestSubject");
        let topic = TestTopic { id: 1 };
        let event = TestEvent {
            message: "Hello".to_string(),
        };
        subject.subscribe_handler(&topic, handler.clone(), "TestHandler");
        subject.unsubscribe_handler(&topic, "TestHandler");
        subject.publish(&topic, &event);
        let guard = received.lock().unwrap();
        assert!(guard.is_empty());
    }

    // test subscribe with multiple observers on the same topic
    #[test]
    fn test_multiple_observers() {
        let received1 = Arc::new(Mutex::new(Vec::new()));
        let observer1 = Arc::new(TestObserver {
            received: received1.clone(),
        });
        let received2 = Arc::new(Mutex::new(Vec::new()));
        let observer2 = Arc::new(TestObserver {
            received: received2.clone(),
        });
        let mut subject: Subject<TestTopic, TestEvent> = Subject::new("TestSubject");
        let topic = TestTopic { id: 1 };
        let event = TestEvent {
            message: "Hello".to_string(),
        };
        subject.subscribe(&topic, observer1.clone());
        subject.subscribe(&topic, observer2.clone());
        subject.publish(&topic, &event);
        let guard1 = received1.lock().unwrap();
        let guard2 = received2.lock().unwrap();
        assert_eq!(
            guard1.as_slice(),
            &[(topic.clone(), event.clone(), "TestSubject".to_string())]
        );
        assert_eq!(
            guard2.as_slice(),
            &[(topic.clone(), event.clone(), "TestSubject".to_string())]
        );
    }

    // test unsubscribe only one observer when multiple are subscribed to the same topic
    #[test]
    fn test_unsubscribe_one_of_multiple_observers() {
        let received1 = Arc::new(Mutex::new(Vec::new()));
        let observer1: Arc<dyn Observer<TestTopic, TestEvent> + Send + Sync> =
            Arc::new(TestObserver {
                received: received1.clone(),
            });
        let received2 = Arc::new(Mutex::new(Vec::new()));
        let observer2: Arc<dyn Observer<TestTopic, TestEvent> + Send + Sync> =
            Arc::new(TestObserver {
                received: received2.clone(),
            });
        let mut subject: Subject<TestTopic, TestEvent> = Subject::new("TestSubject");
        let topic = TestTopic { id: 1 };
        let event = TestEvent {
            message: "Hello".to_string(),
        };
        subject.subscribe(&topic, observer1.clone());
        subject.subscribe(&topic, observer2.clone());
        subject.unsubscribe(&topic, &observer1);
        subject.publish(&topic, &event);
        let guard1 = received1.lock().unwrap();
        let guard2 = received2.lock().unwrap();
        assert!(guard1.is_empty());
        assert_eq!(
            guard2.as_slice(),
            &[(topic.clone(), event.clone(), "TestSubject".to_string())]
        );
    }

    // test unsubscribe handler when multiple handlers are subscribed to the same topic
    #[test]
    fn test_unsubscribe_one_of_multiple_handlers() {
        let received1 = Arc::new(Mutex::new(Vec::new()));
        let handler_received1 = received1.clone();
        let handler1 = Arc::new(move |topic: &TestTopic, event: &TestEvent, origin: &str| {
            let mut guard = handler_received1.lock().unwrap();
            guard.push((topic.clone(), event.clone(), origin.to_string()));
        });
        let received2 = Arc::new(Mutex::new(Vec::new()));
        let handler_received2 = received2.clone();
        let handler2 = Arc::new(move |topic: &TestTopic, event: &TestEvent, origin: &str| {
            let mut guard = handler_received2.lock().unwrap();
            guard.push((topic.clone(), event.clone(), origin.to_string()));
        });
        let mut subject: Subject<TestTopic, TestEvent> = Subject::new("TestSubject");
        let topic = TestTopic { id: 1 };
        let event = TestEvent {
            message: "Hello".to_string(),
        };
        subject.subscribe_handler(&topic, handler1.clone(), "Handler1");
        subject.subscribe_handler(&topic, handler2.clone(), "Handler2");
        subject.unsubscribe_handler(&topic, "Handler1");
        subject.publish(&topic, &event);
        let guard1 = received1.lock().unwrap();
        let guard2 = received2.lock().unwrap();
        assert!(guard1.is_empty());
        assert_eq!(
            guard2.as_slice(),
            &[(topic.clone(), event.clone(), "TestSubject".to_string())]
        );
    }

    // test publish to a topic with no subscribers
    #[test]
    fn test_publish_no_subscribers() {
        let subject: Subject<TestTopic, TestEvent> = Subject::new("TestSubject");
        let topic = TestTopic { id: 1 };
        let event = TestEvent {
            message: "Hello".to_string(),
        };
        // Should not panic
        subject.publish(&topic, &event);
    }
}
