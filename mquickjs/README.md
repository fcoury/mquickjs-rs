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
use mquickjs_rs::{Context, Runtime, Value};

let runtime = Runtime::new().expect("runtime should initialize");
let ctx = runtime.context().expect("context should initialize");

let sum = ctx.eval_i32("1 + 2 + 3", "example").expect("eval should succeed");
assert_eq!(sum, 6);

ctx.register_fn("echo", |args: &[Value<'_>]| Ok(args[0]))
    .expect("register should succeed");

let echoed = ctx
    .eval_string("echo('hello')", "example")
    .expect("eval should succeed");
assert_eq!(echoed, "hello");
```

## Conversions

```rust
use mquickjs_rs::{Context, FromValue, IntoValue, Runtime};

let runtime = Runtime::new().expect("runtime should initialize");
let ctx = runtime.context().expect("context should initialize");

let value = 42i32.into_value(&ctx).expect("convert into JS");
let result = i32::from_value(value).expect("convert back");
assert_eq!(result, 42);
```

## Objects and arrays

```rust
use mquickjs_rs::{Array, Context, Object, Runtime};

let runtime = Runtime::new().expect("runtime should initialize");
let ctx = runtime.context().expect("context should initialize");

let obj_value = ctx.eval("({})", "example").expect("eval should succeed");
let obj = Object::from_value(&ctx, obj_value).expect("object should wrap");
obj.set("name", "mquickjs").expect("set should succeed");
let name: String = obj.get("name").expect("get should succeed");
assert_eq!(name, "mquickjs");

let array_value = ctx.eval("[]", "example").expect("eval should succeed");
let array = Array::from_value(&ctx, array_value).expect("array should wrap");
array.push(1i32).expect("push should succeed");
let first: i32 = array.get(0).expect("get should succeed");
assert_eq!(first, 1);
```

## Persistent handles

```rust
use mquickjs_rs::{Context, Persistent, Runtime};

let runtime = Runtime::new().expect("runtime should initialize");
let ctx = runtime.context().expect("context should initialize");

let value = ctx.eval("'hello'", "example").expect("eval should succeed");
let persistent = Persistent::new(&ctx, value).expect("persist should succeed");
let roundtrip = persistent.to_value().to_string().expect("to_string");
assert_eq!(roundtrip, "hello");
```

## Notes

- MicroQuickJS runs in a stricter ES5-like mode. See `../../mquickjs/README.md` for engine limitations.
- The JS context requires a preallocated memory buffer (minimum 1024 bytes).

## License

MIT
