use crate::{Context, JsError};

/// JavaScript runtime configuration for creating contexts.
#[derive(Debug, Clone, Copy)]
pub struct Runtime {
    memory_bytes: usize,
}

impl Runtime {
    /// Create a runtime with the default memory size (1 MiB).
    pub fn new() -> Result<Self, JsError> {
        Ok(Self {
            memory_bytes: 1024 * 1024,
        })
    }

    /// Create a runtime configured with a custom memory size.
    pub fn with_memory(memory_bytes: usize) -> Result<Self, JsError> {
        Ok(Self { memory_bytes })
    }

    /// Create a new `Context` using the runtime configuration.
    pub fn context(&self) -> Result<Context, JsError> {
        Context::new(self.memory_bytes)
    }
}
