//-----------------------------------------------------------------------------//
// Rust Publish/Subscribe Pattern - Spare time development for fun             //
// (c) 2025-2026 Laurent Lardinois https://be.linkedin.com/in/laurentlardinois //
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

//! Custom pool allocator that caches and reuses small, fixed-power-of-two
//! blocks of memory to reduce heap fragmentation from frequent small
//! allocations (e.g. events/messages). Ported from the C++ framework's
//! `mem_pool_allocator`. Only active when the `pool_allocator` Cargo feature
//! is enabled; install it as the process-wide allocator with
//! `#[global_allocator]` (see `src/main.rs`).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Mutex;

/// Smallest cached block size: 2^4 = 16 bytes.
const MIN_CACHED_BLOCK_POW2: u32 = 4;
/// Largest cached block size: 2^9 = 512 bytes. Bigger requests bypass the pool.
const MAX_CACHED_BLOCK_POW2: u32 = 9;
const MIN_CACHED_BLOCK_SIZE: usize = 1 << MIN_CACHED_BLOCK_POW2;
const MAX_CACHED_BLOCK_SIZE: usize = 1 << MAX_CACHED_BLOCK_POW2;
/// Number of distinct power-of-two size classes cached (16, 32, 64, 128, 256, 512).
const NUM_SIZE_CLASSES: usize = (MAX_CACHED_BLOCK_POW2 - MIN_CACHED_BLOCK_POW2 + 1) as usize;
/// Maximum number of blocks cached per size class (2^9 = 512).
const POOL_CAPACITY: usize = 512;

/// Fixed-size, statically allocated free-list for one power-of-two size class.
/// Never touches the heap itself so it is safe to use from inside the
/// allocator it backs.
struct SizeClassPool {
    storage: Mutex<PoolStorage>,
}

struct PoolStorage {
    // Freed block addresses; slots [0, len) are occupied.
    addresses: [usize; POOL_CAPACITY],
    len: usize,
}

impl SizeClassPool {
    const fn new() -> Self {
        SizeClassPool {
            storage: Mutex::new(PoolStorage {
                addresses: [0; POOL_CAPACITY],
                len: 0,
            }),
        }
    }

    fn try_pop(&self) -> Option<usize> {
        let mut storage = self.storage.lock().unwrap();
        if storage.len == 0 {
            return None;
        }
        storage.len -= 1;
        Some(storage.addresses[storage.len])
    }

    fn try_push(&self, address: usize) -> bool {
        let mut storage = self.storage.lock().unwrap();
        if storage.len == POOL_CAPACITY {
            return false;
        }
        let index = storage.len;
        storage.addresses[index] = address;
        storage.len += 1;
        true
    }
}

/// Rounds `size` up to the nearest cached power-of-two block size.
fn pow2_block_size(size: usize) -> usize {
    size.next_power_of_two().max(MIN_CACHED_BLOCK_SIZE)
}

fn size_class_index(size_pow2: usize) -> usize {
    (size_pow2.trailing_zeros() - MIN_CACHED_BLOCK_POW2) as usize
}

/// Global allocator that caches and recycles small power-of-two blocks
/// instead of returning them to the system allocator immediately.
///
/// Recycled blocks are only reused for requests with `align <= MIN_CACHED_BLOCK_SIZE`;
/// larger alignment requests always go straight to `System`. This mirrors the
/// C++ reference implementation, which never recycles blocks for the (rare)
/// over-aligned allocation case either.
pub struct PoolAllocator {
    pools: [SizeClassPool; NUM_SIZE_CLASSES],
}

impl PoolAllocator {
    /// Creates a new PoolAllocator with all size-class pools empty.
    pub const fn new() -> Self {
        PoolAllocator {
            pools: [
                SizeClassPool::new(),
                SizeClassPool::new(),
                SizeClassPool::new(),
                SizeClassPool::new(),
                SizeClassPool::new(),
                SizeClassPool::new(),
            ],
        }
    }
}

impl Default for PoolAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: alloc/dealloc always operate on a `Layout` whose size is rounded
// up deterministically (`pow2_block_size`) before touching a pool, so a
// block recycled for one request is always sized at least as large as any
// other request landing in the same size class. Pooling is skipped whenever
// `layout.align() > MIN_CACHED_BLOCK_SIZE`, so every recycled block was
// itself originally obtained from `System` with a small (<= 16 byte)
// alignment requirement, which `System` satisfies for any such request on
// all supported target platforms.
unsafe impl GlobalAlloc for PoolAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size_pow2 = pow2_block_size(layout.size());
        if size_pow2 <= MAX_CACHED_BLOCK_SIZE && layout.align() <= MIN_CACHED_BLOCK_SIZE {
            if let Some(address) = self.pools[size_class_index(size_pow2)].try_pop() {
                return address as *mut u8;
            }
            let pow2_layout = Layout::from_size_align(size_pow2, layout.align()).unwrap_or(layout);
            // SAFETY: pow2_layout has a valid non-zero size and the caller's alignment.
            return unsafe { System.alloc(pow2_layout) };
        }
        // SAFETY: forwarding the caller's layout unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size_pow2 = pow2_block_size(layout.size());
        if size_pow2 <= MAX_CACHED_BLOCK_SIZE && layout.align() <= MIN_CACHED_BLOCK_SIZE {
            if self.pools[size_class_index(size_pow2)].try_push(ptr as usize) {
                return;
            }
            let pow2_layout = Layout::from_size_align(size_pow2, layout.align()).unwrap_or(layout);
            // SAFETY: pow2_layout matches the layout actually used to obtain this
            // block from `System` in `alloc` (same deterministic rounding rule).
            unsafe { System.dealloc(ptr, pow2_layout) };
            return;
        }
        // SAFETY: this block was obtained directly from `System` with this exact layout.
        unsafe { System.dealloc(ptr, layout) };
    }
}

// Unit tests for PoolAllocator. These exercise the allocator directly
// (without installing it as the process-wide `#[global_allocator]`).
#[cfg(test)]
mod tests {
    use super::{GlobalAlloc, Layout, MIN_CACHED_BLOCK_SIZE, PoolAllocator, pow2_block_size};

    // test pow2 rounding rules
    #[test]
    fn test_pow2_block_size() {
        assert_eq!(pow2_block_size(1), MIN_CACHED_BLOCK_SIZE);
        assert_eq!(pow2_block_size(16), 16);
        assert_eq!(pow2_block_size(17), 32);
        assert_eq!(pow2_block_size(500), 512);
    }

    // test that a freed small block is recycled by a subsequent same-class allocation
    #[test]
    fn test_alloc_dealloc_recycles_block() {
        let allocator = PoolAllocator::new();
        let layout = Layout::from_size_align(16, 8).unwrap();

        unsafe {
            let first = allocator.alloc(layout);
            assert!(!first.is_null());
            allocator.dealloc(first, layout);

            let second = allocator.alloc(layout);
            assert_eq!(first, second, "expected the freed block to be recycled");
            allocator.dealloc(second, layout);
        }
    }

    // test that oversized allocations bypass the pool entirely
    #[test]
    fn test_large_allocation_bypasses_pool() {
        let allocator = PoolAllocator::new();
        let layout = Layout::from_size_align(4096, 8).unwrap();

        unsafe {
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());
            allocator.dealloc(ptr, layout);
        }
    }

    // test basic read/write through a pooled allocation
    #[test]
    fn test_allocated_memory_is_usable() {
        let allocator = PoolAllocator::new();
        let layout = Layout::from_size_align(64, 8).unwrap();

        unsafe {
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());
            for i in 0..64u8 {
                *ptr.add(i as usize) = i;
            }
            for i in 0..64u8 {
                assert_eq!(*ptr.add(i as usize), i);
            }
            allocator.dealloc(ptr, layout);
        }
    }
}
