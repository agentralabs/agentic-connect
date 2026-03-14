//! HTTP client with retry, auth, and connection soul integration.

use std::collections::HashMap;
use std::time::Instant;

use crate::types::{ConnectError, ConnectResult};

/// Result of an HTTP request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub latency_ms: u64,
    pub url: String,
    pub method: String,
}

/// Make an HTTP request with timing.
#[cfg(feature = "http")]
pub async fn http_request(
    url: &str,
    method: &str,
    headers: Option<&HashMap<String, String>>,
    body: Option<&str>,
    timeout_ms: u64,
) -> ConnectResult<HttpResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| ConnectError::Http(e.to_string()))?;

    let mut req = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        other => return Err(ConnectError::NotSupported(format!("HTTP method: {}", other))),
    };

    if let Some(hdrs) = headers {
        for (k, v) in hdrs {
            req = req.header(k.as_str(), v.as_str());
        }
    }

    if let Some(b) = body {
        req = req.body(b.to_string());
        // Auto-set content-type if not already set
        if headers.map_or(true, |h| !h.contains_key("content-type")) {
            req = req.header("content-type", "application/json");
        }
    }

    let start = Instant::now();
    let resp = req.send().await?;
    let latency_ms = start.elapsed().as_millis() as u64;

    let status = resp.status().as_u16();
    let resp_headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let resp_body = resp.text().await.unwrap_or_default();

    Ok(HttpResponse {
        status,
        headers: resp_headers,
        body: resp_body,
        latency_ms,
        url: url.to_string(),
        method: method.to_uppercase(),
    })
}

/// Extract rate limit info from response headers.
pub fn extract_rate_limit(headers: &HashMap<String, String>) -> Option<(u32, u32, i64)> {
    let limit = headers.get("x-ratelimit-limit")
        .or_else(|| headers.get("ratelimit-limit"))
        .and_then(|v| v.parse::<u32>().ok())?;
    let remaining = headers.get("x-ratelimit-remaining")
        .or_else(|| headers.get("ratelimit-remaining"))
        .and_then(|v| v.parse::<u32>().ok())?;
    let reset = headers.get("x-ratelimit-reset")
        .or_else(|| headers.get("ratelimit-reset"))
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    Some((limit, remaining, reset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rate_limit() {
        let mut headers = HashMap::new();
        headers.insert("x-ratelimit-limit".into(), "100".into());
        headers.insert("x-ratelimit-remaining".into(), "42".into());
        headers.insert("x-ratelimit-reset".into(), "1710000000".into());
        let (limit, remaining, reset) = extract_rate_limit(&headers).unwrap();
        assert_eq!(limit, 100);
        assert_eq!(remaining, 42);
        assert_eq!(reset, 1710000000);
    }

    #[test]
    fn test_extract_rate_limit_missing() {
        let headers = HashMap::new();
        assert!(extract_rate_limit(&headers).is_none());
    }
}
