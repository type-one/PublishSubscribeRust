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

/// Synchronization object using standard Rust constructs.
///
/// This file contains the definition of the sync_object class, which provides
/// a synchronization mechanism using standard Rust constructs such as mutexes and
/// condition variables.
pub struct SyncObject {
    mutex: std::sync::Mutex<bool>,
    condvar: std::sync::Condvar,
    stop: bool,
}

/// Implementation of the SyncObject methods.
impl SyncObject {
    /// Creates a new SyncObject with the specified initial state.
    pub fn new(initial_state: bool) -> Self {
        SyncObject {
            mutex: std::sync::Mutex::new(initial_state),
            condvar: std::sync::Condvar::new(),
            stop: false,
        }
    }

    /// Waits for a signal to be received.
    pub fn wait_for_signal(&mut self) {
        let mut guard = self.mutex.lock().unwrap();
        while !*guard {
            guard = self.condvar.wait(guard).unwrap();
        }
        *guard = self.stop;
    }

    /// Waits for a signal to be received with a timeout.
    pub fn wait_for_signal_timeout(&mut self, timeout_ms: u64) {
        let mut guard = self.mutex.lock().unwrap();
        (guard, _) = self
            .condvar
            .wait_timeout(guard, std::time::Duration::from_millis(timeout_ms))
            .unwrap();
        *guard = self.stop;
    }

    /// Sends a signal to wake up waiting threads.
    pub fn signal(&mut self) {
        {
            let mut guard = self.mutex.lock().unwrap();
            *guard = true;
        }
        self.condvar.notify_one();
    }
}

/// Implementation of the Drop trait for SyncObject.
impl Drop for SyncObject {
    /// Cleans up the SyncObject by signaling all waiting threads to stop.
    fn drop(&mut self) {
        {
            let mut guard = self.mutex.lock().unwrap();
            *guard = true;
            self.stop = true;
        }
        self.condvar.notify_all();
    }
}
