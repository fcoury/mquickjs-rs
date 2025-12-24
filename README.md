# mquickjs-rs

Safe and low-level Rust bindings for the MicroQuickJS (mquickjs) JavaScript engine.

This workspace contains two crates:

- `mquickjs-sys`: raw, unsafe FFI bindings to the mquickjs C API
- `mquickjs-rs`: safe, idiomatic Rust wrapper built on top of `mquickjs-sys`

## Requirements

- Rust stable toolchain (edition 2024)
- A C compiler supported by the `cc` crate
- Submodules initialized in the parent repository

## Install from crates.io

```bash
cargo add mquickjs-rs
```

Or in `Cargo.toml`:

```toml
mquickjs-rs = "0.1.0"
```

## Quick Start

Run the tests from the workspace root:

```bash
cargo test -p mquickjs-sys
cargo test -p mquickjs-rs
```

Generate documentation:

```bash
cargo doc -p mquickjs-rs --no-deps
```

Run the example program:

```bash
cargo run -p mquickjs-rs --example eval
```

## Usage

```rust
use mquickjs_rs::{Context, Value};

let ctx = Context::new(1024 * 1024).expect("context should initialize");

let sum = ctx.eval_i32("1 + 2 + 3", "example").expect("eval should succeed");
assert_eq!(sum, 6);

ctx.register_fn("echo", |args: &[Value<'_>]| Ok(args[0]))
    .expect("register should succeed");

let echoed = ctx
    .eval_string("echo('hello')", "example")
    .expect("eval should succeed");
assert_eq!(echoed, "hello");
```

## Notes

- MicroQuickJS runs in a stricter ES5-like mode. See `../mquickjs/README.md` for engine limitations.
- The JS context requires a preallocated memory buffer (minimum 1024 bytes).

## License

See the parent repository for licensing details.
