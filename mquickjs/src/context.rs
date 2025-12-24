use std::ffi::c_void;
use std::ptr::NonNull;

use mquickjs_sys::{js_stdlib, JSContext, JS_FreeContext, JS_NewContext};

use crate::error::JsError;
use crate::value::Value;

/// JavaScript execution context owning the underlying mquickjs state.
#[derive(Debug)]
pub struct Context {
    ctx: NonNull<JSContext>,
    _heap: Vec<usize>,
}

impl Context {
    /// Create a new JavaScript context with the given memory buffer size.
    pub fn new(memory_bytes: usize) -> Result<Self, JsError> {
        if memory_bytes < 1024 {
            return Err(JsError::ContextInit {
                message: "memory buffer must be at least 1024 bytes".to_string(),
            });
        }

        let word_size = std::mem::size_of::<usize>();
        let words = (memory_bytes + word_size - 1) / word_size;
        let mut heap = vec![0usize; words];
        let mem_start = heap.as_mut_ptr() as *mut c_void;
        let mem_size = heap.len() * word_size;

        let ctx = unsafe { JS_NewContext(mem_start, mem_size, &js_stdlib) };
        let ctx = NonNull::new(ctx).ok_or_else(|| JsError::ContextInit {
            message: "JS_NewContext returned null".to_string(),
        })?;

        Ok(Self { ctx, _heap: heap })
    }

    /// Evaluate a script and return a raw value wrapper.
    pub fn eval(&self, _script: &str, _filename: &str) -> Result<Value, JsError> {
        Err(JsError::Runtime {
            message: "not yet implemented".to_string(),
        })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            JS_FreeContext(self.ctx.as_ptr());
        }
    }
}
