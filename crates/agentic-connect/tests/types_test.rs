//! Core type tests — protocol, connection, auth, retry, soul.

use agentic_connect::*;
use agentic_connect::types::*;

// Protocol tests
#[test]
fn test_protocol_from_scheme() {
    assert_eq!(Protocol::from_scheme("https"), Some(Protocol::Https));
    assert_eq!(Protocol::from_scheme("ssh"), Some(Protocol::Ssh));
    assert_eq!(Protocol::from_scheme("postgres"), Some(Protocol::Postgres));
    assert_eq!(Protocol::from_scheme("postgresql"), Some(Protocol::Postgres));
    assert_eq!(Protocol::from_scheme("unknown"), None);
}

#[test]
fn test_protocol_default_ports() {
    assert_eq!(Protocol::Http.default_port(), Some(80));
    assert_eq!(Protocol::Https.default_port(), Some(443));
    assert_eq!(Protocol::Ssh.default_port(), Some(22));
    assert_eq!(Protocol::Postgres.default_port(), Some(5432));
    assert_eq!(Protocol::Tcp.default_port(), None);
}

#[test]
fn test_protocol_supports_tls() {
    assert!(Protocol::Https.supports_tls());
    assert!(Protocol::Wss.supports_tls());
    assert!(!Protocol::Http.supports_tls());
    assert!(!Protocol::Tcp.supports_tls());
}

#[test]
fn test_protocol_capabilities() {
    let caps = Protocol::WebSocket.capabilities();
    assert!(caps.bidirectional);
    assert!(caps.streaming);
    assert!(caps.pub_sub);

    let http_caps = Protocol::Http.capabilities();
    assert!(http_caps.request_response);
    assert!(!http_caps.streaming);
    assert!(!http_caps.bidirectional);
}

#[test]
fn test_protocol_name() {
    assert_eq!(Protocol::Grpc.name(), "gRPC");
    assert_eq!(Protocol::Postgres.name(), "PostgreSQL");
}

// Connection tests
#[test]
fn test_connection_from_url() {
    let conn = Connection::from_url("github", "https://api.github.com/v1").unwrap();
    assert_eq!(conn.name, "github");
    assert_eq!(conn.protocol, Protocol::Https);
    assert_eq!(conn.host, "api.github.com");
    assert_eq!(conn.port, Some(443));
    assert_eq!(conn.path, Some("/v1".into()));
}

#[test]
fn test_connection_from_url_with_port() {
    let conn = Connection::from_url("local", "http://localhost:8080/api").unwrap();
    assert_eq!(conn.port, Some(8080));
    assert_eq!(conn.host, "localhost");
}

#[test]
fn test_connection_from_invalid_url() {
    let result = Connection::from_url("bad", "not a url");
    assert!(result.is_err());
}

#[test]
fn test_connection_url_roundtrip() {
    let conn = Connection::from_url("test", "https://example.com").unwrap();
    let url = conn.url();
    assert!(url.contains("example.com"));
    assert!(url.starts_with("https://"));
}

#[test]
fn test_session_creation() {
    let conn = Connection::from_url("test", "https://example.com").unwrap();
    let session = Session::new(conn.id);
    assert!(session.active);
    assert_eq!(session.request_count, 0);
    assert_eq!(session.connection_id, conn.id);
}

// Auth tests
#[test]
fn test_auth_method_names() {
    assert_eq!(AuthMethod::None.method_name(), "none");
    assert_eq!(AuthMethod::Bearer { token: "x".into() }.method_name(), "bearer");
    assert_eq!(AuthMethod::Basic { username: "u".into(), password: "p".into() }.method_name(), "basic");
}

#[test]
fn test_oauth2_not_expired() {
    let auth = AuthMethod::OAuth2 {
        client_id: "id".into(), client_secret: None,
        access_token: Some("tk".into()), refresh_token: None,
        token_url: "https://auth.example.com/token".into(),
        scopes: vec![], expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        grant_type: OAuth2GrantType::ClientCredentials,
    };
    assert!(!auth.is_expired());
    assert!(!auth.needs_refresh()); // no refresh_token
}

#[test]
fn test_auth_has_expiry() {
    assert!(AuthMethod::OAuth2 {
        client_id: "".into(), client_secret: None, access_token: None,
        refresh_token: None, token_url: "".into(), scopes: vec![],
        expires_at: None, grant_type: OAuth2GrantType::ClientCredentials,
    }.has_expiry());
    assert!(!AuthMethod::Bearer { token: "x".into() }.has_expiry());
}

// Retry tests
#[test]
fn test_retry_strategy_default() {
    let policy = RetryPolicy::default();
    assert!(policy.strategies.contains_key("transient"));
    assert!(policy.strategies.contains_key("rate_limit"));
    assert!(policy.strategies.contains_key("permanent"));
}

#[test]
fn test_circuit_breaker_new() {
    let cb = CircuitBreaker::new("ep", 5, 60);
    assert_eq!(cb.state, CircuitState::Closed);
    assert_eq!(cb.failure_count, 0);
    assert!(cb.should_allow());
}

#[test]
fn test_circuit_breaker_opens_at_threshold() {
    let mut cb = CircuitBreaker::new("ep", 3, 60);
    cb.record_failure();
    cb.record_failure();
    assert!(cb.should_allow()); // 2 < 3
    cb.record_failure();
    assert!(!cb.should_allow()); // 3 >= 3, now open
    assert_eq!(cb.state, CircuitState::Open);
}

#[test]
fn test_circuit_breaker_success_resets() {
    let mut cb = CircuitBreaker::new("ep", 2, 60);
    cb.record_failure();
    cb.record_failure();
    assert!(!cb.should_allow());
    cb.record_success();
    assert!(cb.should_allow());
    assert_eq!(cb.failure_count, 0);
}

// Soul tests
#[test]
fn test_connection_profile_latency_tracking() {
    let mut profile = ConnectionProfile::new(uuid::Uuid::new_v4());
    profile.record_latency(100.0);
    profile.record_latency(200.0);
    assert_eq!(profile.baseline.sample_count, 2);
    assert!((profile.baseline.avg_latency_ms - 150.0).abs() < 0.01);
}

#[test]
fn test_connection_profile_error_cap() {
    let mut profile = ConnectionProfile::new(uuid::Uuid::new_v4());
    for i in 0..150 {
        profile.record_error("timeout", &format!("err {}", i), None);
    }
    assert!(profile.error_history.len() <= 100);
}

// Serialization roundtrips
#[test]
fn test_protocol_serde_roundtrip() {
    let proto = Protocol::Postgres;
    let json = serde_json::to_string(&proto).unwrap();
    let back: Protocol = serde_json::from_str(&json).unwrap();
    assert_eq!(back, proto);
}

#[test]
fn test_auth_method_serde_roundtrip() {
    let auth = AuthMethod::ApiKey {
        key: "sk-123".into(),
        header_name: Some("X-Api-Key".into()),
        query_param: None,
    };
    let json = serde_json::to_string(&auth).unwrap();
    let back: AuthMethod = serde_json::from_str(&json).unwrap();
    assert_eq!(back.method_name(), "api_key");
}
