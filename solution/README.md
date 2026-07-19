# Keyed Rate Limiter (Rust)

An in-process, keyed rate limiter with two strategies:

- Token bucket — a bucket of capacity permits refilling continuously at refill_per_sec. Absorbs bursts up to capacity, then paces to the refill rate.
- Sliding window — at most limit grants inside any trailing window, tracked with an exact log of grant timestamps.

## Setup

Needs a Rust toolchain (1.70+; developed on 1.95). Install via

install using curl if it fails kindly read the official documentation for installing on different systems (https://rust-lang.org/tools/install/)
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Run everything in one command (compile and run)

```sh
cargo run --release
```

The command above runs the demo, which prints a labeled section for each of the eight requirements with millisecond stamped log messahe and asserts behavior as it executes

There are also pure-logic unit tests for the parts that don't depend on real-clock timing:

```sh
cargo test --release
```