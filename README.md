# PublishSubscribe Rust

Agnostic, Lightweight and Portable Publish-Subscribe Helper

- written in Rust using traits
- synchronous/asynchronous observer
- topic subscription

Goodies:

- simple thread-safe dictionary helper on top of `BTreeMap`
- simple thread-safe contiguous queue on top of `VecDeque`
- simple waitable object on top of `Mutex` and `Condvar`
- simple periodic task helper
- simple worker task helper
- queuable commands

[GitHub repository](https://github.com/type-one/PublishSubscribeRust)

## What

Small test program written in Rust to implement a simple Publish/Subscribe pattern.
The code is portable and lightweight.

## Why

An attempt to write a flexible little framework that can be used on desktop PCs and embedded systems
(micro-computers and micro-controllers) that are able to compile and run Rust code.

## How

Can be compiled on Linux and Windows, and should be easily
adapted for other platforms (micro-computers, micro-controllers)

```bash
cargo build
cargo run
```

## Author

Laurent Lardinois / Type One (TFL-TDV)

[LinkedIn profile](https://be.linkedin.com/in/laurentlardinois)

[Demozoo profile](https://demozoo.org/sceners/19691/)
