use thiserror::Error;

/// Errors returned by the mquickjs safe wrapper.
#[derive(Debug, Error)]
pub enum JsError {
    /// Failures creating or initializing the JS context.
    #[error("context init failed: {message}")]
    ContextInit { message: String },
    /// JavaScript execution errors.
    #[error("runtime error: {message}")]
    Runtime { message: String },
    /// Value conversion failures.
    #[error("conversion error: {message}")]
    Conversion { message: String },
    /// Errors raised by registered Rust callbacks.
    #[error("callback error: {message}")]
    Callback { message: String },
}
