/// Errors returned by the mquickjs safe wrapper.
#[derive(Debug)]
pub enum JsError {
    /// Failures creating or initializing the JS context.
    ContextInit { message: String },
    /// Errors unrelated to JS exceptions (e.g. invalid inputs).
    Runtime { message: String },
    /// JavaScript execution errors.
    Exception {
        message: String,
        stack: Option<String>,
    },
    /// Value conversion failures.
    Conversion { message: String },
    /// Errors raised by registered Rust callbacks.
    Callback { message: String },
}

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsError::ContextInit { message } => {
                write!(f, "context init failed: {message}")
            }
            JsError::Runtime { message } => {
                write!(f, "runtime error: {message}")
            }
            JsError::Exception { message, stack } => {
                write!(f, "runtime error: {message}")?;
                if let Some(stack) = stack {
                    write!(f, "\n{stack}")?;
                }
                Ok(())
            }
            JsError::Conversion { message } => {
                write!(f, "conversion error: {message}")
            }
            JsError::Callback { message } => {
                write!(f, "callback error: {message}")
            }
        }
    }
}

impl std::error::Error for JsError {}
