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

use std::sync::{Condvar, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Synchronization object using standard Rust constructs.
///
/// This file contains the definition of the sync_object class, which provides
/// a synchronization mechanism using standard Rust constructs such as mutexes and
/// condition variables.
pub struct SyncObject {
    signaled: Mutex<bool>,
    condvar: Condvar,
    stop: AtomicBool,
}

/// Implementation of the SyncObject methods.
impl SyncObject {
    /// Creates a new SyncObject with the specified initial state.
    pub fn new(initial_state: bool) -> Self {
        SyncObject {
            signaled: Mutex::new(initial_state),
            condvar: Condvar::new(),
            stop: AtomicBool::new(false),
        }
    }

    /// Waits for a signal to be received.
    pub fn wait_for_signal(&mut self) {
        let mut signaled_guard = self.signaled.lock().unwrap();

        while !*signaled_guard {
            signaled_guard = self.condvar.wait(signaled_guard).unwrap();
        }

        *signaled_guard = self.stop.load(Ordering::Acquire);
    }

    /// Waits for a signal to be received with a timeout.
    pub fn wait_for_signal_timeout(&mut self, timeout_ms: u64) {
        let mut signaled_guard = self.signaled.lock().unwrap();

        while !*signaled_guard {
            let (new_signaled_guard, timeout_status) = self
                .condvar
                .wait_timeout(signaled_guard, Duration::from_millis(timeout_ms))
                .unwrap();

            signaled_guard = new_signaled_guard; // move

            if timeout_status.timed_out() {
                break;
            }
        }

        *signaled_guard = self.stop.load(Ordering::Acquire);
    }

    /// Sends a signal to wake up one of the waiting threads.
    pub fn signal(&mut self) {
        {
            let mut signaled_guard = self.signaled.lock().unwrap();
            *signaled_guard = true;
        }
        self.condvar.notify_one();
    }

    /// Sends a signal to wake up all waiting threads.
    pub fn signal_all(&mut self) {
        {
            let mut signaled_guard = self.signaled.lock().unwrap();
            *signaled_guard = true;
        }
        self.condvar.notify_all();
    }
}

/// Implementation of the Drop trait for SyncObject.
impl Drop for SyncObject {
    /// Cleans up the SyncObject by signaling all waiting threads to stop.
    fn drop(&mut self) {
        {
            let mut signaled_guard = self.signaled.lock().unwrap();
            *signaled_guard = true;
            self.stop.store(true, Ordering::Release);
        }
        self.condvar.notify_all();
    }
}

/// Implementation of the Default trait for SyncObject.
impl Default for SyncObject {
    /// Creates a default SyncObject with an initial state of false.
    fn default() -> Self {
        Self::new(false)
    }
}
