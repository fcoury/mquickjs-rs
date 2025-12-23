//! Unsafe, low-level FFI bindings for the mquickjs C library.
//!
//! This crate is generated via bindgen and exposes the raw C API. All functions
//! and types are unsafe to use directly.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(improper_ctypes)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
