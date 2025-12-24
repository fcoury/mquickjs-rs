//! Safe, idiomatic wrapper over the mquickjs C library.
//!
//! This crate provides a [`Context`] for evaluating JavaScript and working with
//! mquickjs values in Rust.
//!
//! # Quick start
//!
//! ```no_run
//! use mquickjs_rs::Context;
//!
//! let ctx = Context::new(1024 * 1024).expect("context should initialize");
//! let value = ctx.eval_i32("1 + 2", "example").expect("eval should succeed");
//! assert_eq!(value, 3);
//! ```

mod context;
mod convert;
mod error;
mod func;
mod object;
mod rooted;
mod value;

pub use context::Context;
pub use convert::{FromValue, IntoValue};
pub use error::JsError;
pub use object::{Array, Object};
pub use rooted::RootedValue;
pub use value::Value;
