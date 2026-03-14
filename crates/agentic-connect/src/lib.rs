//! AgenticConnect — universal external interface engine for AI agents.
//!
//! Handles all external communication: HTTP, WebSocket, SSH, databases,
//! message queues, email, and every protocol that exists or will exist.

pub mod engine;
pub mod types;

// Re-export commonly used types at crate root
pub use engine::{
    ConnectionStore, CredentialVault, DbConnection, RetryEngine,
};
pub use types::{
    AuthMethod, CircuitBreaker, ConnectError, ConnectResult, Connection, ConnectionId,
    ConnectionProfile, ConnectionResult, FailureClass, HealthCheck, HealthStatus, Protocol,
    ProtocolCapabilities, RetryPolicy, RetryStrategy, Session, StoredCredential,
};
