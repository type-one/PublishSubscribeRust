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

use crate::tools::sync_object::SyncObject;
use crate::tools::sync_queue::SyncQueue;
use crate::tools::task_function::TaskFunction;

/// Struct representing a worker task.
pub struct WorkerTask<ContextType: Send + Sync + 'static> {
    task_name: String,
    context: std::sync::Arc<ContextType>,
    work_sync_object: std::sync::Arc<std::sync::Mutex<SyncObject>>,
    work_queue: std::sync::Arc<SyncQueue<std::sync::Arc<TaskFunction<ContextType>>>>,
    stop_task: std::sync::Arc<std::sync::atomic::AtomicBool>,
    task_handle: Option<std::thread::JoinHandle<()>>,
}

/// Implementation of the WorkerTask methods.
impl<ContextType: Send + Sync + 'static> WorkerTask<ContextType> {
    /// Creates a new WorkerTask.
    pub fn new(task_name: String, context: std::sync::Arc<ContextType>) -> Self {
        WorkerTask {
            task_name,
            context: context.clone(),
            work_sync_object: std::sync::Arc::new(std::sync::Mutex::new(SyncObject::new(false))),
            work_queue: std::sync::Arc::new(SyncQueue::new()),
            stop_task: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            task_handle: None,
        }
    }

    /// Starts the worker task.
    pub fn start(&mut self) {
        let task_name = self.task_name.clone();
        let work_sync_object = self.work_sync_object.clone();
        let work_queue = self.work_queue.clone();
        let stop_task = self.stop_task.clone();
        let context = self.context.clone();

        self.task_handle = Some(
            std::thread::Builder::new()
                .name(task_name.clone())
                .spawn(move || {
                    Self::run_loop(work_sync_object, work_queue, stop_task, context, task_name);
                })
                .expect("Failed to spawn worker task"),
        );
    }

    /// Delegates a task function to the worker task.
    pub fn delegate(&mut self, task_function: std::sync::Arc<TaskFunction<ContextType>>) {
        self.work_queue.enqueue(task_function);
        // lock the SyncObject before signaling
        self.work_sync_object.lock().unwrap().signal();
    }

    // The main loop of the worker task.
    fn run_loop(
        work_sync_object: std::sync::Arc<std::sync::Mutex<SyncObject>>,
        work_queue: std::sync::Arc<SyncQueue<std::sync::Arc<TaskFunction<ContextType>>>>,
        stop_task: std::sync::Arc<std::sync::atomic::AtomicBool>,
        context: std::sync::Arc<ContextType>,
        task_name: String,
    ) {
        while !stop_task.load(std::sync::atomic::Ordering::Acquire) {
            // Wait for work
            {
                //println!("Worker task '{}' waiting for signal...", task_name);
                //work_sync_object.lock().unwrap().wait_for_signal(); //
                work_sync_object
                    .lock()
                    .unwrap()
                    .wait_for_signal_timeout(1000);
            }
            // Process all tasks in the queue
            //println!("Worker task '{}' woke up to process tasks.", task_name);
            while let Some(task_function) = work_queue.dequeue()
                && !stop_task.load(std::sync::atomic::Ordering::Acquire)
            {
                (task_function)(context.clone(), &task_name);
            }
        } // run loop
    }
}

/// Implementation of the Drop trait for WorkerTask.
impl<ContextType: Send + Sync + 'static> Drop for WorkerTask<ContextType> {
    fn drop(&mut self) {
        // Stop the worker task
        //println!("Stopping worker task: {}", self.task_name);
        self.stop_task
            .store(true, std::sync::atomic::Ordering::Release);

        //println!("Signaling worker task to wake up: {}", self.task_name);
        // Wake up the worker task
        self.work_sync_object.lock().unwrap().signal_all();

        //self.delegate(std::sync::Arc::new(|_, _| {
        // no-op task to ensure the worker wakes up
        //println!("No-op task executed to wake up the worker.");
        //}));

        std::thread::yield_now();

        // wait for the worker task to finish
        //println!("Waiting for worker task to finish: {}", self.task_name);
        if let Some(handle) = self.task_handle.take() {
            handle.join().unwrap();
        }
        //println!("Worker task stopped: {}", self.task_name);
    }
}
