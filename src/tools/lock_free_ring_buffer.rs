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

// use num::traits::{Float, PrimInt};
// https://docs.rs/atomic/latest/atomic/
use atomic::Atomic;
use bytemuck::Pod;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct Slot<T: Default + Copy + Send + Sync + Pod> {
    sequence: AtomicUsize,
    value: Atomic<T>,
}

impl<T: Default + Copy + Send + Sync + Pod> Slot<T> {
    fn new(index: usize) -> Self {
        Self {
            sequence: AtomicUsize::new(index),
            value: Atomic::new(T::default()),
        }
    }
}
/// Thread-safe and lock-free ring buffer implementation using standard Rust constructs.
///
/// The capacity of the ring buffer is a power of 2 (2^N).
/// T: The type of elements stored in the ring buffer. Must be a primitive type that
/// implements Default and Copy traits.
///
/// Note: the idea would be to use that structure for AtomicPrimitive types only, but Rust
/// does not provide a trait to constraint T to be an atomic primitive type.
/// Therefore, we use Default + Copy as constraints for T.
#[derive(Debug)]
pub struct LockFreeRingBuffer<T: Default + Copy + Send + Sync + Pod, const POW2N: usize> {
    ring_buffer: Vec<Slot<T>>,
    push_index: AtomicUsize,
    pop_index: AtomicUsize,
}

/// Implementation of the LockFreeRingBuffer methods.
impl<T: Default + Copy + Send + Sync + Pod, const POW2N: usize> LockFreeRingBuffer<T, POW2N> {
    const RING_BUFFER_SIZE: usize = 1 << POW2N;
    const RING_BUFFER_MASK: usize = Self::RING_BUFFER_SIZE - 1;

    /// Creates a new LockFreeRingBuffer with a power of 2 capacity.
    pub fn new() -> Self {
        let mut tmp_buffer = Vec::with_capacity(Self::RING_BUFFER_SIZE);

        // Initialize each slot with its initial sequence number.
        for index in 0..Self::RING_BUFFER_SIZE {
            tmp_buffer.push(Slot::new(index));
        }

        LockFreeRingBuffer {
            ring_buffer: tmp_buffer,
            push_index: AtomicUsize::new(0),
            pop_index: AtomicUsize::new(0),
        }
    }

    /// Adds an item to the back of the ring buffer.
    pub fn enqueue(&self, item: T) -> Result<(), &'static str> {
        loop {
            let write_index = self.push_index.load(Ordering::Acquire);
            let slot = &self.ring_buffer[write_index & Self::RING_BUFFER_MASK];
            let sequence = slot.sequence.load(Ordering::Acquire);
            let dif = sequence as isize - write_index as isize;

            if dif == 0 {
                if self
                    .push_index
                    .compare_exchange_weak(
                        write_index,
                        write_index.wrapping_add(1),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    slot.value.store(item, Ordering::Relaxed);
                    slot.sequence
                        .store(write_index.wrapping_add(1), Ordering::Release);
                    return Ok(());
                }

                continue;
            }

            if dif < 0 {
                return Err("Buffer is full");
            }
        }
    }

    /// Removes and returns an item from the front of the ring buffer.
    pub fn dequeue(&self) -> Option<T> {
        loop {
            let read_index = self.pop_index.load(Ordering::Acquire);
            let slot = &self.ring_buffer[read_index & Self::RING_BUFFER_MASK];
            let sequence = slot.sequence.load(Ordering::Acquire);
            let dif = sequence as isize - read_index.wrapping_add(1) as isize;

            if dif == 0 {
                if self
                    .pop_index
                    .compare_exchange_weak(
                        read_index,
                        read_index.wrapping_add(1),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    let item = slot.value.load(Ordering::Relaxed);
                    slot.sequence.store(
                        read_index.wrapping_add(Self::RING_BUFFER_SIZE),
                        Ordering::Release,
                    );
                    return Some(item);
                }

                continue;
            }

            if dif < 0 {
                return None;
            }
        }
    }

    pub fn capacity(&self) -> usize {
        Self::RING_BUFFER_SIZE
    }
}

/// Default implementation for LockFreeRingBuffer.
impl<T: Default + Copy + Send + Sync + Pod, const POW2N: usize> Default
    for LockFreeRingBuffer<T, POW2N>
{
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for LockFreeRingBuffer.
#[cfg(test)]
mod tests {
    use super::LockFreeRingBuffer;

    // basic test for enqueue and dequeue
    #[test]
    fn test_enqueue_dequeue() {
        let ring_buffer: LockFreeRingBuffer<u32, 3> = LockFreeRingBuffer::new(); // Capacity 8
        assert_eq!(ring_buffer.enqueue(1), Ok(()));
        assert_eq!(ring_buffer.enqueue(2), Ok(()));
        assert_eq!(ring_buffer.dequeue(), Some(1));
        assert_eq!(ring_buffer.dequeue(), Some(2));
        assert_eq!(ring_buffer.dequeue(), None);
    }

    // test for wrap around
    #[test]
    fn test_full_buffer() {
        let ring_buffer: LockFreeRingBuffer<u32, 2> = LockFreeRingBuffer::new(); // Capacity 4
        assert_eq!(ring_buffer.enqueue(1), Ok(()));
        assert_eq!(ring_buffer.enqueue(2), Ok(()));
        assert_eq!(ring_buffer.enqueue(3), Ok(()));
        assert_eq!(ring_buffer.enqueue(4), Ok(()));
        assert_eq!(ring_buffer.enqueue(5), Err("Buffer is full"));
    }

    // test for empty buffer
    #[test]
    fn test_empty_buffer() {
        let ring_buffer: LockFreeRingBuffer<u32, 2> = LockFreeRingBuffer::new(); // Capacity 4
        assert_eq!(ring_buffer.dequeue(), None);
    }

    // test for capacity
    #[test]
    fn test_capacity() {
        let ring_buffer: LockFreeRingBuffer<u32, 4> = LockFreeRingBuffer::new(); // Capacity 16
        assert_eq!(ring_buffer.capacity(), 16);
    }

    // Additional test with two threads
    #[test]
    fn test_concurrent_enqueue_dequeue() {
        use std::sync::Arc;
        use std::thread;

        let ring_buffer: Arc<LockFreeRingBuffer<u32, 4>> = Arc::new(LockFreeRingBuffer::new()); // Capacity 16  
        let rb_producer = ring_buffer.clone();
        let rb_consumer = ring_buffer.clone();

        let producer = thread::spawn(move || {
            for i in 0..10 {
                loop {
                    if rb_producer.enqueue(i).is_ok() {
                        break;
                    }
                }
            }
        });

        let consumer = thread::spawn(move || {
            let mut sum = 0;
            for _ in 0..10 {
                loop {
                    if let Some(value) = rb_consumer.dequeue() {
                        sum += value;
                        break;
                    }
                }
            }
            sum
        });

        producer.join().unwrap();
        let result = consumer.join().unwrap();
        assert_eq!(result, 45); // Sum of numbers from 0 to 9
    }

    // stress test with multiple producers and one consumer
    #[test]
    fn test_mpsc_stress_no_loss_no_duplicates() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;

        const PRODUCERS: usize = 4;
        const ITEMS_PER_PRODUCER: usize = 2_000;
        const TOTAL_ITEMS: usize = PRODUCERS * ITEMS_PER_PRODUCER;

        let ring_buffer: Arc<LockFreeRingBuffer<u32, 8>> = Arc::new(LockFreeRingBuffer::new());
        let seen: Arc<Vec<AtomicUsize>> = Arc::new(
            (0..TOTAL_ITEMS)
                .map(|_| AtomicUsize::new(0))
                .collect::<Vec<_>>(),
        );

        let mut producer_handles = Vec::new();
        for producer_id in 0..PRODUCERS {
            let rb = ring_buffer.clone();
            producer_handles.push(thread::spawn(move || {
                let base = producer_id * ITEMS_PER_PRODUCER;
                for offset in 0..ITEMS_PER_PRODUCER {
                    let value = (base + offset) as u32;
                    loop {
                        if rb.enqueue(value).is_ok() {
                            break;
                        }
                        std::thread::yield_now();
                    }
                }
            }));
        }

        let rb_consumer = ring_buffer.clone();
        let seen_consumer = seen.clone();
        let consumer = thread::spawn(move || {
            let mut consumed = 0usize;
            while consumed < TOTAL_ITEMS {
                if let Some(value) = rb_consumer.dequeue() {
                    let index = value as usize;
                    seen_consumer[index].fetch_add(1, Ordering::AcqRel);
                    consumed += 1;
                } else {
                    std::thread::yield_now();
                }
            }
        });

        for handle in producer_handles {
            handle.join().unwrap();
        }
        consumer.join().unwrap();

        for entry in seen.iter() {
            assert_eq!(entry.load(Ordering::Acquire), 1);
        }
    }

    // stress test with multiple producers and multiple consumers
    #[test]
    fn test_mpmc_stress_no_loss_no_duplicates() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;

        const PRODUCERS: usize = 4;
        const CONSUMERS: usize = 4;
        const ITEMS_PER_PRODUCER: usize = 2_000;
        const TOTAL_ITEMS: usize = PRODUCERS * ITEMS_PER_PRODUCER;

        let ring_buffer: Arc<LockFreeRingBuffer<u32, 8>> = Arc::new(LockFreeRingBuffer::new());
        let seen: Arc<Vec<AtomicUsize>> = Arc::new(
            (0..TOTAL_ITEMS)
                .map(|_| AtomicUsize::new(0))
                .collect::<Vec<_>>(),
        );
        let consumed_count = Arc::new(AtomicUsize::new(0));

        let mut producer_handles = Vec::new();
        for producer_id in 0..PRODUCERS {
            let rb = ring_buffer.clone();
            producer_handles.push(thread::spawn(move || {
                let base = producer_id * ITEMS_PER_PRODUCER;
                for offset in 0..ITEMS_PER_PRODUCER {
                    let value = (base + offset) as u32;
                    loop {
                        if rb.enqueue(value).is_ok() {
                            break;
                        }
                        std::thread::yield_now();
                    }
                }
            }));
        }

        let mut consumer_handles = Vec::new();
        for _ in 0..CONSUMERS {
            let rb = ring_buffer.clone();
            let seen_consumer = seen.clone();
            let consumed = consumed_count.clone();
            consumer_handles.push(thread::spawn(move || {
                loop {
                    if consumed.load(Ordering::Acquire) >= TOTAL_ITEMS {
                        break;
                    }

                    if let Some(value) = rb.dequeue() {
                        let index = value as usize;
                        seen_consumer[index].fetch_add(1, Ordering::AcqRel);
                        consumed.fetch_add(1, Ordering::AcqRel);
                    } else {
                        std::thread::yield_now();
                    }
                }
            }));
        }

        for handle in producer_handles {
            handle.join().unwrap();
        }
        for handle in consumer_handles {
            handle.join().unwrap();
        }

        assert_eq!(consumed_count.load(Ordering::Acquire), TOTAL_ITEMS);
        for entry in seen.iter() {
            assert_eq!(entry.load(Ordering::Acquire), 1);
        }
    }

    // test for Default trait
    #[test]
    fn test_default_trait() {
        let ring_buffer: LockFreeRingBuffer<u32, 3> = LockFreeRingBuffer::default(); // Capacity 8
        assert_eq!(ring_buffer.capacity(), 8);
    }
}
