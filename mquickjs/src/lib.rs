//! Safe, idiomatic wrapper over the mquickjs C library.
//!
//! This crate provides a `Context` for evaluating JavaScript and working with
//! mquickjs values in Rust.

mod context;
mod error;
mod func;
mod value;

pub use context::Context;
pub use error::JsError;
pub use value::Value;
