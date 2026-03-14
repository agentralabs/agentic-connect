//! Connection soul types — Invention 3: Connection Soul.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::connection::ConnectionId;

/// Accumulated knowledge about a remote system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub connection_id: ConnectionId,
    pub fingerprint: SystemFingerprint,
    pub baseline: PerformanceBaseline,
    pub error_history: Vec<ErrorRecord>,
    pub capabilities: HashMap<String, String>,
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub connection_count: u64,
}

/// What we know about the remote system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemFingerprint {
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub server_software: Option<String>,
    pub server_version: Option<String>,
    pub tls_version: Option<String>,
    pub supported_protocols: Vec<String>,
    pub detected_services: Vec<String>,
    pub timezone: Option<String>,
}

/// Typical performance characteristics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub avg_throughput_bps: f64,
    pub error_rate: f64,
    pub sample_count: u64,
    pub last_measured: Option<DateTime<Utc>>,
}

/// A recorded error for pattern learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub timestamp: DateTime<Utc>,
    pub error_type: String,
    pub error_message: String,
    pub http_status: Option<u16>,
    pub resolved: bool,
}

impl ConnectionProfile {
    pub fn new(connection_id: ConnectionId) -> Self {
        let now = Utc::now();
        Self {
            connection_id,
            fingerprint: SystemFingerprint::default(),
            baseline: PerformanceBaseline::default(),
            error_history: Vec::new(),
            capabilities: HashMap::new(),
            first_seen: now,
            last_updated: now,
            connection_count: 0,
        }
    }

    pub fn record_latency(&mut self, latency_ms: f64) {
        let b = &mut self.baseline;
        let n = b.sample_count as f64;
        b.avg_latency_ms = (b.avg_latency_ms * n + latency_ms) / (n + 1.0);
        b.sample_count += 1;
        b.last_measured = Some(Utc::now());
        self.last_updated = Utc::now();
    }

    pub fn record_error(&mut self, error_type: &str, message: &str, status: Option<u16>) {
        self.error_history.push(ErrorRecord {
            timestamp: Utc::now(),
            error_type: error_type.to_string(),
            error_message: message.to_string(),
            http_status: status,
            resolved: false,
        });
        self.last_updated = Utc::now();
        // Keep last 100 errors
        if self.error_history.len() > 100 {
            self.error_history.drain(..self.error_history.len() - 100);
        }
    }
}
