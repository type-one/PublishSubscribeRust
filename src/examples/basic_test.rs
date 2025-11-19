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

//! Basic test example for the Publish/Subscribe Rust library.
//! This example demonstrates the basic usage of the library.
//! It uses the different helpers, and creates a publisher and a subscriber,
//! showing how they can communicate with each other.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::tools::async_observer::AsyncObserver;
use crate::tools::data_task::DataTask;
use crate::tools::histogram::Histogram;
use crate::tools::lock_free_ring_buffer::LockFreeRingBuffer;
use crate::tools::periodic_task::PeriodicTask;
use crate::tools::sync_dictionary::SyncDictionary;
use crate::tools::sync_object::SyncObject;
use crate::tools::sync_observer::{Observer, Subject, SubjectTrait};
use crate::tools::sync_queue::SyncQueue;
use crate::tools::task_trait::TaskTrait;
use crate::tools::worker_pool::WorkerPool;
use crate::tools::worker_task::WorkerTask;
use crate::tools::worker_trait::WorkerTrait;

/// An enum representing different types of values to be stored in the SyncDictionary.
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum ValueType {
    Int(i32),
    Float(f64),
    Text(String),
}
/// A sample context struct to be used with the tasks.
///
/// The idea is to share several structures between tasks, such as environment variables or pub/sub interfaces
struct MyContext {
    info: String,
    data: Arc<Mutex<Vec<i32>>>,
    variables: Arc<Mutex<SyncDictionary<String, ValueType>>>,
}

impl Clone for MyContext {
    fn clone(&self) -> Self {
        MyContext {
            info: self.info.clone(),
            data: self.data.clone(),
            variables: self.variables.clone(),
        }
    }
}

/// A sample periodic function to be used with the PeriodicTask.
fn my_periodic_function(context: Arc<MyContext>, task_name: &String) {
    println!(
        "Periodic task '{}' executed with context: {:?}",
        task_name, context.info
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

/// Test function for SyncQueue.
fn test_sync_queue() {
    println!("Testing SyncQueue...");
    println!("-----------------------------------------------");

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
    println!("-----------------------------------------------");
}

/// Test function for SyncObject.
fn test_sync_object() {
    println!("Testing SyncObject...");
    println!("-----------------------------------------------");

    let sync = SyncObject::new();
    // In a real test, you would spawn a thread to signal the object after some time.
    // Here we just demonstrate the wait_for_signal_timeout method.
    sync.wait_for_signal_timeout(1000);
    println!("-----------------------------------------------");
}

/// Test function for SyncObject with signaling from another thread.
fn test_sync_object_signal() {
    println!("Testing SyncObject with signal...");
    println!("-----------------------------------------------");

    let sync = Arc::new(SyncObject::new());
    let child_sync = sync.clone();

    // Spawn a thread to signal after 500ms
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        child_sync.signal();
        println!("Signal sent from thread.");
    });

    // Wait for the signal
    sync.wait_for_signal();
    println!("Wait for signal completed.");
    println!("-----------------------------------------------");
}

/// Test function for SyncDictionary.
fn test_sync_dictionary() {
    println!("Testing SyncDictionary...");
    println!("-----------------------------------------------");

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
    println!("-----------------------------------------------");
}

/// Test function for Histogram.
fn test_histogram() {
    println!("Testing Histogram...");
    println!("-----------------------------------------------");

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
    println!("-----------------------------------------------");
}

/// Test function for Histogram with String values.
fn test_histogram_string() {
    println!("Testing Histogram with String values...");
    println!("-----------------------------------------------");

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

    hist.clear();
    println!("-----------------------------------------------");
}

/// Test function for Histogram and statistical metrics.
fn test_histogram_and_metrics() {
    println!("Testing Histogram and Metrics...");
    println!("-----------------------------------------------");

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

    println!("-----------------------------------------------");
}

/// Test function for SyncQueue with a child thread.
fn test_sync_queue_thread() {
    println!("Testing SyncQueue with a child thread...");
    println!("-----------------------------------------------");

    let sync_queue = Arc::new(SyncQueue::<i32>::new());

    // Spawn a producer thread
    let producer_handle = std::thread::spawn({
        let sync_queue = sync_queue.clone();
        move || {
            for i in 0..10 {
                sync_queue.enqueue(i);
                println!("SyncQueue Enqueued: {}", i);
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    });

    // Consumer in the main thread
    for _ in 0..10 {
        let item = loop {
            if let Some(value) = sync_queue.dequeue() {
                break value;
            } else {
                // Queue is empty, retry
                std::thread::yield_now();
            }
        };
        println!("SyncQueue Dequeued: {}", item);
        std::thread::sleep(Duration::from_millis(150));
    }

    producer_handle.join().unwrap();
    println!("-----------------------------------------------");
}

/// Test function for LockFreeRingBuffer with a child thread.
fn test_lock_free_ring_buffer_thread() {
    println!("Testing LockFreeRingBuffer with a child thread...");
    println!("-------------------------------------------------");

    const RING_BUFFER_POW2N: usize = 4; // 16 elements
    let ring_buffer = Arc::new(LockFreeRingBuffer::<f64, RING_BUFFER_POW2N>::new());

    // Spawn a producer thread
    let producer_handle = std::thread::spawn({
        let ring_buffer = ring_buffer.clone();
        move || {
            for i in 0..20 {
                loop {
                    if ring_buffer.enqueue(i as f64).is_ok() {
                        println!("LockFreeRingBuffer Enqueued: {}", i);
                        break;
                    } else {
                        // Buffer is full, retry
                        std::thread::yield_now();
                    }
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    });

    // Consumer in the main thread
    for _ in 0..20 {
        loop {
            if let Some(value) = ring_buffer.dequeue() {
                println!("LockFreeRingBuffer Dequeued: {}", value);
                break;
            } else {
                // Buffer is empty, retry
                std::thread::yield_now();
            }
        }
        std::thread::sleep(Duration::from_millis(150));
    }

    producer_handle.join().unwrap();
    println!("-----------------------------------------------");
}

/// Test function for PeriodicTask with a function.
fn test_periodic_task_function() {
    println!("Testing PeriodicTask with function...");
    println!("-----------------------------------------------");

    let context = Arc::new(MyContext {
        info: "My periodic task context".to_string(),
        data: Arc::new(Mutex::new(vec![1, 2, 3])),
        variables: Arc::new(Mutex::new(SyncDictionary::new())),
    });

    let mut task = PeriodicTask::new(
        context.clone(),
        "MyPeriodicTask".to_string(),
        Arc::new(my_periodic_function),
        1000,
    );

    task.start();

    // Let the periodic task run for a few seconds
    std::thread::sleep(Duration::from_secs(5));
    println!("-----------------------------------------------");
}

/// Test function for PeriodicTask with a closure.
fn test_periodic_task_closure() {
    println!("Testing PeriodicTask with closure...");
    println!("-----------------------------------------------");

    let context_closure = Arc::new(MyContext {
        info: "Closure periodic task context".to_string(),
        data: Arc::new(Mutex::new(vec![4, 5, 6])),
        variables: Arc::new(Mutex::new(SyncDictionary::new())),
    });

    let mut task_closure = PeriodicTask::new(
        context_closure.clone(),
        "ClosurePeriodicTask".to_string(),
        Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
            println!(
                "Periodic task '{}' executed with context: {}",
                task_name, ctx.info
            );
        }),
        1500,
    );

    task_closure.start();

    // Let the periodic task with closure run for a few seconds
    std::thread::sleep(Duration::from_secs(5));
    println!("-----------------------------------------------");
}

/// Test function for WorkerTask.
fn test_worker_task() {
    println!("Testing WorkerTask...");
    println!("-----------------------------------------------");

    let context = Arc::new(MyContext {
        info: "My worker task context".to_string(),
        data: Arc::new(Mutex::new(vec![7, 8, 9])),
        variables: Arc::new(Mutex::new(SyncDictionary::new())),
    });

    let mut worker_task = WorkerTask::new(context.clone(), "MyWorkerTask".to_string());

    worker_task.start();

    worker_task.delegate(Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
        println!(
            "Worker task '{}' executed with context: {}",
            task_name, ctx.info
        );
    }));

    worker_task.delegate(Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
        println!(
            "Another worker task '{}' executed with context: {}",
            task_name, ctx.info
        );
    }));

    // Let the worker task run for a few seconds
    std::thread::sleep(Duration::from_secs(2));

    worker_task.delegate(Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
        println!(
            "Yet another worker task '{}' executed with context: {}",
            task_name, ctx.info
        );
    }));

    // Let the worker task run for a few seconds
    std::thread::sleep(Duration::from_secs(1));

    worker_task.delegate(Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
        println!(
            "And a last one '{}' executed with context: {}",
            task_name, ctx.info
        );
    }));

    // Let the worker task run for a few seconds
    std::thread::sleep(Duration::from_secs(2));
    println!("-----------------------------------------------");
}

/// Test function for WorkerPool.
fn test_workers_pool() {
    println!("Testing WorkerPool...");
    println!("-----------------------------------------------");

    let context = Arc::new(MyContext {
        info: "My worker pool context".to_string(),
        data: Arc::new(Mutex::new(vec![10, 11, 12])),
        variables: Arc::new(Mutex::new(SyncDictionary::new())),
    });

    let mut workers_pool = WorkerPool::new(context.clone());

    workers_pool.start();

    workers_pool.delegate(Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
        println!(
            "Worker pool task '{}' executed with context: {}",
            task_name, ctx.info
        );
    }));

    workers_pool.delegate(Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
        println!(
            "Another worker pool task '{}' executed with context: {}",
            task_name, ctx.info
        );
    }));

    // Let the workers pool run for a few seconds
    std::thread::sleep(Duration::from_secs(2));

    workers_pool.delegate(Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
        println!(
            "Yet another worker pool task '{}' executed with context: {}",
            task_name, ctx.info
        );
    }));

    // Let the worker task run for a few seconds
    std::thread::sleep(Duration::from_secs(1));

    workers_pool.delegate(Arc::new(|ctx: Arc<MyContext>, task_name: &String| {
        println!(
            "And a last one '{}' executed with context: {}",
            task_name, ctx.info
        );
    }));

    // Let the worker task run for a few seconds
    std::thread::sleep(Duration::from_secs(2));
    println!("-----------------------------------------------");
}

/// Test function for DataTask with scalar data.
fn test_data_task_scalar() {
    println!("Testing DataTask with scalar data...");
    println!("-----------------------------------------------");

    let context = Arc::new(MyContext {
        info: "My data task context".to_string(),
        data: Arc::new(Mutex::new(vec![13, 14, 15])),
        variables: Arc::new(Mutex::new(SyncDictionary::new())),
    });

    let mut data_task = DataTask::<MyContext, i32>::new(
        context.clone(),
        "MyDataTask".to_string(),
        Arc::new(|ctx: Arc<MyContext>, task_name: &String, data: i32| {
            println!(
                "Data task '{}' executed with context: {}, data: {}",
                task_name, ctx.info, data
            );
        }),
    );

    data_task.start();

    data_task.submit(100);
    data_task.submit(200);
    data_task.submit(300);

    // Let the data task run for a few seconds
    std::thread::sleep(Duration::from_secs(2));
    println!("-----------------------------------------------");
}

/// Test function for DataTask with array data.
fn test_data_task_array() {
    println!("Testing DataTask with array data...");
    println!("-----------------------------------------------");

    let context = Arc::new(MyContext {
        info: "My data task with array context".to_string(),
        data: Arc::new(Mutex::new(vec![16, 17, 18])),
        variables: Arc::new(Mutex::new(SyncDictionary::new())),
    });

    let mut data_task = DataTask::<MyContext, Vec<i32>>::new(
        context.clone(),
        "MyDataTaskArray".to_string(),
        Arc::new(|ctx: Arc<MyContext>, task_name: &String, data: Vec<i32>| {
            println!(
                "Data task '{}' executed with context: {}, data: {:?}",
                task_name, ctx.info, data
            );
        }),
    );

    data_task.start();

    data_task.submit(vec![1, 2, 3]);
    data_task.submit(vec![4, 5, 6]);
    data_task.submit(vec![7, 8, 9]);

    // Let the data task run for a few seconds
    std::thread::sleep(Duration::from_secs(2));
    println!("-----------------------------------------------");
}

/// Test function for SyncObserver and SyncSubject.
fn test_sync_observer_and_subject() {
    println!("Testing SyncObserver and SyncSubject...");
    println!("-----------------------------------------------");

    struct MyObserver;

    impl Observer<String, String> for MyObserver {
        fn inform(&self, topic: &String, event: &String, origin: &str) {
            println!(
                "Observer informed - Topic: {}, Event: {}, Origin: {}",
                topic, event, origin
            );
        }
    }

    let mut subject = Subject::<String, String>::new("MySyncSubject");

    let observer = Arc::new(MyObserver);
    subject.subscribe(&"TestTopic".to_string(), observer.clone());

    // subscribe a closure as a loose-coupled handler
    subject.subscribe_handler(
        &"TestTopic".to_string(),
        Arc::new(|topic: &String, event: &String, origin: &str| {
            println!(
                "Loose-coupled handler - Topic: {}, Event: {}, Origin: {}",
                topic, event, origin
            );
        }),
        "MyHandler",
    );

    subject.publish(&"TestTopic".to_string(), &"TestEvent".to_string());
    println!("-----------------------------------------------");
}

/// Test function for AsyncObserver in a PeriodicTask.
fn test_async_observer_in_periodic_task() {
    println!("Testing AsyncObserver in PeriodicTask...");
    println!("-----------------------------------------------");

    // Define a struct that implements SyncObserver and contains an AsyncObserver
    struct MyAsyncObserver {
        observer: Arc<AsyncObserver<String, String>>,
        task: PeriodicTask<MyContext>,
    }

    // Implement methods for MyAsyncObserver
    impl MyAsyncObserver {
        /// Creates a new MyAsyncObserver with an internal AsyncObserver and a periodic task
        fn new() -> Self {
            // Create the observer inside an Arc so the periodic task closure can clone it
            let observer = Arc::new(AsyncObserver::new());
            // Clone the observer (shared ptr) for use in the periodic task
            let child_observer = observer.clone();

            // Create the periodic task that will process events
            let task = PeriodicTask::new(
                Arc::new(MyContext {
                    info: "AsyncObserver periodic task context".to_string(),
                    data: Arc::new(Mutex::new(vec![])),
                    variables: Arc::new(Mutex::new(SyncDictionary::new())),
                }),
                "AsyncObserverTask".to_string(),
                Arc::new(move |_ctx: Arc<MyContext>, _task_name: &String| {
                    // Process events directly using the captured observer clone
                    child_observer.wait_for_events(500);

                    // Process all available events
                    if child_observer.has_events() {
                        // Pop and process all events
                        let to_process = child_observer.pop_all_events();

                        // Process each event
                        for (topic, event, origin) in to_process {
                            println!(
                                "Async Observer processed - Topic: {}, Event: {}, Origin: {}",
                                topic, event, origin
                            );
                        }
                    }
                }),
                1000,
            );

            MyAsyncObserver { observer, task }
        }

        /// Starts the internal periodic task
        fn start(&mut self) {
            self.task.start();
        }
    }

    // Implement SyncObserver for MyAsyncObserver by delegating to the internal AsyncObserver
    impl Observer<String, String> for MyAsyncObserver {
        fn inform(&self, topic: &String, event: &String, origin: &str) {
            self.observer.inform(topic, event, origin);
        }
    }

    let mut subject = Subject::<String, String>::new("MySubject");

    let mut async_observer = Arc::new(MyAsyncObserver::new());

    // Start the internal periodic task
    Arc::get_mut(&mut async_observer).unwrap().start();

    subject.subscribe(&"TestTopic".to_string(), async_observer.clone());

    // subscribe a "debug" closure as a loose-coupled handler
    subject.subscribe_handler(
        &"TestTopic".to_string(),
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
    println!("-----------------------------------------------");
}

/// The main basic test function that calls all individual tests.
pub fn basic_test() {
    println!("Starting basic test of synchronization tools...");
    println!("-----------------------------------------------");

    // Test sync queue
    test_sync_queue();

    // Test sync object
    test_sync_object();
    test_sync_object_signal();

    // Test sync dictionary
    test_sync_dictionary();

    // Test histogram
    test_histogram();
    test_histogram_string();
    test_histogram_and_metrics();

    // Test sync queue with main and a child thread
    test_sync_queue_thread();

    // Test lock-free ring buffer with main and a child thread
    test_lock_free_ring_buffer_thread();

    // Test periodic task with a function pointer
    test_periodic_task_function();

    // Test periodic task with a closure
    test_periodic_task_closure();

    // Test worker task
    test_worker_task();

    // Test workers pool
    test_workers_pool();

    // Test data task with simple scalar data
    test_data_task_scalar();

    // Test data task with array of data
    test_data_task_array();

    // Test sync observer and subject
    test_sync_observer_and_subject();

    // Test async observer in a periodic task
    test_async_observer_in_periodic_task();

    println!("Basic test of synchronization tools completed.");
    println!("-----------------------------------------------");
}
