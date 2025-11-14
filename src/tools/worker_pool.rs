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

use crate::tools::task_function::TaskFunction;
use crate::tools::task_trait::TaskTrait;
use crate::tools::worker_trait::WorkerTrait;

// ContextType must be Send + Sync + 'static to be safely shared across threads.
// It means that ContextType can be transferred across thread boundaries (Send),
// can be referenced from multiple threads simultaneously (Sync), and does not
// contain any non-static references ('static - static lifetime - valid for the
// entire duration of the program).

struct Job<ContextType: Send + Sync + 'static> {
    context: Arc<ContextType>,
    task_function: Arc<TaskFunction<ContextType>>,
}

impl<ContextType: Send + Sync + 'static> Job<ContextType> {
    fn process(self, task_name: &String) {
        (self.task_function)(self.context.clone(), task_name);
    }
}
/// Struct representing a worker pool.
pub struct WorkerPool<ContextType: Send + Sync + 'static> {
    context: Arc<ContextType>,
}

/// Implementation of the WorkerPool methods.
impl<ContextType: Send + Sync + 'static> WorkerPool<ContextType> {
    /// Creates a new WorkerPool.
    pub fn new(context: Arc<ContextType>) -> Self {
        WorkerPool { context }
    }

    /// Spawns a new worker (using tokio's own reactor pool).
    #[tokio::main]
    async fn spawn_worker(&self, task_function: Arc<TaskFunction<ContextType>>) {
        let job = Job {
            context: self.context.clone(),
            task_function: task_function.clone(),
        };

        // https://tokio.rs/tokio/tutorial/spawning

        // one-way function, no need to await
        tokio::spawn(async move {
            let task_name = std::thread::current()
                .name()
                .unwrap_or("Worker")
                .to_string();
            job.process(&task_name);
        });
    }
}

/// Implementation of the Drop trait for WorkerPool.
impl<ContextType: Send + Sync + 'static> Drop for WorkerPool<ContextType> {
    fn drop(&mut self) {
        // Clean up resources when the WorkerPool is dropped
    }
}

/// Implementation of the TaskTrait for WorkerPool.
impl<ContextType: Send + Sync + 'static> TaskTrait<ContextType> for WorkerPool<ContextType> {
    /// Starts the worker pool.
    fn start(&mut self) {
        // Start the worker pool (e.g., by spawning a certain number of workers)
        // default implementation does nothing as tokio spawns workers on demand
    }

    /// Checks if the worker pool has been started.
    fn is_started(&self) -> bool {
        // For simplicity, we can assume the worker pool is always started
        true
    }
}

/// Implementation of the WorkerTrait for WorkerPool.
impl<ContextType: Send + Sync + 'static> WorkerTrait<ContextType> for WorkerPool<ContextType> {
    /// Delegates a task function to the worker pool.
    fn delegate(&mut self, task_function: Arc<TaskFunction<ContextType>>) {
        // Enqueue the task function for processing by the worker pool
        self.spawn_worker(task_function);
    }
}
