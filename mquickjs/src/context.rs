use std::ffi::{c_char, c_void, CString};
use std::ptr::NonNull;

use mquickjs_sys::{
    js_stdlib, JSContext, JS_EVAL_RETVAL, JS_Eval, JS_EX_NORMAL, JS_FreeContext,
    JS_GetErrorStr, JS_NewContext, JS_TAG_EXCEPTION, JS_TAG_SPECIAL_BITS, JSValue,
};

use crate::error::JsError;
use crate::func::{register_callback, register_context, unregister_context};
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

        register_context(ctx);

        Ok(Self { ctx, _heap: heap })
    }

    /// Evaluate a script and return a raw value wrapper.
    pub fn eval(&self, script: &str, filename: &str) -> Result<Value<'_>, JsError> {
        let value = self.eval_raw(script, filename)?;
        Ok(Value::new(self.ctx, value))
    }

    /// Evaluate a script and convert the result to i32.
    pub fn eval_i32(&self, script: &str, filename: &str) -> Result<i32, JsError> {
        self.eval(script, filename)?.to_i32()
    }

    /// Evaluate a script and convert the result to bool.
    pub fn eval_bool(&self, script: &str, filename: &str) -> Result<bool, JsError> {
        self.eval(script, filename)?.to_bool()
    }

    /// Evaluate a script and convert the result to f64.
    pub fn eval_f64(&self, script: &str, filename: &str) -> Result<f64, JsError> {
        self.eval(script, filename)?.to_f64()
    }

    /// Evaluate a script and convert the result to String.
    pub fn eval_string(&self, script: &str, filename: &str) -> Result<String, JsError> {
        self.eval(script, filename)?.to_string()
    }

    /// Register a Rust callback callable from JavaScript.
    pub fn register_fn<F>(&self, name: &str, func: F) -> Result<(), JsError>
    where
        F: for<'ctx> Fn(&[Value<'ctx>]) -> Result<Value<'ctx>, JsError> + 'static,
    {
        let id = register_callback(self.ctx, Box::new(func));
        let name = escape_js_string(name);
        let script = format!(
            "globalThis['{name}'] = function() {{\n  var args = [{id}];\n  for (var i = 0; i < arguments.length; i++) {{\n    args.push(arguments[i]);\n  }}\n  return load.apply(null, args);\n}};"
        );
        self.eval_raw(&script, "<register_fn>")?;
        Ok(())
    }

    fn eval_raw(&self, script: &str, filename: &str) -> Result<JSValue, JsError> {
        let script = CString::new(script).map_err(|_| JsError::Runtime {
            message: "script contains null byte".to_string(),
        })?;
        let filename = CString::new(filename).map_err(|_| JsError::Runtime {
            message: "filename contains null byte".to_string(),
        })?;

        let value = unsafe {
            JS_Eval(
                self.ctx.as_ptr(),
                script.as_ptr() as *const c_char,
                script.as_bytes().len(),
                filename.as_ptr(),
                JS_EVAL_RETVAL as i32,
            )
        };

        if value == js_exception_value() {
            return Err(JsError::Runtime {
                message: error_message(self.ctx.as_ptr()),
            });
        }

        Ok(value)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unregister_context(self.ctx.as_ptr());
        unsafe {
            JS_FreeContext(self.ctx.as_ptr());
        }
    }
}

fn js_exception_value() -> JSValue {
    (JS_TAG_EXCEPTION as JSValue) | ((JS_EX_NORMAL as JSValue) << JS_TAG_SPECIAL_BITS)
}

fn error_message(ctx: *mut JSContext) -> String {
    let mut buf = [0u8; 256];
    unsafe {
        JS_GetErrorStr(ctx, buf.as_mut_ptr() as *mut c_char, buf.len());
    }
    let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

fn escape_js_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
