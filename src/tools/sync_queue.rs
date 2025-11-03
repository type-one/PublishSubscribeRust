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

/// Thread-safe queue implementation using standard Rust constructs.
pub struct SyncQueue<T> {
    queue: std::sync::Mutex<std::collections::VecDeque<T>>,
}

/// Implementation of the SyncQueue methods.
impl<T> SyncQueue<T> {
    /// Creates a new SyncQueue.
    pub fn new() -> Self {
        SyncQueue {
            queue: std::sync::Mutex::new(std::collections::VecDeque::new()),
        }
    }

    /// Adds an item to the back of the queue.
    pub fn enqueue(&self, item: T) {
        let mut guard = self.queue.lock().unwrap();
        guard.push_back(item);
    }

    /// Removes and returns an item from the front of the queue.
    pub fn dequeue(&self) -> Option<T> {
        let mut guard = self.queue.lock().unwrap();
        guard.pop_front()
    }

    /// Checks if the queue is empty.
    pub fn is_empty(&self) -> bool {
        let guard = self.queue.lock().unwrap();
        guard.is_empty()
    }

    /// Returns the size of the queue.
    pub fn size(&self) -> usize {
        let guard = self.queue.lock().unwrap();
        guard.len()
    }

    /// Clears all items from the queue.
    pub fn clear(&self) {
        let mut guard = self.queue.lock().unwrap();
        guard.clear();
    }

    /// Returns a reference to the front item of the queue.
    pub fn front(&self) -> Option<T>
    where
        T: Clone,
    {
        let guard = self.queue.lock().unwrap();
        guard.front().cloned()
    }

    /// Returns a reference to the back item of the queue.
    pub fn back(&self) -> Option<T>
    where
        T: Clone,
    {
        let guard = self.queue.lock().unwrap();
        guard.back().cloned()
    }
}

/// Default implementation for SyncQueue.
impl<T> Default for SyncQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
