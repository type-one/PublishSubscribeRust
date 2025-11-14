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
use crate::tools::task_function::TaskFunction;
use crate::tools::task_trait::TaskTrait;
use crate::tools::worker_trait::WorkerTrait;

// ContextType must be Send + Sync + 'static to be safely shared across threads.
// It means that ContextType can be transferred across thread boundaries (Send),
// can be referenced from multiple threads simultaneously (Sync), and does not
// contain any non-static references ('static - static lifetime - valid for the
// entire duration of the program).

/// Struct representing a worker task.
pub struct WorkerTask<ContextType: Send + Sync + 'static> {
    task_name: String,
    context: Arc<ContextType>,
    work_sender: Option<Sender<bool>>,
    work_queue: Arc<SyncQueue<Arc<TaskFunction<ContextType>>>>,
    task_handle: Option<std::thread::JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
}

/// Implementation of the WorkerTask methods.
impl<ContextType: Send + Sync + 'static> WorkerTask<ContextType> {
    /// Creates a new WorkerTask.
    pub fn new(context: Arc<ContextType>, task_name: String) -> Self {
        WorkerTask {
            task_name,
            context: context.clone(),
            work_sender: None, // dummy initialization
            work_queue: Arc::new(SyncQueue::new()),
            task_handle: None,
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    // The main loop of the worker task.
    fn run_loop(
        receiver: Receiver<bool>,
        work_queue: Arc<SyncQueue<Arc<TaskFunction<ContextType>>>>,
        stop_signal: Arc<AtomicBool>,
        context: Arc<ContextType>,
        task_name: String,
    ) {
        // Wait for work
        loop {
            println!("Worker task '{}' waiting for work...", task_name);
            // Wait for a signal to do work (or stop)
            // Channel closed, treat as stop signal
            //let received_message = receiver.recv().unwrap_or(false);

            if stop_signal.load(Ordering::Acquire) {
                break; // Exit the loop if a stop signal is received or channel is closed
            }

            std::thread::sleep(std::time::Duration::from_millis(1000)); // simulate waiting for work

            if work_queue.is_empty() {
                continue; // No work to do, continue waiting
            }

            println!("Worker task '{}' received work signal.", task_name);

            // Process all tasks in the queue
            while let Some(task_function) = work_queue.dequeue() {
                (task_function)(context.clone(), &task_name);
            }
        } // run loop
    }
}

/// Implementation of the Drop trait for WorkerTask.
impl<ContextType: Send + Sync + 'static> Drop for WorkerTask<ContextType> {
    fn drop(&mut self) {
        // Stop the worker task
        if let Some(sender) = &self.work_sender {
            self.stop_signal.store(true, Ordering::Release);
            sender.send(false).unwrap_or_default(); // send stop signal to worker task and ignore errors

            // wait for the worker task to finish
            if let Some(handle) = self.task_handle.take() {
                handle.join().unwrap();
            }
        }
    }
}

/// Implementation of the TaskTrait for WorkerTask.
impl<ContextType: Send + Sync + 'static> TaskTrait<ContextType> for WorkerTask<ContextType> {
    /// Starts the worker task.
    fn start(&mut self) {
        let task_name = self.task_name.clone();
        let work_queue = self.work_queue.clone();
        let context = self.context.clone();
        let stop_signal = self.stop_signal.clone();

        // https://kundan926.medium.com/exploring-the-basics-of-rusts-thread-concept-d8922d12e2f0

        let (sender, receiver) = std::sync::mpsc::channel();
        self.work_sender = Some(sender);

        self.task_handle = Some(
            std::thread::Builder::new()
                .name(task_name.clone())
                .spawn(move || {
                    Self::run_loop(receiver, work_queue, stop_signal, context, task_name);
                })
                .expect("Failed to spawn worker task"),
        );
    }
}

/// Implementation of the WorkerTrait for WorkerTask.
impl<ContextType: Send + Sync + 'static> WorkerTrait<ContextType> for WorkerTask<ContextType> {
    /// Delegates a task function to the worker task.
    fn delegate(&mut self, task_function: Arc<TaskFunction<ContextType>>) {
        self.work_queue.enqueue(task_function);

        if self.work_sender.is_some() {
            // Signal the worker task that there is work to do
            self.work_sender
                .as_ref()
                .unwrap()
                .send(true)
                .unwrap_or_else(|_| eprintln!("Failed to send work signal to worker task"));
        }
    }
}
