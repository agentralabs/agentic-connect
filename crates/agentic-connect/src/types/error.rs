//! Error types for AgenticConnect.

use thiserror::Error;

/// Core error type for all AgenticConnect operations.
#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Circuit breaker open for {endpoint}")]
    CircuitOpen { endpoint: String },

    #[error("Credential not found: {0}")]
    CredentialNotFound(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type alias for AgenticConnect operations.
pub type ConnectResult<T> = Result<T, ConnectError>;

impl From<serde_json::Error> for ConnectError {
    fn from(e: serde_json::Error) -> Self {
        ConnectError::Serialization(e.to_string())
    }
}

impl From<rusqlite::Error> for ConnectError {
    fn from(e: rusqlite::Error) -> Self {
        ConnectError::Database(e.to_string())
    }
}

#[cfg(feature = "http")]
impl From<reqwest::Error> for ConnectError {
    fn from(e: reqwest::Error) -> Self {
        ConnectError::Http(e.to_string())
    }
}
