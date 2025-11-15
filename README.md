# PublishSubscribe Rust

Agnostic, Lightweight and Portable Publish-Subscribe Helper

- written in Rust using traits
- synchronous/asynchronous observer
- topic subscription

Goodies:

- thread-safe dictionary helper on top of `BTreeMap`
- thread-safe contiguous queue on top of `VecDeque`
- lock-free ring buffer on top of `Vec`
- waitable object on top of `Mutex` and `Condvar`
- periodic task helper
- worker task and worker pool helper
- queuable commands
- a simple FSM example based on Enum state and methods

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

## Author

Laurent Lardinois / Type One (TFL-TDV)

[LinkedIn profile](https://be.linkedin.com/in/laurentlardinois)

[Demozoo profile](https://demozoo.org/sceners/19691/)
