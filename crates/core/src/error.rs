//! Error types for LLM Latency Lens core

/// Result type alias for core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Core error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Timing error occurred during measurement
    #[error("Timing error: {0}")]
    Timing(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a new timing error
    pub fn timing<S: Into<String>>(msg: S) -> Self {
        Error::Timing(msg.into())
    }

    /// Create a new configuration error
    pub fn invalid_config<S: Into<String>>(msg: S) -> Self {
        Error::InvalidConfig(msg.into())
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Error::Other(msg.into())
    }
}
