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

//! Demonstrates the containers and helpers added to reach feature parity with
//! the C++ PublishSubscribe framework: `SyncVector`, `SyncPriorityQueue`,
//! `RingBuffer`/`SyncRingBuffer`, `TimeList`/`SyncTimeList`, bounded
//! `AsyncObserver` overflow detection, priority-ordered `AsyncObserver`, and
//! `WorkerPool`/`WorkerTask` async delegation.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::tools::async_observer::AsyncObserver;
use crate::tools::ring_buffer::RingBuffer;
use crate::tools::sync_observer::Observer;
use crate::tools::sync_priority_queue::SyncPriorityQueue;
use crate::tools::sync_ring_buffer::SyncRingBuffer;
use crate::tools::sync_time_list::SyncTimeList;
use crate::tools::sync_vector::SyncVector;
use crate::tools::task_trait::TaskTrait;
use crate::tools::time_list::TimeList;
use crate::tools::worker_pool::WorkerPool;
use crate::tools::worker_task::WorkerTask;

/// Test function for SyncVector (bounded, reject-on-full and overwrite-on-full modes).
fn test_sync_vector() {
    println!("Testing SyncVector...");
    println!("-----------------------------------------------");

    let vector = SyncVector::<i32>::new(3);
    vector.push(1);
    vector.push(2);
    vector.push(3);
    println!("reject mode: push(4) accepted = {}", vector.push(4));

    println!("overwrite mode: evicted oldest = {}", vector.push_overwrite(4));
    println!("contents after overwrite: {:?}", vector.pop_range(3));
    println!("-----------------------------------------------");
}

/// Test function for SyncPriorityQueue, standalone and as an AsyncObserver backend.
fn test_sync_priority_queue() {
    println!("Testing SyncPriorityQueue...");
    println!("-----------------------------------------------");

    let min_heap = SyncPriorityQueue::<i32>::new();
    min_heap.push(5);
    min_heap.push(1);
    min_heap.push(3);
    println!("min-heap pop order:");
    while let Some(value) = min_heap.top_pop() {
        println!("  {value}");
    }

    // async_observer can swap SyncQueue for SyncPriorityQueue transparently.
    let observer: AsyncObserver<String, i32> = AsyncObserver::with_priority();
    observer.inform(&"topic".to_string(), &3, "worker");
    observer.inform(&"topic".to_string(), &1, "worker");
    observer.inform(&"topic".to_string(), &2, "worker");

    println!("async_observer priority order:");
    for (_, value, _) in observer.pop_all_events() {
        println!("  {value}");
    }
    println!("-----------------------------------------------");
}

/// Test function for RingBuffer and SyncRingBuffer (reject vs overwrite modes).
fn test_ring_buffer_and_sync_ring_buffer() {
    println!("Testing RingBuffer and SyncRingBuffer...");
    println!("-----------------------------------------------");

    let mut buffer = RingBuffer::<i32, 4>::new();
    buffer.push_range(vec![1, 2, 3, 4, 5]);
    println!("reject mode contents: {:?}", buffer.pop_range(4));

    let sync_buffer = SyncRingBuffer::<i32, 4>::new();
    let overwrite_result = sync_buffer.push_range_overwrite(vec![1, 2, 3, 4, 5, 6]);
    println!(
        "sync overwrite range inserted={} overwritten={}",
        overwrite_result.inserted, overwrite_result.overwritten
    );
    println!("sync contents (recent history): {:?}", sync_buffer.pop_range(4));
    println!("-----------------------------------------------");
}

/// Test function for TimeList and SyncTimeList (chronological ordering).
fn test_time_list_and_sync_time_list() {
    println!("Testing TimeList and SyncTimeList...");
    println!("-----------------------------------------------");

    let mut list = TimeList::new();
    list.push(300, "three hundred");
    list.push(100, "one hundred");
    list.push(200, "two hundred");
    println!("drain order:");
    while let Some((timestamp, value)) = list.top_pop() {
        println!("  {timestamp} => {value}");
    }

    let base_time = Instant::now();
    let sync_list = SyncTimeList::new();
    sync_list.push(base_time + Duration::from_millis(30), "c");
    sync_list.push(base_time + Duration::from_millis(10), "a");
    sync_list.push(base_time + Duration::from_millis(20), "b");
    println!("sync drain order (values should be a, b, c):");
    while let Some((_, value)) = sync_list.top_pop() {
        println!("  {value}");
    }
    println!("-----------------------------------------------");
}

/// Test function for bounded AsyncObserver queue overflow detection.
fn test_async_observer_queue_overflow() {
    println!("Testing AsyncObserver queue overflow...");
    println!("-----------------------------------------------");

    let observer: AsyncObserver<String, String> = AsyncObserver::with_capacity(2);
    observer.inform(&"topic".to_string(), &"event-1".to_string(), "producer");
    observer.inform(&"topic".to_string(), &"event-2".to_string(), "producer");
    observer.inform(
        &"topic".to_string(),
        &"event-3-dropped".to_string(),
        "producer",
    );

    println!(
        "overflow detected = {}, dropped = {}",
        observer.has_queue_overflow(),
        observer.queue_overflow_count()
    );
    println!(
        "consumed dropped count = {}, overflow pending = {}",
        observer.consume_queue_overflow_count(),
        observer.has_queue_overflow()
    );
    println!("queued events = {}", observer.pop_all_events().len());
    println!("-----------------------------------------------");
}

/// Test function for WorkerPool/WorkerTask async delegation (request/response style jobs).
fn test_worker_delegate_async() {
    println!("Testing WorkerPool/WorkerTask delegate_async...");
    println!("-----------------------------------------------");

    struct EmptyContext;
    let context = Arc::new(EmptyContext);

    let mut pool = WorkerPool::new(context.clone());
    pool.start();
    let pool_handle = pool
        .delegate_async(|_ctx, _task_name| 6 * 7)
        .expect("pool should be started");
    let pool_result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(pool_handle)
        .expect("pool task should not panic");
    println!("worker pool delegate_async result = {pool_result}");
    pool.stop();

    let mut task = WorkerTask::new(context, "AsyncDemoWorker".to_string());
    task.start();
    let receiver = task.delegate_async(|_ctx, _task_name| 6 * 7);
    let task_result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(receiver)
        .expect("worker task should send a result");
    println!("worker task delegate_async result = {task_result}");
    task.stop();
    println!("-----------------------------------------------");
}

/// Runs all the container/helper demos added for C++ framework parity.
pub fn containers_test() {
    println!("Starting containers/helpers parity test...");
    println!("-----------------------------------------------");

    test_sync_vector();
    test_sync_priority_queue();
    test_ring_buffer_and_sync_ring_buffer();
    test_time_list_and_sync_time_list();
    test_async_observer_queue_overflow();
    test_worker_delegate_async();

    println!("Containers/helpers parity test completed.");
    println!("-----------------------------------------------");
}
