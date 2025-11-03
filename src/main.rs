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

use publish_subscribe_rs::tools::async_observer;
use publish_subscribe_rs::tools::histogram;
use publish_subscribe_rs::tools::periodic_task;
use publish_subscribe_rs::tools::sync_dictionary;
use publish_subscribe_rs::tools::sync_object;
use publish_subscribe_rs::tools::sync_observer;
use publish_subscribe_rs::tools::sync_queue;
use publish_subscribe_rs::tools::worker_task;

type MyContext = String;

fn my_periodic_function(context: std::sync::Arc<MyContext>, task_name: &String) {
    println!(
        "Periodic task '{}' executed with context: {}",
        task_name, context
    );
}

fn main() {
    // Test sync queue
    let sync_queue = sync_queue::SyncQueue::<i32>::new();
    sync_queue.enqueue(10);
    let item = sync_queue.dequeue();
    println!("Dequeued item: {:?}", item);

    // Test sync object
    let mut sync = sync_object::SyncObject::new(false);
    //sync.wait_for_signal();

    sync.wait_for_signal_timeout(1000);

    // Test sync dictionary
    let dict = sync_dictionary::SyncDictionary::<String, i32>::new();
    dict.insert("key1".to_string(), 42);
    if let Some(value) = dict.get(&"key1".to_string()) {
        println!("Value for 'key1': {}", value);
    }

    // Test histogram
    let mut hist: histogram::Histogram<i32> = histogram::Histogram::<i32>::new();
    hist.add(42);
    if let Some((top_value, count)) = hist.top_value_with_count() {
        println!("Top occurrence: value = {}, count = {}", top_value, count);
    } else {
        println!("No occurrences in histogram.");
    }

    // Test histogram with String type
    let mut hist: histogram::Histogram<String> = histogram::Histogram::<String>::new();
    hist.add("hello".to_string());
    if let Some((top_value, count)) = hist.top_value_with_count() {
        println!("Top occurrence: value = {}, count = {}", top_value, count);
    } else {
        println!("No occurrences in histogram.");
    }

    // Test periodic task with a function pointer
    {
        let context = std::sync::Arc::new("My periodic task context".to_string());
        let mut task = periodic_task::PeriodicTask::new(
            "MyPeriodicTask".to_string(),
            std::sync::Arc::new(my_periodic_function),
            1000,
            context.clone(),
        );

        task.start();

        // Let the periodic task run for a few seconds
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // Test periodic task with a closure
    {
        let context_closure = std::sync::Arc::new("Closure context".to_string());
        let mut task_closure = periodic_task::PeriodicTask::new(
            "ClosurePeriodicTask".to_string(),
            std::sync::Arc::new(|ctx: std::sync::Arc<MyContext>, task_name: &String| {
                println!(
                    "Periodic task '{}' executed with context: {}",
                    task_name, ctx
                );
            }),
            1500,
            context_closure.clone(),
        );

        task_closure.start();

        // Let the periodic task with closure run for a few seconds
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // Test worker task
    {
        let context = std::sync::Arc::new("My worker task context".to_string());
        let mut worker_task =
            worker_task::WorkerTask::new("MyWorkerTask".to_string(), context.clone());

        worker_task.start();

        worker_task.delegate(std::sync::Arc::new(
            |ctx: std::sync::Arc<MyContext>, task_name: &String| {
                println!("Worker task '{}' executed with context: {}", task_name, ctx);
            },
        ));

        worker_task.delegate(std::sync::Arc::new(
            |ctx: std::sync::Arc<MyContext>, task_name: &String| {
                println!(
                    "Another worker task '{}' executed with context: {}",
                    task_name, ctx
                );
            },
        ));

        // Let the worker task run for a few seconds
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    // Test sync observer and subject
    {
        use sync_observer::{SyncObserver, SyncSubject};

        struct MyObserver;

        impl SyncObserver<String, String> for MyObserver {
            fn inform(&self, topic: &String, event: &String, origin: &str) {
                println!(
                    "Observer informed - Topic: {}, Event: {}, Origin: {}",
                    topic, event, origin
                );
            }
        }

        let mut subject = SyncSubject::<String, String>::new("MySyncSubject");

        let observer = std::sync::Arc::new(MyObserver);
        subject.subscribe("TestTopic".to_string(), observer.clone());

        // subscribe a closure as a loose-coupled handler
        subject.subscribe_handler(
            "TestTopic".to_string(),
            std::sync::Arc::new(|topic: &String, event: &String, origin: &str| {
                println!(
                    "Loose-coupled handler - Topic: {}, Event: {}, Origin: {}",
                    topic, event, origin
                );
            }),
            "MyHandler",
        );

        subject.publish(&"TestTopic".to_string(), &"TestEvent".to_string());
    }

    // Test async observer in a periodic task
    {
        use async_observer::AsyncObserver;
        use periodic_task::PeriodicTask;
        use sync_observer::{SyncObserver, SyncSubject};

        // Define a struct that implements SyncObserver and contains an AsyncObserver
        struct MyAsyncObserver {
            observer: std::sync::Arc<AsyncObserver<String, String>>,
            task: PeriodicTask<String>,
        }

        // Implement methods for MyAsyncObserver
        impl MyAsyncObserver {
            /// Creates a new MyAsyncObserver with an internal AsyncObserver and a periodic task
            fn new() -> Self {
                // Create the observer inside an Arc so the periodic task closure can clone it
                let observer = std::sync::Arc::new(AsyncObserver::new());
                let observer_clone = observer.clone();

                // Create the periodic task that will process events
                let task = PeriodicTask::new(
                    "AsyncObserverTask".to_string(),
                    std::sync::Arc::new(
                        move |_ctx: std::sync::Arc<MyContext>, _task_name: &String| {
                            // Process events directly using the captured observer clone
                            observer_clone.wait_for_events(500);

                            // Process all available events
                            if observer_clone.has_events() {
                                // Pop and process all events
                                let to_process = observer_clone.pop_all_events();

                                // Process each event
                                for (topic, event, origin) in to_process {
                                    println!(
                                        "Async Observer processed - Topic: {}, Event: {}, Origin: {}",
                                        topic, event, origin
                                    );
                                }
                            }
                        },
                    ),
                    1000,
                    std::sync::Arc::new("".to_string()),
                );

                MyAsyncObserver { observer, task }
            }

            /// Starts the internal periodic task
            fn start(&mut self) {
                self.task.start();
            }
        }

        // Implement SyncObserver for MyAsyncObserver by delegating to the internal AsyncObserver
        impl SyncObserver<String, String> for MyAsyncObserver {
            fn inform(&self, topic: &String, event: &String, origin: &str) {
                self.observer.inform(topic, event, origin);
            }
        }

        let mut subject = SyncSubject::<String, String>::new("MySubject");

        let mut async_observer = std::sync::Arc::new(MyAsyncObserver::new());

        // Start the internal periodic task
        std::sync::Arc::get_mut(&mut async_observer)
            .unwrap()
            .start();

        subject.subscribe("TestTopic".to_string(), async_observer.clone());

        // subscribe a "debug" closure as a loose-coupled handler
        subject.subscribe_handler(
            "TestTopic".to_string(),
            std::sync::Arc::new(|topic: &String, event: &String, origin: &str| {
                println!(
                    "Loose-coupled handler - Topic: {}, Event: {}, Origin: {}",
                    topic, event, origin
                );
            }),
            "MyHandler",
        );

        subject.publish(&"TestTopic".to_string(), &"TestEvent1".to_string());
        subject.publish(&"TestTopic".to_string(), &"TestEvent2".to_string());
        subject.publish(&"TestTopic".to_string(), &"TestEvent3".to_string());

        // Let the periodic task with closure run for a few seconds
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
