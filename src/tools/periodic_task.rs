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
use std::time::{Duration, Instant};

use crate::tools::task_function::TaskFunction;
use crate::tools::task_trait::TaskTrait;
/// Struct representing a periodic task.
pub struct PeriodicTask<ContextType> {
    task_name: String,
    task_function: Arc<TaskFunction<ContextType>>,
    period_ms: u64,
    context: Arc<ContextType>,
    stop_task: Arc<AtomicBool>,
    started: Arc<AtomicBool>,
    task_handle: Option<std::thread::JoinHandle<()>>,
}

/// Implementation of the PeriodicTask methods.
impl<ContextType: Send + Sync + 'static> PeriodicTask<ContextType> {
    /// Creates a new PeriodicTask.
    pub fn new(
        context: Arc<ContextType>,
        task_name: String,
        task_function: Arc<TaskFunction<ContextType>>,
        period_ms: u64,
    ) -> Self {
        PeriodicTask {
            task_name,
            task_function,
            period_ms,
            context: context.clone(),
            stop_task: Arc::new(AtomicBool::new(false)),
            started: Arc::new(AtomicBool::new(false)),
            task_handle: None,
        }
    }

    // The main loop of the periodic task.
    fn run_loop(
        task_function: Arc<TaskFunction<ContextType>>,
        task_name: String,
        period_ms: u64,
        context: Arc<ContextType>,
        stop_task: Arc<AtomicBool>,
        started: Arc<AtomicBool>,
    ) {
        let start_time = Instant::now();
        let mut deadline = start_time + Duration::from_millis(period_ms);

        // mark the task as started
        started.store(true, Ordering::Release);

        // periodic task loop
        while !stop_task.load(Ordering::Acquire) {
            let mut current_time = Instant::now();

            // active wait until the deadline is reached
            while !stop_task.load(Ordering::Acquire) && current_time < deadline {
                current_time = Instant::now();
                std::hint::spin_loop();
            }

            // exit if stop requested
            if stop_task.load(Ordering::Acquire) {
                break;
            }

            // execute given periodic function
            (task_function)(context.clone(), &task_name);

            // compute next deadline
            current_time = Instant::now();
            deadline += Duration::from_millis(period_ms);

            // wait period
            if deadline > current_time {
                let remaining_time = (deadline - current_time).as_micros();
                // wait 90% of the remaining time to avoid busy waiting
                // sleep until we are close to the deadline
                std::thread::sleep(Duration::from_micros((remaining_time * 90 / 100) as u64));
            } // end if wait period needed
        } // periodic task loop
    }
}

/// Implementation of the Drop trait for PeriodicTask.
impl<ContextType> Drop for PeriodicTask<ContextType> {
    fn drop(&mut self) {
        // signal the task to stop
        self.stop_task.store(true, Ordering::Release);

        // wait for the task to finish
        if let Some(handle) = self.task_handle.take() {
            handle.join().unwrap();
        }
    }
}

/// Implementation of the TaskTrait for PeriodicTask.
impl<ContextType: Send + Sync + 'static> TaskTrait<ContextType> for PeriodicTask<ContextType> {
    /// Starts the periodic task.
    fn start(&mut self) {
        let task_function = self.task_function.clone();
        let context = self.context.clone();
        let task_name = self.task_name.clone();
        let period_ms = self.period_ms;
        let stop_task = self.stop_task.clone();
        let started = self.started.clone();

        self.task_handle = Some(
            std::thread::Builder::new()
                .name(task_name.clone())
                .spawn(move || {
                    Self::run_loop(
                        task_function,
                        task_name,
                        period_ms,
                        context,
                        stop_task,
                        started,
                    );
                })
                .expect("Failed to spawn periodic task"),
        );
    }

    /// Checks if the periodic task has been started.
    /// Returns true if started, false otherwise.
    fn is_started(&self) -> bool {
        self.started.load(Ordering::Acquire)
    }
}
