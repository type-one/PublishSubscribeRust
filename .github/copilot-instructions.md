# GitHub Copilot Instructions

## Quick Context

This is the PublishSubscribeRust project (Cargo, Rust 2024 edition), a
portable, lightweight publish/subscribe framework with a small collection of
concurrency and utility helpers.

First-party code:

- `src/tools/` (sync/async observers, queues, dictionary, worker pool,
  periodic task, ring buffer, histogram, etc.)
- `src/examples/` (basic, advanced, FSM, JSON, cJSON demonstrations)
- `src/lib.rs`, `src/main.rs`
- `Cargo.toml`, `build.rs`, `deny.toml`, `README.md`

Vendor code:

- `src/cJSON/` (upstream C library, edit only when explicitly requested)

## What To Edit By Default

- Prefer `src/tools/` and `src/examples/`.
- Keep edits minimal and targeted.
- Avoid wide refactors unless requested.

## Compatibility Rules

- Target the Rust edition and toolchain declared in `Cargo.toml` (currently
  edition 2024). Do not silently bump the edition or MSRV.
- Keep `src/cJSON/` and `build.rs` untouched unless the task is explicitly
  about the C interop layer.
- New features behind optional behavior should use Cargo features (see
  `basic_tests`, `advanced_tests`, `fsm`, `all_tests` in `Cargo.toml`) rather
  than ad-hoc `cfg` flags.

## Build/Run/Test Reference

- Build: `cargo build`
- Run: `cargo run`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Format: `cargo fmt`
- Supply-chain/license checks: `cargo deny check` (see `deny.toml`)

## Coding Conventions

- Follow `rustfmt` defaults (`cargo fmt`); do not hand-format code that
  `rustfmt` would reformat.
- Follow idiomatic Rust naming: `snake_case` for functions/modules/variables,
  `UpperCamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Prefer descriptive identifiers over short/cryptic names.
- Keep module layout consistent with `src/tools/` (one concern per file, one
  `pub mod` entry added to `src/lib.rs`).

## API Design Preferences

- Keep public APIs simple, explicit, and consistent with existing
  `src/tools/*` style (constructors named `new`, optional bounded variants
  named `with_capacity`, accessors as small `pub fn` methods).
- Prefer returning `Option<T>` / `Result<T, E>` over panicking in library
  code; reserve `panic!`/`unwrap`/`expect` for programmer errors, tests, and
  `main.rs`/examples.
- Use `thiserror`-style explicit error enums (or simple structs) instead of
  stringly-typed errors when a new fallible API is introduced.

## Style and Naming Details

- Use variable and parameter names with at least three characters, except
  for conventional loop indices (`i`, `j`) in very small local scopes.
- Enforce immutability by default: only mark bindings `mut` when they are
  actually reassigned, and prefer `const`/`static` for compile-time values.
- Do not use magic numbers; replace them with named `const` values.
- Keep struct fields private by default; expose behavior through methods.
- Derive standard traits (`Debug`, `Clone`, `Default`, `PartialEq`, etc.)
  instead of hand-writing them when semantics allow it.
- Implement `Default` via `#[derive(Default)]` or by delegating to `new()`
  when a type already has a sensible `new()`.

## Standard Library and Crate Usage

- Prefer `std` containers and synchronization primitives (`VecDeque`,
  `RwLock`, `Mutex`, `Condvar`, `Arc`, `atomic::*`) and the facilities in
  `src/tools/` over custom data structures when they provide the required
  behavior.
- Do not re-implement functionality the standard library or an already
  declared dependency provides correctly and portably.
- Justify any new external dependency in `Cargo.toml`: keep the dependency
  list lean, and prefer `std` first.
- Prefer iterators and combinators (`map`, `filter`, `collect`, ...) over
  manual index loops when they improve clarity, but avoid overly clever
  chains that hurt readability.
- Prefer `?` and `From`/`Into` error conversions over manual `match`
  boilerplate for error propagation.

## Generic and Trait API Design

- Follow the established style in `src/tools/*`: keep concrete, easy-to-call
  APIs for common paths and add generic/trait-bounded APIs only when they
  provide real value.
- Bound generic type parameters with the minimal trait set actually required
  (e.g. `Send + Sync + 'static` for types shared across threads, `Clone`
  only where cloning is needed).
- Prefer trait objects (`dyn Trait`) only when runtime polymorphism is
  required; prefer generics/monomorphization for hot paths.
- Avoid `unsafe` unless there is a narrow, well-documented, and justified
  need; when used, keep the `unsafe` block minimal and add a `// SAFETY:`
  comment explaining the invariant that makes it sound.

## Ownership and API Signatures

- Use RAII (`Drop`) for locks, timers, threads, and other acquire/release
  lifecycles, mirroring the existing `Drop` impls in `src/tools/`.
- Use `Arc`/`Arc<Mutex/RwLock<T>>` to encode shared ownership across
  threads, consistent with `src/tools/sync_object.rs`,
  `src/tools/sync_queue.rs`, and `src/tools/async_observer.rs`.
- Pass `Copy` types by value, pass `&str`/`&[T]` instead of
  `&String`/`&Vec<T>` for read-only borrows, and take ownership (`String`,
  `Vec<T>`) only when the callee needs to store or mutate the data.
- Prefer borrowing (`&T`, `&mut T`) over cloning; clone only when ownership
  must cross a thread boundary or be stored independently.

## OOP and Clean Code

- Respect encapsulation: keep struct fields private and expose behavior
  through methods and traits (see `Observer` in
  `src/tools/sync_observer.rs`).
- Do not introduce global mutable state (`static mut`, ad-hoc singletons)
  unless a narrow, concrete requirement justifies it; prefer passing shared
  state via `Arc`.
- Prefer fixing root causes over layered workarounds.
- Use meaningful and searchable names instead of unnecessary abbreviations.
- Comments should explain why, not restate what the code already says.
- Keep each struct/module focused on one responsibility and functions short
  and focused.
- Prefer early returns and positive conditionals when they improve
  readability.
- Avoid boolean parameter traps; use small enums or separate constructors
  (e.g. `new()` vs `with_capacity()`) when a flag controls construction
  behavior.
- Keep cyclomatic complexity reasonable; extract non-trivial closures into
  named helper functions.

## Concurrency and Platform Abstractions

- Reuse the synchronization and task abstractions already present in
  `src/tools/` (`SyncObject`, `SyncQueue`, `SyncVector`, `SyncPriorityQueue`,
  `RingBuffer`, `SyncRingBuffer`, `AsyncObserver`, `WorkerTask`, `WorkerPool`,
  `PeriodicTask`) before introducing new primitives.
- Keep direct `std::thread`, `Mutex`, `RwLock`, and `Condvar` usage
  consistent with existing code and limited to cases where the local
  abstractions do not fit.
- Keep asynchronous callbacks and queue operations small; do not perform
  unnecessary blocking, allocation-heavy work, or complex business logic in
  latency-sensitive paths.
- `AsyncObserver` stores events in a pluggable container: `new()` uses an
  unbounded `SyncQueue`, `with_capacity(n)` uses a bounded, preallocated
  `SyncVector`, `with_ring_buffer_capacity::<N>()` uses a fixed-capacity
  `SyncRingBuffer`, and `with_priority()` uses a `SyncPriorityQueue` (requires
  `Topic`/`Evt` to be `Ord`) to deliver events in priority order instead of
  FIFO order. Bounded observers report entries dropped once full through
  `has_queue_overflow()`, `queue_overflow_count()`, and
  `consume_queue_overflow_count()`. Components using bounded observers should
  poll the consumed count and publish an explicit notification when dropped
  events matter to the application.
- Prefer `std::sync::atomic` with an explicit, minimal `Ordering` (usually
  `Relaxed` for simple counters) over a `Mutex<usize>` for single-value
  counters.

## Design Patterns

### Messages and Events

- Group related messages, commands, or events into an `enum` when the
  domain has a closed set of alternatives.
- Dispatch enums with `match` and focused arms instead of chains of
  string/tag comparisons.
- Prefer `enum` variants with named fields so each alternative carries
  semantic meaning.

### Finite State Machines

- When an explicit finite state machine is needed, model states as an
  `enum` (see `src/examples/fsm_test.rs`) and dispatch transitions with
  `match`. Give each meaningful state/event combination a focused handler
  and make unhandled combinations explicit (`_ =>` with a clear comment, not
  a silent no-op).

### Publish/Subscribe

- Use `tools::sync_observer::Observer` or `tools::async_observer::AsyncObserver`
  for observer behavior and the existing synchronized containers
  (`SyncQueue`, `SyncDictionary`) for queued delivery.
- Components should react to typed events through focused handlers rather
  than one monolithic callback.
- For bounded async observers, treat queue overflow as an observable event
  when losing messages could affect correctness or diagnosis.

## Documentation Conventions

- Use `///` doc comments for public modules, structs, enums, traits, and
  functions; use `//!` for module-level documentation where applicable.
- Keep doc comments short and focused: state what the item does and any
  non-obvious constraint; avoid restating the signature.
- Use `//` line comments only to explain why, not what, for non-obvious
  implementation details.
- Update `README.md` when public behavior, APIs, or the feature list
  change.

## Error Handling and Testing

- Prefer `Result<T, E>` and `Option<T>` over panics for recoverable and
  absent-value cases in library code (`src/tools/`, `src/lib.rs`).
- Avoid `unwrap()`/`expect()` in library code paths that can be reached with
  invalid input; they remain acceptable in tests, examples, and `main.rs`
  for conditions that indicate programmer error.
- For new or changed behavior, add `#[cfg(test)] mod tests` unit tests
  covering both the success path and edge/failure paths, following the
  existing test style in `src/tools/*.rs`.
- Run `cargo test`, and `cargo clippy --all-targets --all-features -- -D
  warnings` before considering a change complete.

## Preferred Contribution Pattern

1. Read the nearby reference implementation in `src/tools/` and relevant
   usage in `src/examples/`.
2. Make the smallest localized change that satisfies the behavior.
3. Keep naming, formatting, ownership, and documentation consistent with
   this file and the existing code.
4. Add or update focused unit tests for both normal and failure behavior
   when the change can fail.
5. Run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D
   warnings`, and `cargo test` before finishing.
6. Update `README.md` and examples when public behavior or API usage
   changes.

## Idiomatic Rust Additions

- Prefer `impl Trait` or generics over `Box<dyn Trait>` in return position
  unless dynamic dispatch or type erasure is genuinely required.
- Use `if let` / `while let` / `?` instead of manual `match` on
  `Option`/`Result` when it improves readability.
- Prefer `#[derive(...)]` over manual trait implementations unless custom
  behavior is required.
- Use `String`/`&str` and `Vec<T>`/`&[T]` idiomatically; avoid needless
  `.to_string()`/`.clone()` calls.
- Keep `unsafe` out of first-party code unless explicitly required and
  documented; the `src/cJSON` FFI boundary in `build.rs`/`main.rs` is the
  one place where `unsafe` is expected.
