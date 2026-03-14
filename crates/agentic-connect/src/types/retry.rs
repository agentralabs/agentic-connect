//! Retry and circuit breaker types — Invention 4: Intelligent Retry Fabric.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Classification of a connection failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    /// Temporary network issue, DNS timeout, 503.
    Transient,
    /// 404, 400, invalid auth — won't succeed on retry.
    Permanent,
    /// 429 with Retry-After header.
    RateLimit,
    /// 401/403 — may succeed after token refresh.
    AuthFailure,
    /// Connection refused, DNS failure.
    NetworkError,
    /// Server closed connection unexpectedly.
    ServerError,
}

/// How to retry based on failure classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryStrategy {
    /// Exponential backoff (base_ms * 2^attempt).
    ExponentialBackoff {
        base_ms: u64,
        max_ms: u64,
        max_attempts: u32,
    },
    /// Wait a fixed duration then retry.
    FixedDelay { delay_ms: u64, max_attempts: u32 },
    /// Refresh auth then retry once.
    RefreshAndRetry,
    /// Don't retry — fail immediately.
    FailFast,
    /// Wait for the server-specified duration.
    WaitRetryAfter,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        RetryStrategy::ExponentialBackoff {
            base_ms: 1000,
            max_ms: 30_000,
            max_attempts: 3,
        }
    }
}

/// Per-connection retry policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub strategies: HashMap<String, RetryStrategy>,
    pub default_strategy: RetryStrategy,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        let mut strategies = HashMap::new();
        strategies.insert("transient".to_string(), RetryStrategy::default());
        strategies.insert(
            "rate_limit".to_string(),
            RetryStrategy::WaitRetryAfter,
        );
        strategies.insert(
            "auth_failure".to_string(),
            RetryStrategy::RefreshAndRetry,
        );
        strategies.insert(
            "permanent".to_string(),
            RetryStrategy::FailFast,
        );
        Self {
            strategies,
            default_strategy: RetryStrategy::default(),
        }
    }
}

/// Circuit breaker state for an endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub endpoint: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub failure_threshold: u32,
    pub last_failure: Option<DateTime<Utc>>,
    pub reset_after_secs: u64,
    pub half_open_at: Option<DateTime<Utc>>,
}

/// Circuit breaker FSM states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(endpoint: &str, threshold: u32, reset_secs: u64) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold: threshold,
            last_failure: None,
            reset_after_secs: reset_secs,
            half_open_at: None,
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Utc::now());
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            self.half_open_at = Some(
                Utc::now() + chrono::Duration::seconds(self.reset_after_secs as i64),
            );
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.half_open_at = None;
    }

    pub fn should_allow(&self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                self.half_open_at.map_or(false, |t| Utc::now() >= t)
            }
            CircuitState::HalfOpen => true,
        }
    }
}

/// Rate limit tracking for an endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitWindow {
    pub endpoint: String,
    pub limit: u32,
    pub remaining: u32,
    pub resets_at: DateTime<Utc>,
    pub window_secs: u64,
}
