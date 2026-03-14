//! Phase 3: Session management tests — lifecycle, state, multi-connection.

use agentic_connect_mcp::SessionManager;
use agentic_connect::types::*;

#[test]
fn test_session_in_memory() {
    let sess = SessionManager::in_memory().unwrap();
    let stats = sess.store().stats().unwrap();
    assert_eq!(stats.connection_count, 0);
}

#[test]
fn test_session_store_and_retrieve() {
    let sess = SessionManager::in_memory().unwrap();
    let conn = Connection::from_url("test", "https://example.com").unwrap();
    sess.store().save_connection(&conn).unwrap();
    let loaded = sess.store().get_connection(&conn.id).unwrap();
    assert!(loaded.is_some());
}

#[test]
fn test_session_retry_engine() {
    let mut sess = SessionManager::in_memory().unwrap();
    assert!(sess.retry().should_allow("https://api.example.com"));
    for _ in 0..5 {
        sess.retry_mut().record_failure("https://api.example.com", FailureClass::Transient, "timeout", None);
    }
    assert!(!sess.retry().should_allow("https://api.example.com"));
}

#[test]
fn test_session_vault() {
    let mut sess = SessionManager::in_memory().unwrap();
    assert_eq!(sess.vault().count(), 0);
    sess.vault_mut().store(StoredCredential {
        id: uuid::Uuid::new_v4(),
        name: "test-cred".into(),
        auth: AuthMethod::Bearer { token: "tk-123".into() },
        created_at: chrono::Utc::now(),
        last_rotated: None,
        tags: vec![],
    });
    assert_eq!(sess.vault().count(), 1);
}

#[test]
fn test_session_db_lifecycle() {
    let mut sess = SessionManager::in_memory().unwrap();
    assert!(sess.get_db("test").is_none());
    sess.open_db("test", "sqlite://:memory:").unwrap();
    assert!(sess.get_db("test").is_some());
    assert_eq!(sess.list_dbs().len(), 1);
}

#[test]
fn test_session_multiple_dbs() {
    let mut sess = SessionManager::in_memory().unwrap();
    sess.open_db("db1", "sqlite://:memory:").unwrap();
    sess.open_db("db2", "sqlite://:memory:").unwrap();
    sess.open_db("db3", "sqlite://:memory:").unwrap();
    assert_eq!(sess.list_dbs().len(), 3);
}

#[test]
fn test_session_db_invalid_url() {
    let mut sess = SessionManager::in_memory().unwrap();
    let result = sess.open_db("bad", "postgres://localhost/db");
    assert!(result.is_err());
}

#[test]
fn test_session_connection_with_auth() {
    let sess = SessionManager::in_memory().unwrap();
    let mut conn = Connection::from_url("api", "https://api.stripe.com").unwrap();
    conn.auth = Some(AuthMethod::Bearer { token: "sk_test_123".into() });
    conn.tags = vec!["payments".into(), "prod".into()];
    sess.store().save_connection(&conn).unwrap();

    let loaded = sess.store().get_connection(&conn.id).unwrap().unwrap();
    assert_eq!(loaded.auth.as_ref().unwrap().method_name(), "bearer");
    assert_eq!(loaded.tags.len(), 2);
}

#[test]
fn test_session_connection_profile_roundtrip() {
    let sess = SessionManager::in_memory().unwrap();
    let conn = Connection::from_url("test", "https://api.example.com").unwrap();
    sess.store().save_connection(&conn).unwrap();

    let mut profile = ConnectionProfile::new(conn.id);
    profile.record_latency(100.0);
    profile.record_latency(200.0);
    profile.record_latency(150.0);
    profile.fingerprint.os = Some("Linux".into());
    profile.fingerprint.server_software = Some("nginx".into());
    sess.store().save_profile(&profile).unwrap();

    let loaded = sess.store().get_profile(&conn.id).unwrap().unwrap();
    assert_eq!(loaded.baseline.sample_count, 3);
    assert_eq!(loaded.fingerprint.os, Some("Linux".into()));
}

#[test]
fn test_session_health_check_storage() {
    let sess = SessionManager::in_memory().unwrap();
    let conn = Connection::from_url("test", "https://api.example.com").unwrap();
    sess.store().save_connection(&conn).unwrap();

    for i in 0..10 {
        let check = HealthCheck {
            connection_id: conn.id,
            protocol: Protocol::Https,
            host: "api.example.com".into(),
            port: 443,
            status: HealthStatus::Healthy,
            latency_ms: Some(50.0 + i as f64),
            message: None,
            checked_at: chrono::Utc::now(),
        };
        sess.store().save_health_check(&check).unwrap();
    }

    let history = sess.store().get_health_history(&conn.id, 5).unwrap();
    assert_eq!(history.len(), 5);
}

#[test]
fn test_session_list_connections_with_tag_filter() {
    let sess = SessionManager::in_memory().unwrap();
    for i in 0..5 {
        let mut conn = Connection::from_url(&format!("c{}", i), &format!("https://api{}.example.com", i)).unwrap();
        conn.tags = if i % 2 == 0 { vec!["prod".into()] } else { vec!["staging".into()] };
        sess.store().save_connection(&conn).unwrap();
    }
    let prod = sess.store().list_connections(Some("prod")).unwrap();
    let staging = sess.store().list_connections(Some("staging")).unwrap();
    assert_eq!(prod.len(), 3); // 0, 2, 4
    assert_eq!(staging.len(), 2); // 1, 3
}
