use thiserror::Error;

/// Errors returned by the mquickjs safe wrapper.
#[derive(Debug, Error)]
pub enum JsError {
    /// Placeholder until implementation is completed.
    #[error("not yet implemented")]
    Unimplemented,
}
