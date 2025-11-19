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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};

use crate::tools::sync_queue::SyncQueue;
use crate::tools::task_function::DataTaskFunction;
use crate::tools::task_trait::TaskTrait;

// ContextType must be Send + Sync + 'static to be safely shared across threads.
// It means that ContextType can be transferred across thread boundaries (Send),
// can be referenced from multiple threads simultaneously (Sync), and does not
// contain any non-static references ('static - static lifetime - valid for the
// entire duration of the program).

/// Struct representing a data task.
pub struct DataTask<ContextType: Send + Sync + 'static, DataType: Send + Sync + 'static> {
    task_name: String,
    context: Arc<ContextType>,
    data_sender: Option<Sender<bool>>,
    data_queue: Arc<SyncQueue<DataType>>,
    data_processing_function: Arc<DataTaskFunction<ContextType, DataType>>,
    task_handle: Option<std::thread::JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
    started: Arc<AtomicBool>,
}

/// Implementation of the DataTask methods.
impl<ContextType: Send + Sync + 'static, DataType: Send + Sync + 'static>
    DataTask<ContextType, DataType>
{
    /// Creates a new DataTask.
    pub fn new(
        context: Arc<ContextType>,
        task_name: String,
        data_processing_function: Arc<DataTaskFunction<ContextType, DataType>>,
    ) -> Self {
        DataTask {
            task_name,
            context: context.clone(),
            data_sender: None, // dummy initialization
            data_queue: Arc::new(SyncQueue::new()),
            data_processing_function,
            task_handle: None,
            stop_signal: Arc::new(AtomicBool::new(false)),
            started: Arc::new(AtomicBool::new(false)),
        }
    }

    // The main loop of the data task.
    fn run_loop(
        receiver: Receiver<bool>,
        data_queue: Arc<SyncQueue<DataType>>,
        stop_signal: Arc<AtomicBool>,
        started: Arc<AtomicBool>,
        context: Arc<ContextType>,
        task_name: String,
        data_processing_function: Arc<DataTaskFunction<ContextType, DataType>>,
    ) {
        // mark the task as started
        started.store(true, Ordering::Release);

        // Wait for work
        loop {
            // Wait for a signal to do work (or stop)
            let received_message = receiver.recv().unwrap_or(false);

            if !received_message || stop_signal.load(Ordering::Acquire) {
                break; // Exit the loop if a stop signal is received or channel is closed
            }

            // Process all data in the queue
            while let Some(data) = data_queue.dequeue() {
                (data_processing_function)(context.clone(), &task_name, data);
            }
        } // run loop
    }

    /// Submits data to the data task.
    pub fn submit(&self, data: DataType) {
        if !self.is_started() {
            return; // ignore submissions if the task is not started
        }

        self.data_queue.enqueue(data);
        // Notify the data task that new data is available
        if let Some(sender) = &self.data_sender {
            sender.send(true).unwrap();
        }
    }
}

/// Implementation of the Drop trait for DataTask.
impl<ContextType: Send + Sync + 'static, DataType: Send + Sync + 'static> Drop
    for DataTask<ContextType, DataType>
{
    fn drop(&mut self) {
        // Stop the data task
        self.stop_signal.store(true, Ordering::Release);

        if let Some(sender) = &self.data_sender {
            sender.send(true).unwrap_or_default(); // send signal to unblock the receiver and ignore errors

            // wait for the data task to finish
            if let Some(handle) = self.task_handle.take() {
                handle.join().unwrap();
            }
        }
    }
}

/// Implementation of the TaskTrait for DataTask.
impl<ContextType: Send + Sync + 'static, DataType: Send + Sync + 'static> TaskTrait<ContextType>
    for DataTask<ContextType, DataType>
{
    /// Starts the data task.
    fn start(&mut self) {
        let task_name = self.task_name.clone();
        let data_queue = self.data_queue.clone();
        let context = self.context.clone();
        let data_processing_function = self.data_processing_function.clone();
        let stop_signal = self.stop_signal.clone();
        let started = self.started.clone();

        stop_signal.store(false, Ordering::Release);
        started.store(false, Ordering::Release);

        // https://kundan926.medium.com/exploring-the-basics-of-rusts-thread-concept-d8922d12e2f0

        let (sender, receiver) = std::sync::mpsc::channel();
        self.data_sender = Some(sender);

        self.task_handle = Some(
            std::thread::Builder::new()
                .name(task_name.clone())
                .spawn(move || {
                    Self::run_loop(
                        receiver,
                        data_queue,
                        stop_signal,
                        started,
                        context,
                        task_name,
                        data_processing_function,
                    );
                })
                .expect("Failed to spawn data task"),
        );
    }

    /// Checks if the data task has been started.
    /// Returns true if started, false otherwise.
    fn is_started(&self) -> bool {
        self.started.load(Ordering::Acquire)
    }

    /// Stops the data task.
    fn stop(&mut self) {
        // Stop the data task
        self.stop_signal.store(true, Ordering::Release);

        if let Some(sender) = &self.data_sender {
            sender.send(true).unwrap_or_default(); // send signal to unblock the receiver and ignore errors

            // wait for the data task to finish
            if let Some(handle) = self.task_handle.take() {
                handle.join().unwrap();
            }
        }

        self.started.store(false, Ordering::Release);
    }
}

// Unit tests for DataTask.
#[cfg(test)]
mod tests {
    use super::DataTask;
    use crate::tools::task_trait::TaskTrait;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    // basic test for data task submit and processing
    #[test]
    fn test_data_task_submit() {
        struct TestContext {
            counter: AtomicUsize,
        }

        let context = Arc::new(TestContext {
            counter: AtomicUsize::new(0),
        });

        let data_processing_function: Arc<
            dyn Fn(Arc<TestContext>, &String, usize) + Send + Sync + 'static,
        > = Arc::new(|ctx: Arc<TestContext>, _task_name: &String, data: usize| {
            ctx.counter.fetch_add(data, Ordering::AcqRel);
        });

        let mut data_task = DataTask::new(
            context.clone(),
            "TestDataTask".to_string(),
            data_processing_function,
        );

        data_task.start();

        // give some time to ensure the task has started
        thread::sleep(Duration::from_millis(100));

        for i in 1..=5 {
            data_task.submit(i);
        }

        // Give some time for all data to be processed
        thread::sleep(Duration::from_millis(500));

        assert_eq!(context.counter.load(Ordering::Acquire), 15); // 1+2+3+4+5 = 15

        data_task.stop();
    }

    // test start and stop
    #[test]
    fn test_data_task_start_stop() {
        struct TestContext {}
        let context = Arc::new(TestContext {});

        let data_processing_function: Arc<
            dyn Fn(Arc<TestContext>, &String, usize) + Send + Sync + 'static,
        > = Arc::new(
            |_ctx: Arc<TestContext>, _task_name: &String, _data: usize| {
                // Dummy processing
            },
        );

        let mut data_task = DataTask::new(
            context.clone(),
            "TestDataTask".to_string(),
            data_processing_function,
        );

        assert!(!data_task.is_started());
        data_task.start();

        // Give some time to ensure the task has started
        thread::sleep(Duration::from_millis(100));
        assert!(data_task.is_started());

        data_task.stop();
        assert!(!data_task.is_started());
    }

    // submit after stop should not process data
    #[test]
    fn test_data_task_submit_after_stop() {
        struct TestContext {
            counter: AtomicUsize,
        }
        let context = Arc::new(TestContext {
            counter: AtomicUsize::new(0),
        });

        let data_processing_function: Arc<
            dyn Fn(Arc<TestContext>, &String, usize) + Send + Sync + 'static,
        > = Arc::new(|ctx: Arc<TestContext>, _task_name: &String, data: usize| {
            ctx.counter.fetch_add(data, Ordering::AcqRel);
        });

        let mut data_task = DataTask::new(
            context.clone(),
            "TestDataTask".to_string(),
            data_processing_function,
        );
        data_task.start();
        data_task.stop();

        for i in 1..=5 {
            data_task.submit(i);
        }

        // Give some time to see if any data is processed
        thread::sleep(Duration::from_millis(500));

        assert_eq!(context.counter.load(Ordering::Acquire), 0); // No data should be processed
    }

    // test drop stops the task
    #[test]
    fn test_data_task_drop() {
        struct TestContext {
            counter: AtomicUsize,
        }
        let context = Arc::new(TestContext {
            counter: AtomicUsize::new(0),
        });

        let data_processing_function: Arc<
            dyn Fn(Arc<TestContext>, &String, usize) + Send + Sync + 'static,
        > = Arc::new(|ctx: Arc<TestContext>, _task_name: &String, data: usize| {
            ctx.counter.fetch_add(data, Ordering::AcqRel);
        });

        {
            let mut data_task = DataTask::new(
                context.clone(),
                "TestDataTask".to_string(),
                data_processing_function,
            );
            data_task.start();

            // give some time to ensure the task has started
            thread::sleep(Duration::from_millis(100));

            for i in 1..=5 {
                data_task.submit(i);
            }

            // Give some time for all data to be processed
            thread::sleep(Duration::from_millis(500));
        } // data_task goes out of scope and should be dropped here

        // Give some time to ensure the task has stopped
        thread::sleep(Duration::from_millis(100));

        assert_eq!(context.counter.load(Ordering::Acquire), 15); // 1+2+3+4+5 = 15
    }
}
