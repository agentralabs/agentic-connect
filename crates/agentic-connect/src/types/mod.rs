//! Core types for AgenticConnect.

pub mod auth;
pub mod connection;
pub mod error;
pub mod health;
pub mod protocol;
pub mod retry;
pub mod soul;

pub use auth::{AuthMethod, OAuth2GrantType, StoredCredential};
pub use connection::{Connection, ConnectionId, ConnectionResult, Session};
pub use error::{ConnectError, ConnectResult};
pub use health::{HealthCheck, HealthStatus, Slo, TrendPoint};
pub use protocol::{Protocol, ProtocolCapabilities};
pub use retry::{CircuitBreaker, CircuitState, FailureClass, RateLimitWindow, RetryPolicy, RetryStrategy};
pub use soul::{ConnectionProfile, ErrorRecord, PerformanceBaseline, SystemFingerprint};
