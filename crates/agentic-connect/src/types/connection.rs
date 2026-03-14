//! Connection types — core data model for all connections.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::auth::AuthMethod;
use super::protocol::Protocol;

/// Unique identifier for a connection.
pub type ConnectionId = Uuid;

/// A configured connection to an external system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: ConnectionId,
    pub name: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub auth: Option<AuthMethod>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl Connection {
    /// Create a new connection with auto-detected protocol from URL.
    pub fn from_url(name: &str, url_str: &str) -> Result<Self, String> {
        let parsed = url::Url::parse(url_str).map_err(|e| e.to_string())?;
        let protocol = Protocol::from_scheme(parsed.scheme())
            .ok_or_else(|| format!("Unknown scheme: {}", parsed.scheme()))?;
        let host = parsed
            .host_str()
            .ok_or("No host in URL")?
            .to_string();
        let port = parsed.port().or_else(|| protocol.default_port());

        Ok(Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            protocol,
            host,
            port,
            path: Some(parsed.path().to_string()).filter(|p| p != "/"),
            auth: None,
            tags: Vec::new(),
            created_at: Utc::now(),
            last_used: None,
            metadata: serde_json::Value::Null,
        })
    }

    /// Full URL representation of this connection.
    pub fn url(&self) -> String {
        let scheme = format!("{:?}", self.protocol).to_lowercase();
        let port_str = self
            .port
            .map(|p| format!(":{}", p))
            .unwrap_or_default();
        let path_str = self.path.as_deref().unwrap_or("");
        format!("{}://{}{}{}", scheme, self.host, port_str, path_str)
    }
}

/// Result of a connection attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub connection_id: ConnectionId,
    pub success: bool,
    pub latency_ms: u64,
    pub protocol_version: Option<String>,
    pub server_info: Option<String>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// A session with a remote system (stateful connection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub connection_id: ConnectionId,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub request_count: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub active: bool,
}

impl Session {
    pub fn new(connection_id: ConnectionId) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            connection_id,
            started_at: now,
            last_activity: now,
            request_count: 0,
            bytes_sent: 0,
            bytes_received: 0,
            active: true,
        }
    }
}
