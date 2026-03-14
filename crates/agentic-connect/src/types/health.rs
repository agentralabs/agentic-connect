//! Health check types — Invention 20: Network Sentinel.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::connection::ConnectionId;
use super::protocol::Protocol;

/// Result of a health probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub connection_id: ConnectionId,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub status: HealthStatus,
    pub latency_ms: Option<f64>,
    pub message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

/// Health status classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Service level objective for monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slo {
    pub name: String,
    pub connection_id: ConnectionId,
    pub max_latency_ms: Option<f64>,
    pub min_availability_pct: Option<f64>,
    pub max_error_rate_pct: Option<f64>,
}

/// Trend data point for monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub timestamp: DateTime<Utc>,
    pub latency_ms: f64,
    pub success: bool,
    pub error_type: Option<String>,
}
