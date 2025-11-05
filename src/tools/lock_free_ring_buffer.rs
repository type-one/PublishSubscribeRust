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

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Thread-safe and lock-free ring buffer implementation using standard Rust constructs.
#[derive(Debug)]
pub struct LockFreeRingBuffer<T, const POW2N: usize> {
    ring_buffer: Vec<T>,
    push_index: AtomicUsize,
    pop_index: AtomicUsize,
    reading: AtomicBool,
    writing: AtomicBool,
}

/// Implementation of the LockFreeRingBuffer methods.
impl<T, const POW2N: usize> LockFreeRingBuffer<T, POW2N> {
    const RING_BUFFER_SIZE: usize = 1 << POW2N;
    const RING_BUFFER_MASK: usize = Self::RING_BUFFER_SIZE - 1;

    /// Creates a new LockFreeRingBuffer with a power of 2 capacity.
    pub fn new() -> Self {
        let mut tmp_buffer = Vec::with_capacity(Self::RING_BUFFER_SIZE);

        // Initialize the buffer with default values
        for _ in 0..Self::RING_BUFFER_SIZE {
            tmp_buffer.push(unsafe { std::mem::MaybeUninit::uninit().assume_init() });
        }

        LockFreeRingBuffer {
            ring_buffer: tmp_buffer,
            push_index: AtomicUsize::new(0),
            pop_index: AtomicUsize::new(0),
            reading: AtomicBool::new(false),
            writing: AtomicBool::new(false),
        }
    }

    /// Adds an item to the back of the ring buffer.
    pub fn enqueue(&mut self, item: T) -> Result<(), &'static str> {
        let snap_write_index = self.push_index.load(Ordering::Acquire);
        let snap_read_index = self.pop_index.load(Ordering::Acquire);

        // Check if the buffer is full
        if (snap_read_index & Self::RING_BUFFER_MASK)
            == (snap_write_index + 1) & Self::RING_BUFFER_MASK
        {
            return Err("Buffer is full");
        }

        // Getting close or wrap around, risk of race condition
        if ((snap_write_index - snap_read_index) <= 2) || (snap_write_index < snap_read_index) {
            // Spin until we can write (no pending reads)
            while self.reading.load(Ordering::Acquire) {
                std::hint::spin_loop();
            }
        }

        // Add the item to the buffer
        self.writing.store(true, Ordering::Release);
        let write_index = self.push_index.fetch_add(1, Ordering::AcqRel);
        self.ring_buffer[write_index & Self::RING_BUFFER_MASK] = item;
        self.writing.store(false, Ordering::Release);

        Ok(())
    }

    /// Removes and returns an item from the front of the ring buffer.
    pub fn dequeue(&mut self) -> Option<T> {
        let snap_write_index = self.push_index.load(Ordering::Acquire);
        let snap_read_index = self.pop_index.load(Ordering::Acquire);

        // Check if the buffer is empty
        if (snap_read_index & Self::RING_BUFFER_MASK) == (snap_write_index & Self::RING_BUFFER_MASK)
        {
            return None;
        }

        // Getting close or wrap around, risk of race condition
        if ((snap_write_index - snap_read_index) <= 2) || (snap_write_index < snap_read_index) {
            // Spin until we can read (no pending writes)
            while self.writing.load(Ordering::Acquire) {
                std::hint::spin_loop();
            }
        }

        self.reading.store(true, Ordering::Release);
        let read_index = self.pop_index.fetch_add(1, Ordering::AcqRel);
        let item = std::mem::replace(
            &mut self.ring_buffer[read_index & Self::RING_BUFFER_MASK],
            unsafe { std::mem::MaybeUninit::uninit().assume_init() },
        );
        self.reading.store(false, Ordering::Release);

        Some(item)
    }

    pub fn capacity(&self) -> usize {
        Self::RING_BUFFER_SIZE
    }
}

/// Default implementation for LockFreeRingBuffer.
impl<T, const POW2N: usize> Default for LockFreeRingBuffer<T, POW2N> {
    fn default() -> Self {
        Self::new()
    }
}
