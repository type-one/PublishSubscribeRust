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
use std::time::{Duration, Instant};

/// Synchronization object using standard Rust constructs.
///
/// This file contains the definition of the sync_object class, which provides
/// a synchronization mechanism using standard Rust constructs such as mutexes and
/// condition variables.
pub struct SyncObject {
    state: Mutex<SyncState>,
    condvar: Condvar,
}

#[derive(Debug, Default)]
struct SyncState {
    tokens: usize,
    waiters: usize,
}

/// Implementation of the SyncObject methods.
impl SyncObject {
    /// Creates a new SyncObject.
    pub fn new() -> Self {
        SyncObject {
            state: Mutex::new(SyncState::default()),
            condvar: Condvar::new(),
        }
    }

    /// Waits for a signal to be received.
    pub fn wait_for_signal(&self) {
        let mut state_guard = self.state.lock().unwrap();

        if state_guard.tokens > 0 {
            state_guard.tokens -= 1;
            return;
        }

        state_guard.waiters += 1;

        while state_guard.tokens == 0 {
            state_guard = self.condvar.wait(state_guard).unwrap();
        }

        state_guard.tokens -= 1;
        state_guard.waiters -= 1;
    }

    /// Waits for a signal to be received with a timeout.
    pub fn wait_for_signal_timeout(&self, timeout_ms: u64) {
        let mut state_guard = self.state.lock().unwrap();
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);

        if state_guard.tokens > 0 {
            state_guard.tokens -= 1;
            return;
        }

        state_guard.waiters += 1;

        while state_guard.tokens == 0 {
            let now = Instant::now();
            if now >= deadline {
                state_guard.waiters -= 1;
                return;
            }

            let (new_state_guard, timeout_status) = self
                .condvar
                .wait_timeout(state_guard, deadline.saturating_duration_since(now))
                .unwrap();

            state_guard = new_state_guard;

            if timeout_status.timed_out() && state_guard.tokens == 0 {
                state_guard.waiters -= 1;
                return;
            }
        }

        state_guard.tokens -= 1;
        state_guard.waiters -= 1;
    }

    /// Sends a signal to wake up one of the waiting threads.
    pub fn signal(&self) {
        {
            let mut state_guard = self.state.lock().unwrap();
            state_guard.tokens = state_guard.tokens.saturating_add(1);
        }
        self.condvar.notify_one();
    }

    /// Sends a signal to wake up all waiting threads.
    pub fn signal_all(&self) {
        {
            let mut state_guard = self.state.lock().unwrap();
            let wake_count = state_guard.waiters.max(1);
            state_guard.tokens = state_guard.tokens.saturating_add(wake_count);
        }
        self.condvar.notify_all();
    }
}

/// Implementation of the Default trait for SyncObject.
impl Default for SyncObject {
    /// Creates a default SyncObject with an initial state of false.
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for SyncObject.
#[cfg(test)]
mod tests {
    use super::SyncObject;
    use std::sync::Arc;
    use std::thread;
    use std::time::Instant;

    // basic test for signal and wait
    #[test]
    fn test_signal() {
        let sync_object = Arc::new(SyncObject::new());
        let child_sync_object = sync_object.clone();

        let handle = thread::spawn(move || {
            child_sync_object.wait_for_signal();
        });

        thread::sleep(std::time::Duration::from_millis(100));
        sync_object.signal();
        handle.join().unwrap();
    }

    // basic test for signal with timeout
    #[test]
    fn test_signal_timeout() {
        let sync_object = Arc::new(SyncObject::new());
        let child_sync_object = sync_object.clone();

        let handle = thread::spawn(move || {
            let start = Instant::now();
            child_sync_object.wait_for_signal_timeout(200);
            start.elapsed()
        });

        thread::sleep(std::time::Duration::from_millis(100));
        sync_object.signal();
        let elapsed = handle.join().unwrap();
        assert!(elapsed < std::time::Duration::from_millis(300));
    }

    // test for signal timeout expire
    #[test]
    fn test_signal_timeout_expire() {
        let sync_object = Arc::new(SyncObject::new());
        let child_sync_object = sync_object.clone();

        let handle = thread::spawn(move || {
            let start = Instant::now();
            child_sync_object.wait_for_signal_timeout(200);
            start.elapsed()
        });

        let elapsed = handle.join().unwrap();
        assert!(elapsed >= std::time::Duration::from_millis(200));
    }

    // test for Default trait
    #[test]
    fn test_default_trait() {
        let sync_object = Arc::new(SyncObject::default());
        let child_sync_object = sync_object.clone();

        let handle = thread::spawn(move || {
            child_sync_object.wait_for_signal();
        });

        thread::sleep(std::time::Duration::from_millis(100));
        sync_object.signal();
        handle.join().unwrap();
    }

    // Additional test with two threads
    #[test]
    fn test_concurrent_signal_wait() {
        let sync_object = Arc::new(SyncObject::new());
        let sync_object_for_waiter = sync_object.clone();
        let sync_object_for_signaler = sync_object.clone();

        let waiter = thread::spawn(move || {
            sync_object_for_waiter.wait_for_signal();
        });

        let signaler = thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(100));
            sync_object_for_signaler.signal();
        });

        waiter.join().unwrap();
        signaler.join().unwrap();
    }

    // test for Default trait with timeout
    #[test]
    fn test_default_trait_with_timeout() {
        let sync_object = Arc::new(SyncObject::default());
        let child_sync_object = sync_object.clone();

        let handle = thread::spawn(move || {
            child_sync_object.wait_for_signal_timeout(500);
        });

        thread::sleep(std::time::Duration::from_millis(100));
        sync_object.signal();
        handle.join().unwrap();
    }

    // test for Default trait with timeout expire
    #[test]
    fn test_default_trait_with_timeout_expire() {
        let sync_object = Arc::new(SyncObject::default());
        let child_sync_object = sync_object.clone();

        let handle = thread::spawn(move || {
            let start = Instant::now();
            child_sync_object.wait_for_signal_timeout(200);
            start.elapsed()
        });

        let elapsed = handle.join().unwrap();
        assert!(elapsed >= std::time::Duration::from_millis(200));
    }

    // signal before waiting should be observed immediately
    #[test]
    fn test_signal_before_wait_is_not_lost() {
        let sync_object = SyncObject::new();

        sync_object.signal();

        let start = Instant::now();
        sync_object.wait_for_signal_timeout(500);
        let elapsed = start.elapsed();

        assert!(elapsed < std::time::Duration::from_millis(50));
    }

    // test for signal_all
    #[test]
    fn test_signal_all() {
        let sync_object = Arc::new(SyncObject::new());
        let mut handles = vec![];

        for _ in 0..5 {
            let child_sync_object = Arc::clone(&sync_object);
            let handle = thread::spawn(move || {
                child_sync_object.wait_for_signal();
            });
            handles.push(handle);
        }

        thread::sleep(std::time::Duration::from_millis(100));
        sync_object.signal_all();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
