//! Webhook engine — send, receive, verify signatures.

use ring::hmac;
use std::collections::HashMap;

use crate::types::{ConnectError, ConnectResult};

/// Webhook delivery record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookDelivery {
    pub id: uuid::Uuid,
    pub url: String,
    pub payload: serde_json::Value,
    pub status: u16,
    pub latency_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
}

/// Registered webhook endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookEndpoint {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub delivery_count: u64,
}

/// Compute HMAC-SHA256 signature for webhook payload.
pub fn compute_hmac_sha256(secret: &str, payload: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let tag = hmac::sign(&key, payload.as_bytes());
    hex_encode(tag.as_ref())
}

/// Verify HMAC-SHA256 signature.
pub fn verify_hmac_sha256(secret: &str, payload: &str, signature: &str) -> bool {
    let expected = compute_hmac_sha256(secret, payload);
    // Constant-time comparison via ring
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let sig_bytes = hex_decode(signature);
    match sig_bytes {
        Some(bytes) => hmac::verify(&key, payload.as_bytes(), &bytes).is_ok(),
        None => {
            // Try comparing hex strings as fallback
            expected == signature.trim_start_matches("sha256=")
        }
    }
}

/// Send a webhook with optional signature.
#[cfg(feature = "http")]
pub async fn send_webhook(
    url: &str,
    payload: &serde_json::Value,
    secret: Option<&str>,
) -> ConnectResult<WebhookDelivery> {
    let body = serde_json::to_string(payload)
        .map_err(|e| ConnectError::Serialization(e.to_string()))?;

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    if let Some(s) = secret {
        let sig = compute_hmac_sha256(s, &body);
        headers.insert("x-webhook-signature".to_string(), format!("sha256={}", sig));
    }

    let resp = crate::engine::http_client::http_request(
        url, "POST", Some(&headers), Some(&body), 30_000,
    ).await?;

    Ok(WebhookDelivery {
        id: uuid::Uuid::new_v4(),
        url: url.to_string(),
        payload: payload.clone(),
        status: resp.status,
        latency_ms: resp.latency_ms,
        timestamp: chrono::Utc::now(),
        success: resp.status < 400,
    })
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.trim_start_matches("sha256=");
    if hex.len() % 2 != 0 { return None; }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sha256_sign_verify() {
        let secret = "my-webhook-secret";
        let payload = r#"{"event":"push","ref":"refs/heads/main"}"#;
        let sig = compute_hmac_sha256(secret, payload);
        assert!(!sig.is_empty());
        assert!(verify_hmac_sha256(secret, payload, &sig));
    }

    #[test]
    fn test_hmac_wrong_secret() {
        let payload = r#"{"test": true}"#;
        let sig = compute_hmac_sha256("correct-secret", payload);
        assert!(!verify_hmac_sha256("wrong-secret", payload, &sig));
    }

    #[test]
    fn test_hmac_with_sha256_prefix() {
        let secret = "sec";
        let payload = "data";
        let sig = compute_hmac_sha256(secret, payload);
        let prefixed = format!("sha256={}", sig);
        assert!(verify_hmac_sha256(secret, payload, &prefixed));
    }

    #[test]
    fn test_hex_roundtrip() {
        let bytes = b"hello";
        let hex = hex_encode(bytes);
        let decoded = hex_decode(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }
}
