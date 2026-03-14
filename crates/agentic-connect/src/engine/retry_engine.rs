//! Retry engine — manages circuit breakers, rate limits, and failure classification.

use std::collections::HashMap;
use chrono::Utc;

use crate::types::{CircuitBreaker, CircuitState, FailureClass, RateLimitWindow, RetryStrategy};

/// Runtime retry state for all connections.
pub struct RetryEngine {
    circuit_breakers: HashMap<String, CircuitBreaker>,
    rate_limits: HashMap<String, RateLimitWindow>,
    failure_history: Vec<FailureRecord>,
}

/// Recorded failure for pattern learning.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FailureRecord {
    pub endpoint: String,
    pub failure_class: FailureClass,
    pub timestamp: chrono::DateTime<Utc>,
    pub http_status: Option<u16>,
    pub message: String,
}

impl RetryEngine {
    pub fn new() -> Self {
        Self {
            circuit_breakers: HashMap::new(),
            rate_limits: HashMap::new(),
            failure_history: Vec::new(),
        }
    }

    /// Classify an HTTP status code into a failure class.
    pub fn classify_http_status(status: u16) -> FailureClass {
        match status {
            429 => FailureClass::RateLimit,
            401 | 403 => FailureClass::AuthFailure,
            404 | 400 | 405 | 410 | 422 => FailureClass::Permanent,
            500..=599 => FailureClass::ServerError,
            _ => FailureClass::Transient,
        }
    }

    /// Classify a connection error string.
    pub fn classify_error(error: &str) -> FailureClass {
        let lower = error.to_lowercase();
        if lower.contains("timeout") || lower.contains("timed out") {
            FailureClass::Transient
        } else if lower.contains("refused") || lower.contains("dns") || lower.contains("resolve") {
            FailureClass::NetworkError
        } else if lower.contains("unauthorized") || lower.contains("forbidden") || lower.contains("auth") {
            FailureClass::AuthFailure
        } else if lower.contains("rate") || lower.contains("throttl") || lower.contains("429") {
            FailureClass::RateLimit
        } else if lower.contains("not found") || lower.contains("404") {
            FailureClass::Permanent
        } else {
            FailureClass::Transient
        }
    }

    /// Get the retry strategy for a failure class.
    pub fn strategy_for(class: FailureClass) -> RetryStrategy {
        match class {
            FailureClass::Transient | FailureClass::ServerError => RetryStrategy::ExponentialBackoff {
                base_ms: 1000,
                max_ms: 30_000,
                max_attempts: 3,
            },
            FailureClass::RateLimit => RetryStrategy::WaitRetryAfter,
            FailureClass::AuthFailure => RetryStrategy::RefreshAndRetry,
            FailureClass::Permanent => RetryStrategy::FailFast,
            FailureClass::NetworkError => RetryStrategy::ExponentialBackoff {
                base_ms: 2000,
                max_ms: 60_000,
                max_attempts: 5,
            },
        }
    }

    /// Get or create a circuit breaker for an endpoint.
    pub fn get_circuit(&mut self, endpoint: &str) -> &CircuitBreaker {
        self.circuit_breakers
            .entry(endpoint.to_string())
            .or_insert_with(|| CircuitBreaker::new(endpoint, 5, 60))
    }

    /// Record a failure for an endpoint.
    pub fn record_failure(&mut self, endpoint: &str, class: FailureClass, message: &str, status: Option<u16>) {
        // Update circuit breaker
        let cb = self.circuit_breakers
            .entry(endpoint.to_string())
            .or_insert_with(|| CircuitBreaker::new(endpoint, 5, 60));
        cb.record_failure();

        // Record for pattern learning
        self.failure_history.push(FailureRecord {
            endpoint: endpoint.to_string(),
            failure_class: class,
            timestamp: Utc::now(),
            http_status: status,
            message: message.to_string(),
        });

        // Keep last 500 failures
        if self.failure_history.len() > 500 {
            self.failure_history.drain(..self.failure_history.len() - 500);
        }
    }

    /// Record a success for an endpoint.
    pub fn record_success(&mut self, endpoint: &str) {
        if let Some(cb) = self.circuit_breakers.get_mut(endpoint) {
            cb.record_success();
        }
    }

    /// Check if a request to this endpoint should be allowed.
    pub fn should_allow(&self, endpoint: &str) -> bool {
        match self.circuit_breakers.get(endpoint) {
            Some(cb) => cb.should_allow(),
            None => true,
        }
    }

    /// Update rate limit tracking from response headers.
    pub fn update_rate_limit(&mut self, endpoint: &str, limit: u32, remaining: u32, reset_epoch: i64) {
        let resets_at = chrono::DateTime::from_timestamp(reset_epoch, 0)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        self.rate_limits.insert(endpoint.to_string(), RateLimitWindow {
            endpoint: endpoint.to_string(),
            limit,
            remaining,
            resets_at,
            window_secs: 60,
        });
    }

    /// Get rate limit status for an endpoint.
    pub fn rate_limit_status(&self, endpoint: &str) -> Option<&RateLimitWindow> {
        self.rate_limits.get(endpoint)
    }

    /// Get all circuit breaker states.
    pub fn all_circuits(&self) -> &HashMap<String, CircuitBreaker> {
        &self.circuit_breakers
    }

    /// Reset a circuit breaker.
    pub fn reset_circuit(&mut self, endpoint: &str) -> bool {
        if let Some(cb) = self.circuit_breakers.get_mut(endpoint) {
            cb.record_success();
            true
        } else {
            false
        }
    }

    /// Get failure patterns for an endpoint.
    pub fn failure_patterns(&self, endpoint: &str) -> Vec<&FailureRecord> {
        self.failure_history.iter().filter(|f| f.endpoint == endpoint).collect()
    }

    /// Get all recent failures.
    pub fn recent_failures(&self, limit: usize) -> Vec<&FailureRecord> {
        self.failure_history.iter().rev().take(limit).collect()
    }
}

impl Default for RetryEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_http_status() {
        assert_eq!(RetryEngine::classify_http_status(429), FailureClass::RateLimit);
        assert_eq!(RetryEngine::classify_http_status(401), FailureClass::AuthFailure);
        assert_eq!(RetryEngine::classify_http_status(404), FailureClass::Permanent);
        assert_eq!(RetryEngine::classify_http_status(503), FailureClass::ServerError);
    }

    #[test]
    fn test_circuit_breaker_opens() {
        let mut engine = RetryEngine::new();
        let ep = "https://api.example.com";
        for _ in 0..5 {
            engine.record_failure(ep, FailureClass::Transient, "timeout", None);
        }
        assert!(!engine.should_allow(ep));
    }

    #[test]
    fn test_circuit_breaker_resets() {
        let mut engine = RetryEngine::new();
        let ep = "https://api.example.com";
        for _ in 0..5 {
            engine.record_failure(ep, FailureClass::Transient, "timeout", None);
        }
        assert!(!engine.should_allow(ep));
        engine.reset_circuit(ep);
        assert!(engine.should_allow(ep));
    }

    #[test]
    fn test_classify_error_string() {
        assert_eq!(RetryEngine::classify_error("connection timed out"), FailureClass::Transient);
        assert_eq!(RetryEngine::classify_error("connection refused"), FailureClass::NetworkError);
        assert_eq!(RetryEngine::classify_error("rate limit exceeded"), FailureClass::RateLimit);
        assert_eq!(RetryEngine::classify_error("unauthorized"), FailureClass::AuthFailure);
    }

    #[test]
    fn test_failure_history_capped() {
        let mut engine = RetryEngine::new();
        for i in 0..600 {
            engine.record_failure(&format!("ep{}", i), FailureClass::Transient, "err", None);
        }
        assert!(engine.failure_history.len() <= 500);
    }
}
