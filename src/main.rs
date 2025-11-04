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
use std::time::Duration;

use pubsub_rs::tools::async_observer::AsyncObserver;
use pubsub_rs::tools::histogram::Histogram;
use pubsub_rs::tools::periodic_task::PeriodicTask;
use pubsub_rs::tools::sync_dictionary::SyncDictionary;
use pubsub_rs::tools::sync_object::SyncObject;
use pubsub_rs::tools::sync_observer::{SyncObserver, SyncSubject};
use pubsub_rs::tools::sync_queue::SyncQueue;
use pubsub_rs::tools::worker_task::WorkerTask;

type MyContext = String;

/// A sample periodic function to be used with the PeriodicTask.
fn my_periodic_function(context: Arc<MyContext>, task_name: &String) {
    println!(
        "Periodic task '{}' executed with context: {}",
        task_name, context
    );
}
/// A function generating an array of random i32 following a Gaussian Distribution with given mean and standard deviation.
fn generate_gaussian_samples(mean: f64, std_dev: f64, count: usize) -> Vec<i32> {
    let mut samples = Vec::with_capacity(count);

    for _ in 0..count {
        // Box-Muller transform using rand::random() to avoid the `gen` keyword conflict
        let u1: f64 = rand::random();
        let u2: f64 = rand::random();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let sample = (z0 * std_dev + mean).round() as i32;
        samples.push(sample);
    }

    samples
}
/// Main entry
fn main() {
    // Test sync queue
    let sync_queue = SyncQueue::<i32>::new();
    sync_queue.enqueue(10);
    sync_queue.enqueue(20);
    sync_queue.enqueue(30);
    let item = sync_queue.dequeue();
    println!("Dequeued item: {:?}", item);
    let item = sync_queue.dequeue();
    println!("Dequeued item: {:?}", item);
    let item = sync_queue.dequeue();
    println!("Dequeued item: {:?}", item);

    // Test sync object
    let mut sync = SyncObject::new(false);
    //sync.wait_for_signal();

    sync.wait_for_signal_timeout(1000);

    // Test sync dictionary
    let dict = SyncDictionary::<String, i32>::new();
    dict.insert("key1".to_string(), 42);
    dict.insert("key2".to_string(), 100);
    dict.insert("key3".to_string(), 7);
    if let Some(value) = dict.get(&"key1".to_string()) {
        println!("Value for 'key1': {}", value);
    }
    if let Some(value) = dict.get(&"key2".to_string()) {
        println!("Value for 'key2': {}", value);
    }
    if let Some(value) = dict.get(&"key3".to_string()) {
        println!("Value for 'key3': {}", value);
    }

    // Test histogram
    let mut hist = Histogram::<u64>::new();
    hist.add(42);
    hist.add(42);
    hist.add(7);
    hist.add(100);

    if let Some((top_value, count)) = hist.top_value_with_count() {
        println!("Top occurrence: value = {}, count = {}", top_value, count);
    } else {
        println!("No occurrences in histogram.");
    }

    hist.clear();

    println!("Generating Gaussian samples (mean 50.0, std_dev 5.0)...");

    let mut hist = Histogram::<i32>::new();

    // Generate Gaussian samples and add them to the histogram
    let samples = generate_gaussian_samples(50.0, 5.0, 100000);
    for sample in samples {
        hist.add(sample);
    }

    if let Some((top_value, count)) = hist.top_value_with_count() {
        println!("Top occurrence: value = {}, count = {}", top_value, count);
    } else {
        println!("No occurrences in histogram.");
    }

    let average = hist.average();
    println!("Average value: {}", average);
    println!("Total count: {}", hist.total_count());
    let variance = hist.variance(average);
    println!("Variance: {}", variance);
    let std_dev = hist.standard_deviation(variance);
    println!("Standard deviation: {}", std_dev);
    println!("Median: {}", hist.median());
    println!(
        "Gaussian probability between 45 and 55: {} should be around 68.27%",
        hist.gaussian_probability_between(45.0, 55.0, average, std_dev, 10000)
    );
    println!(
        "Gaussian probability between 40 and 45: {} should be around 13.59%",
        hist.gaussian_probability_between(40.0, 45.0, average, std_dev, 10000)
    );
    println!(
        "Gaussian probability between 60 and 65: {} should be around 2.28%",
        hist.gaussian_probability_between(60.0, 65.0, average, std_dev, 10000)
    );
    println!(
        "Gaussian probability between 40 and 60: {} should be around 95.45%",
        hist.gaussian_probability_between(40.0, 60.0, average, std_dev, 10000)
    );
    println!(
        "Gaussian probability between 35 and 65: {} should be around 99.73%",
        hist.gaussian_probability_between(35.0, 65.0, average, std_dev, 10000)
    );

    // Test histogram with String type
    let mut hist: Histogram<String> = Histogram::<String>::new();
    hist.add("hello".to_string());
    hist.add("hello".to_string());
    hist.add("world".to_string());
    hist.add("rust".to_string());

    if let Some((top_value, count)) = hist.top_value_with_count() {
        println!("Top occurrence: value = {}, count = {}", top_value, count);
    } else {
        println!("No occurrences in histogram.");
    }

    // Test periodic task with a function pointer
    {
        let context = Arc::new("My periodic task context".to_string());
        let mut task = PeriodicTask::new(
            "MyPeriodicTask".to_string(),
            Arc::new(my_periodic_function),
            1000,
            context.clone(),
        );

        task.start();

        // Let the periodic task run for a few seconds
        std::thread::sleep(Duration::from_secs(5));
    }

    // Test periodic task with a closure
    {
        let context_closure = Arc::new("Closure context".to_string());
        let mut task_closure = PeriodicTask::new(
            "ClosurePeriodicTask".to_string(),
            Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
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
        std::thread::sleep(Duration::from_secs(5));
    }

    // Test worker task
    {
        let context = Arc::new("My worker task context".to_string());
        let mut worker_task =
            WorkerTask::new("MyWorkerTask".to_string(), context.clone());

        worker_task.start();

        worker_task.delegate(Arc::new(
            |ctx: Arc<MyContext>, task_name: &String| {
                println!("Worker task '{}' executed with context: {}", task_name, ctx);
            },
        ));

        worker_task.delegate(Arc::new(
            |ctx: Arc<MyContext>, task_name: &String| {
                println!(
                    "Another worker task '{}' executed with context: {}",
                    task_name, ctx
                );
            },
        ));

        // Let the worker task run for a few seconds
        std::thread::sleep(Duration::from_secs(2));

        worker_task.delegate(Arc::new(
            |ctx: Arc<MyContext>, task_name: &String| {
                println!(
                    "Yet another worker task '{}' executed with context: {}",
                    task_name, ctx
                );
            },
        ));

        // Let the worker task run for a few seconds
        std::thread::sleep(Duration::from_secs(1));        
        
        worker_task.delegate(Arc::new(
            |ctx: Arc<MyContext>, task_name: &String| {
                println!(
                    "And a last one '{}' executed with context: {}",
                    task_name, ctx
                );
            },
        ));        

        // Let the worker task run for a few seconds
        std::thread::sleep(Duration::from_secs(2));
    }

    // Test sync observer and subject
    {
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

        let observer = Arc::new(MyObserver);
        subject.subscribe("TestTopic".to_string(), observer.clone());

        // subscribe a closure as a loose-coupled handler
        subject.subscribe_handler(
            "TestTopic".to_string(),
            Arc::new(|topic: &String, event: &String, origin: &str| {
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
        // Define a struct that implements SyncObserver and contains an AsyncObserver
        struct MyAsyncObserver {
            observer: Arc<AsyncObserver<String, String>>,
            task: PeriodicTask<String>,
        }

        // Implement methods for MyAsyncObserver
        impl MyAsyncObserver {
            /// Creates a new MyAsyncObserver with an internal AsyncObserver and a periodic task
            fn new() -> Self {
                // Create the observer inside an Arc so the periodic task closure can clone it
                let observer = Arc::new(AsyncObserver::new());
                // Clone the observer (shared ptr) for use in the periodic task
                let observer_clone = observer.clone();

                // Create the periodic task that will process events
                let task = PeriodicTask::new(
                    "AsyncObserverTask".to_string(),
                    Arc::new(
                        move |_ctx: Arc<MyContext>, _task_name: &String| {
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
                    Arc::new("".to_string()),
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

        let mut async_observer = Arc::new(MyAsyncObserver::new());

        // Start the internal periodic task
        Arc::get_mut(&mut async_observer)
            .unwrap()
            .start();

        subject.subscribe("TestTopic".to_string(), async_observer.clone());

        // subscribe a "debug" closure as a loose-coupled handler
        subject.subscribe_handler(
            "TestTopic".to_string(),
            Arc::new(|topic: &String, event: &String, origin: &str| {
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
        std::thread::sleep(Duration::from_secs(2));

        subject.publish(&"TestTopic".to_string(), &"TestEvent4".to_string());
        subject.publish(&"TestTopic".to_string(), &"TestEvent5".to_string());
        subject.publish(&"TestTopic".to_string(), &"TestEvent6".to_string());

        // Let the periodic task with closure run for a few seconds
        std::thread::sleep(Duration::from_secs(2));

        subject.publish(&"TestTopic".to_string(), &"TestEvent7".to_string());
        subject.publish(&"TestTopic".to_string(), &"TestEvent8".to_string());
        subject.publish(&"TestTopic".to_string(), &"TestEvent9".to_string());

        // Let the periodic task with closure run for a few seconds
        std::thread::sleep(Duration::from_secs(2));
    }
}
