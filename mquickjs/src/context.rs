use crate::error::JsError;
use crate::value::Value;

/// JavaScript execution context owning the underlying mquickjs state.
#[derive(Debug)]
pub struct Context {
    _private: (),
}

impl Context {
    /// Create a new JavaScript context with the given memory buffer size.
    pub fn new(_memory_bytes: usize) -> Result<Self, JsError> {
        Err(JsError::Unimplemented)
    }

    /// Evaluate a script and return a raw value wrapper.
    pub fn eval(&self, _script: &str, _filename: &str) -> Result<Value, JsError> {
        Err(JsError::Unimplemented)
    }
}
