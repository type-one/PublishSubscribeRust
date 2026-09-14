# PublishSubscribe Rust

Agnostic, Lightweight and Portable Publish-Subscribe Helper

- written in Rust using traits
- synchronous/asynchronous observer
- topic subscription

Goodies:

- thread-safe dictionary helper (`SyncDictionary`), generic over its backing
  associative container (`BTreeMap` by default, or `HashMap`), with batch
  insertion via `add_range()`
- thread-safe contiguous queue on top of `VecDeque`
- thread-safe bounded ring vector on top of a preallocated `VecDeque`
- thread-safe priority queue on top of `BinaryHeap` (min-heap by default)
- fixed-capacity ring buffer (`RingBuffer`) with reject-on-full or
  overwrite-on-full (evict oldest) push modes, and a thread-safe
  `SyncRingBuffer` wrapper
- `SyncQueue`, `SyncVector`, `SyncPriorityQueue`, and `SyncRingBuffer` all
  share a queue-compatible API (`front_pop`, batch `push_range`/`pop_range`,
  and `push_overwrite`/`push_range_overwrite` where bounded) so they can be
  used interchangeably
- chronological `TimeList`/`SyncTimeList` helpers: keyed by timestamp
  (integral or `Instant`/`SystemTime`), always expose the earliest entry
  first via `top()`/`top_pop()`, and `snapshot_sorted()` returns all entries
  earliest-to-latest without draining
- lock-free ring buffer on top of `Vec`
- waitable object on top of `Mutex` and `Condvar`
- periodic task helper
- worker task and worker pool helper, both with an async `delegate_async()`
  variant for request/response style jobs: `WorkerPool::delegate_async()`
  returns a `tokio::task::JoinHandle<R>`, `WorkerTask::delegate_async()`
  returns a `tokio::sync::oneshot::Receiver<R>`; chain continuations with
  plain `.await` (instead of a `future<T>::then()`-style API) and fan
  multiple handles in with `tokio::join!`/`tokio::task::JoinSet`
- queuable commands
- a simple FSM example based on Enum state and methods
- `AsyncObserver` supports pluggable event storage (unbounded `SyncQueue`,
  bounded `SyncVector` or `SyncRingBuffer`, or priority-ordered
  `SyncPriorityQueue`); bounded observers report dropped events through
  `has_queue_overflow()`, `queue_overflow_count()`, and
  `consume_queue_overflow_count()`
- optional custom pool allocator (`PoolAllocator`) caching and reusing small
  power-of-two blocks to reduce heap fragmentation from frequent
  events/messages; opt-in via the `pool_allocator` Cargo feature, installed
  as the process-wide `#[global_allocator]`

[GitHub repository](https://github.com/type-one/PublishSubscribeRust)

## What

Small test program written in Rust to implement a simple Publish/Subscribe pattern.
The code is portable and lightweight.

## Why

An attempt to write a flexible little framework that can be used on desktop PCs and embedded systems
(micro-computers and micro-controllers) that are able to compile and run Rust code.

## How

Can be compiled on Linux and Windows, and should be easily
adapted for other platforms (Mac, micro-computers, micro-controllers) as long as they have a Rust tool-chain.

```bash
cargo build
cargo run
```

To build and run with the custom pool allocator enabled instead of the
default system allocator:

```bash
cargo build --features pool_allocator
cargo run --features pool_allocator
```

To run unit tests:

```bash
cargo test
```

To format the code:

```bash
cargo fmt
```

To check unit tests coverage of the helper tools:

```bash
cargo tarpaulin --lib --exclude-files *_test.rs
```

See more at [tarpaulin](https://github.com/xd009642/tarpaulin)

## Author

Laurent Lardinois / Type One (TFL-TDV)

[LinkedIn profile](https://be.linkedin.com/in/laurentlardinois)

[Demozoo profile](https://demozoo.org/sceners/19691/)
