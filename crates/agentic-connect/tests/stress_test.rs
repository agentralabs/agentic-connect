//! Stress tests — high volume, concurrent access, memory pressure.

use agentic_connect::*;
use agentic_connect::engine::*;

#[test]
fn test_store_1000_connections() {
    let store = ConnectionStore::open_memory().unwrap();
    for i in 0..1000 {
        let conn = Connection::from_url(
            &format!("conn-{}", i),
            &format!("https://api{}.example.com", i),
        ).unwrap();
        store.save_connection(&conn).unwrap();
    }
    let all = store.list_connections(None).unwrap();
    assert_eq!(all.len(), 1000);
    let stats = store.stats().unwrap();
    assert_eq!(stats.connection_count, 1000);
}

#[test]
fn test_store_1000_health_checks() {
    let store = ConnectionStore::open_memory().unwrap();
    let conn = Connection::from_url("test", "https://example.com").unwrap();
    store.save_connection(&conn).unwrap();

    for i in 0..1000 {
        let check = HealthCheck {
            connection_id: conn.id,
            protocol: Protocol::Https,
            host: "example.com".into(),
            port: 443,
            status: if i % 10 == 0 { HealthStatus::Degraded } else { HealthStatus::Healthy },
            latency_ms: Some(50.0 + (i as f64) * 0.1),
            message: None,
            checked_at: chrono::Utc::now(),
        };
        store.save_health_check(&check).unwrap();
    }

    let history = store.get_health_history(&conn.id, 50).unwrap();
    assert_eq!(history.len(), 50);
}

#[test]
fn test_retry_engine_10000_failures() {
    let mut engine = RetryEngine::new();
    for i in 0..10_000 {
        let ep = format!("https://api{}.example.com", i % 100);
        let class = match i % 5 {
            0 => FailureClass::Transient,
            1 => FailureClass::RateLimit,
            2 => FailureClass::AuthFailure,
            3 => FailureClass::Permanent,
            _ => FailureClass::ServerError,
        };
        engine.record_failure(&ep, class, "test error", Some((i % 600) as u16));
    }
    // History capped at 500
    assert!(engine.recent_failures(1000).len() <= 500);
    // Circuit breakers should be open for most endpoints
    let circuits = engine.all_circuits();
    assert!(circuits.len() <= 100); // 100 unique endpoints
}

#[test]
fn test_circuit_breaker_rapid_open_close() {
    let mut cb = CircuitBreaker::new("ep", 3, 1);
    for _ in 0..100 {
        // Open
        for _ in 0..3 { cb.record_failure(); }
        assert!(!cb.should_allow());
        // Close
        cb.record_success();
        assert!(cb.should_allow());
    }
    assert_eq!(cb.failure_count, 0);
}

#[test]
fn test_vault_100_credentials() {
    let mut vault = vault::CredentialVault::new();
    for i in 0..100 {
        vault.store(StoredCredential {
            id: uuid::Uuid::new_v4(),
            name: format!("cred-{}", i),
            auth: AuthMethod::Bearer { token: format!("token-{}", i) },
            created_at: chrono::Utc::now(),
            last_rotated: None,
            tags: vec![format!("env-{}", i % 3)],
        });
    }
    assert_eq!(vault.count(), 100);
    let list = vault.list();
    assert_eq!(list.len(), 100);
    // Verify no tokens leaked in summaries
    for summary in &list {
        let json = serde_json::to_string(summary).unwrap();
        assert!(!json.contains("token-"));
    }
}

#[test]
fn test_vault_encrypt_decrypt_large_payload() {
    let vault = vault::CredentialVault::with_encryption("stress-test-key").unwrap();
    let large_data = "x".repeat(100_000); // 100KB
    let encrypted = vault.encrypt(large_data.as_bytes()).unwrap();
    let decrypted = vault.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, large_data.as_bytes());
}

#[test]
fn test_db_large_resultset() {
    let db = db_engine::DbConnection::open_sqlite(":memory:").unwrap();
    db.execute("CREATE TABLE big (id INTEGER PRIMARY KEY, val TEXT)").unwrap();
    for i in 0..1000 {
        db.execute(&format!("INSERT INTO big VALUES ({}, 'value-{}')", i, i)).unwrap();
    }
    let result = db.query("SELECT * FROM big").unwrap();
    assert_eq!(result.row_count, 1000);
}

#[test]
fn test_db_schema_many_tables() {
    let db = db_engine::DbConnection::open_sqlite(":memory:").unwrap();
    for i in 0..50 {
        db.execute(&format!("CREATE TABLE t{} (id INTEGER PRIMARY KEY, name TEXT, value REAL)", i)).unwrap();
    }
    let schema = db.discover_schema().unwrap();
    assert_eq!(schema.len(), 50);
    for table in &schema {
        assert_eq!(table.columns.len(), 3);
    }
}

#[test]
fn test_profile_10000_latency_samples() {
    let mut profile = types::soul::ConnectionProfile::new(uuid::Uuid::new_v4());
    for i in 0..10_000 {
        profile.record_latency(100.0 + (i as f64) * 0.01);
    }
    assert_eq!(profile.baseline.sample_count, 10_000);
    // Average should be roughly in the middle
    assert!(profile.baseline.avg_latency_ms > 100.0);
    assert!(profile.baseline.avg_latency_ms < 200.0);
}

#[test]
fn test_webhook_hmac_many_payloads() {
    let secret = "stress-test-secret-key";
    for i in 0..1000 {
        let payload = format!(r#"{{"event":"test","seq":{}}}"#, i);
        let sig = webhook::compute_hmac_sha256(secret, &payload);
        assert!(webhook::verify_hmac_sha256(secret, &payload, &sig));
        assert!(!webhook::verify_hmac_sha256("wrong", &payload, &sig));
    }
}

#[test]
fn test_protocol_detect_all_ports() {
    let known_ports = [22, 80, 443, 21, 25, 587, 993, 53, 1883, 5672, 6379, 5432, 3306];
    for port in known_ports {
        let proto = protocol_detect::probe_host("127.0.0.1", port, 1);
        // We're not actually connecting, just testing the function doesn't panic
        let _ = proto;
    }
}
