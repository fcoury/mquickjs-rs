# mquickjs-rs

Safe, idiomatic Rust wrapper for the MicroQuickJS engine.

## Installation

```bash
cargo add mquickjs-rs
```

Or in `Cargo.toml`:

```toml
mquickjs-rs = "0.1.0"
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

- MicroQuickJS runs in a stricter ES5-like mode. See `../../mquickjs/README.md` for engine limitations.
- The JS context requires a preallocated memory buffer (minimum 1024 bytes).

## License

MIT
