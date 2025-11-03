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

use publish_subscribe_rs::tools::histogram;
use publish_subscribe_rs::tools::periodic_task;
use publish_subscribe_rs::tools::sync_dictionary;
use publish_subscribe_rs::tools::sync_object;
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
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
