# mquickjs-sys

Low-level, unsafe Rust bindings to the MicroQuickJS engine.

Most users should prefer the safe wrapper crate: `mquickjs-rs`.

## Installation

```bash
cargo add mquickjs-sys
```

Or in `Cargo.toml`:

```toml
mquickjs-sys = "0.1.0"
```

## Notes

- Requires a C compiler supported by the `cc` crate.
- The engine runs in a stricter ES5-like mode. See `../../mquickjs/README.md` for engine limitations.

## License

MIT
