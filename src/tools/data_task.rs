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
    data_sender: Sender<bool>,
    data_queue: Arc<SyncQueue<DataType>>,
    data_processing_function: Arc<DataTaskFunction<ContextType, DataType>>,
    task_handle: Option<std::thread::JoinHandle<()>>,
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
            data_sender: std::sync::mpsc::channel().0, // dummy initialization
            data_queue: Arc::new(SyncQueue::new()),
            data_processing_function,
            task_handle: None,
        }
    }

    // The main loop of the data task.
    fn run_loop(
        receiver: Receiver<bool>,
        data_queue: Arc<SyncQueue<DataType>>,
        context: Arc<ContextType>,
        task_name: String,
        data_processing_function: Arc<DataTaskFunction<ContextType, DataType>>,
    ) {
        // Wait for work
        loop {
            // Wait for a signal to do work (or stop)
            let received_message = receiver.recv().unwrap_or(false);

            if !received_message {
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
        self.data_queue.enqueue(data);
        // Notify the data task that new data is available
        self.data_sender.send(true).unwrap();
    }
}

/// Implementation of the Drop trait for DataTask.
impl<ContextType: Send + Sync + 'static, DataType: Send + Sync + 'static> Drop
    for DataTask<ContextType, DataType>
{
    fn drop(&mut self) {
        // Stop the data task
        self.data_sender.send(false).unwrap_or_default(); // send stop signal to data task and ignore errors

        // wait for the data task to finish
        if let Some(handle) = self.task_handle.take() {
            handle.join().unwrap();
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

        // https://kundan926.medium.com/exploring-the-basics-of-rusts-thread-concept-d8922d12e2f0

        let (sender, receiver) = std::sync::mpsc::channel();
        self.data_sender = sender;

        self.task_handle = Some(
            std::thread::Builder::new()
                .name(task_name.clone())
                .spawn(move || {
                    Self::run_loop(
                        receiver,
                        data_queue,
                        context,
                        task_name,
                        data_processing_function,
                    );
                })
                .expect("Failed to spawn data task"),
        );
    }
}
