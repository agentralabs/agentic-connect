//! Edge case tests — boundary conditions, error paths, malformed input.

use agentic_connect::*;
use agentic_connect::types::*;
use agentic_connect::engine::*;

// === Protocol edge cases ===

#[test]
fn test_protocol_from_empty_scheme() {
    assert_eq!(Protocol::from_scheme(""), None);
}

#[test]
fn test_protocol_from_scheme_case_insensitive() {
    assert_eq!(Protocol::from_scheme("HTTPS"), Some(Protocol::Https));
    assert_eq!(Protocol::from_scheme("Ssh"), Some(Protocol::Ssh));
    assert_eq!(Protocol::from_scheme("PostgreSQL"), Some(Protocol::Postgres));
}

#[test]
fn test_protocol_all_have_names() {
    let protos = [
        Protocol::Http, Protocol::Https, Protocol::WebSocket, Protocol::Wss,
        Protocol::Grpc, Protocol::Ssh, Protocol::Ftp, Protocol::Sftp,
        Protocol::Smtp, Protocol::Imap, Protocol::Dns, Protocol::Mqtt,
        Protocol::Amqp, Protocol::Redis, Protocol::Postgres, Protocol::Mysql,
        Protocol::Tcp, Protocol::Udp,
    ];
    for p in &protos {
        assert!(!p.name().is_empty());
        assert!(p.capabilities().request_response || p.capabilities().pub_sub || p.capabilities().streaming);
    }
}

// === Connection edge cases ===

#[test]
fn test_connection_from_url_no_path() {
    let conn = Connection::from_url("test", "https://example.com").unwrap();
    assert!(conn.path.is_none() || conn.path.as_deref() == Some("/"));
}

#[test]
fn test_connection_from_url_with_query() {
    let conn = Connection::from_url("test", "https://example.com/api?key=val").unwrap();
    assert_eq!(conn.host, "example.com");
}

#[test]
fn test_connection_from_url_ipv4() {
    let conn = Connection::from_url("local", "http://192.168.1.1:8080").unwrap();
    assert_eq!(conn.host, "192.168.1.1");
    assert_eq!(conn.port, Some(8080));
}

#[test]
fn test_connection_from_url_missing_scheme() {
    let result = Connection::from_url("bad", "example.com");
    assert!(result.is_err());
}

// === Store edge cases ===

#[test]
fn test_store_get_nonexistent() {
    let store = ConnectionStore::open_memory().unwrap();
    let id = uuid::Uuid::new_v4();
    assert!(store.get_connection(&id).unwrap().is_none());
}

#[test]
fn test_store_delete_nonexistent() {
    let store = ConnectionStore::open_memory().unwrap();
    let id = uuid::Uuid::new_v4();
    assert!(!store.delete_connection(&id).unwrap());
}

#[test]
fn test_store_save_same_id_twice() {
    let store = ConnectionStore::open_memory().unwrap();
    let conn = Connection::from_url("test", "https://example.com").unwrap();
    store.save_connection(&conn).unwrap();
    store.save_connection(&conn).unwrap(); // Should upsert, not fail
    let all = store.list_connections(None).unwrap();
    assert_eq!(all.len(), 1);
}

#[test]
fn test_store_empty_health_history() {
    let store = ConnectionStore::open_memory().unwrap();
    let id = uuid::Uuid::new_v4();
    let history = store.get_health_history(&id, 10).unwrap();
    assert!(history.is_empty());
}

// === Retry engine edge cases ===

#[test]
fn test_classify_boundary_status_codes() {
    assert_eq!(retry_engine::RetryEngine::classify_http_status(200), FailureClass::Transient);
    assert_eq!(retry_engine::RetryEngine::classify_http_status(301), FailureClass::Transient);
    assert_eq!(retry_engine::RetryEngine::classify_http_status(499), FailureClass::Transient);
    assert_eq!(retry_engine::RetryEngine::classify_http_status(500), FailureClass::ServerError);
    assert_eq!(retry_engine::RetryEngine::classify_http_status(599), FailureClass::ServerError);
}

#[test]
fn test_circuit_breaker_threshold_zero() {
    // Edge: threshold of 1 means first failure opens circuit
    let mut cb = CircuitBreaker::new("ep", 1, 60);
    cb.record_failure();
    assert!(!cb.should_allow());
}

#[test]
fn test_retry_engine_empty_endpoint() {
    let engine = RetryEngine::new();
    assert!(engine.should_allow(""));
    assert!(engine.failure_patterns("").is_empty());
}

// === Vault edge cases ===

#[test]
fn test_vault_retrieve_nonexistent() {
    let vault = vault::CredentialVault::new();
    assert!(vault.retrieve("nope").is_none());
}

#[test]
fn test_vault_delete_nonexistent() {
    let mut vault = vault::CredentialVault::new();
    assert!(!vault.delete("nope"));
}

#[test]
fn test_vault_encrypt_without_key() {
    let vault = vault::CredentialVault::new(); // No encryption key
    let result = vault.encrypt(b"test");
    assert!(result.is_err());
}

#[test]
fn test_vault_decrypt_too_short() {
    let vault = vault::CredentialVault::with_encryption("key").unwrap();
    let result = vault.decrypt(&[1, 2, 3]); // Less than 12 bytes (nonce)
    assert!(result.is_err());
}

#[test]
fn test_vault_decrypt_wrong_key() {
    let vault1 = vault::CredentialVault::with_encryption("key1").unwrap();
    let vault2 = vault::CredentialVault::with_encryption("key2").unwrap();
    let ct = vault1.encrypt(b"secret").unwrap();
    let result = vault2.decrypt(&ct);
    assert!(result.is_err());
}

// === DB engine edge cases ===

#[test]
fn test_db_invalid_sql() {
    let db = db_engine::DbConnection::open_sqlite(":memory:").unwrap();
    let result = db.query("SELECT * FROM nonexistent_table");
    assert!(result.is_err());
}

#[test]
fn test_db_unsupported_url() {
    let result = db_engine::DbConnection::from_url("postgres://localhost/mydb");
    match result {
        Err(e) => {
            let msg = format!("{}", e);
            assert!(msg.contains("SQLite") || msg.contains("supported"), "Error: {}", msg);
        }
        Ok(_) => panic!("Expected error for postgres URL"),
    }
}

#[test]
fn test_db_empty_table_schema() {
    let db = db_engine::DbConnection::open_sqlite(":memory:").unwrap();
    db.execute("CREATE TABLE empty (id INTEGER)").unwrap();
    let schema = db.discover_schema().unwrap();
    assert_eq!(schema.len(), 1);
    assert_eq!(schema[0].row_count, Some(0));
}

// === Webhook edge cases ===

#[test]
fn test_webhook_verify_empty_secret() {
    let sig = webhook::compute_hmac_sha256("", "data");
    assert!(webhook::verify_hmac_sha256("", "data", &sig));
}

#[test]
fn test_webhook_verify_empty_payload() {
    let sig = webhook::compute_hmac_sha256("secret", "");
    assert!(webhook::verify_hmac_sha256("secret", "", &sig));
}

#[test]
fn test_webhook_verify_garbage_signature() {
    assert!(!webhook::verify_hmac_sha256("secret", "data", "not-a-hex-string"));
}

// === Auth edge cases ===

#[test]
fn test_oauth2_expired() {
    let auth = AuthMethod::OAuth2 {
        client_id: "id".into(), client_secret: None,
        access_token: Some("tk".into()), refresh_token: Some("rt".into()),
        token_url: "url".into(), scopes: vec![],
        expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
        grant_type: OAuth2GrantType::ClientCredentials,
    };
    assert!(auth.is_expired());
    assert!(auth.needs_refresh());
}

#[test]
fn test_oauth2_no_expiry() {
    let auth = AuthMethod::OAuth2 {
        client_id: "id".into(), client_secret: None,
        access_token: None, refresh_token: None,
        token_url: "url".into(), scopes: vec![],
        expires_at: None,
        grant_type: OAuth2GrantType::AuthorizationCode,
    };
    assert!(!auth.is_expired());
}

// === Soul edge cases ===

#[test]
fn test_profile_latency_single_sample() {
    let mut p = ConnectionProfile::new(uuid::Uuid::new_v4());
    p.record_latency(42.0);
    assert_eq!(p.baseline.avg_latency_ms, 42.0);
    assert_eq!(p.baseline.sample_count, 1);
}

#[test]
fn test_profile_error_with_status() {
    let mut p = ConnectionProfile::new(uuid::Uuid::new_v4());
    p.record_error("http", "Not Found", Some(404));
    assert_eq!(p.error_history.len(), 1);
    assert_eq!(p.error_history[0].http_status, Some(404));
}
