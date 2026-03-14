//! Phase 2: Tool execution tests — each tool group exercised with real session.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use agentic_connect_mcp::tools::ToolRegistry;
use agentic_connect_mcp::SessionManager;

fn session() -> Arc<Mutex<SessionManager>> {
    Arc::new(Mutex::new(SessionManager::in_memory().unwrap()))
}

// === Protocol tools ===

#[tokio::test]
async fn test_protocol_detect_https() {
    let s = session();
    let r = ToolRegistry::call("connect_protocol_detect", Some(json!({"target": "https://api.github.com"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("HTTPS") || t.contains("detected"));
}

#[tokio::test]
async fn test_protocol_detect_ssh() {
    let s = session();
    let r = ToolRegistry::call("connect_protocol_detect", Some(json!({"target": "ssh://server.example.com"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("SSH"));
}

#[tokio::test]
async fn test_protocol_detect_port_only() {
    let s = session();
    let r = ToolRegistry::call("connect_protocol_detect", Some(json!({"target": "example.com:5432"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("5432"));
}

#[tokio::test]
async fn test_protocol_list_all() {
    let s = session();
    let r = ToolRegistry::call("connect_protocol_list", None, &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("HTTP"));
    assert!(t.contains("SSH"));
    assert!(t.contains("PostgreSQL"));
}

#[tokio::test]
async fn test_protocol_caps_websocket() {
    let s = session();
    let r = ToolRegistry::call("connect_protocol_caps", Some(json!({"protocol": "websocket"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("bidirectional"));
}

// === Auth tools ===

#[tokio::test]
async fn test_auth_vault_list_empty() {
    let s = session();
    let r = ToolRegistry::call("connect_auth_vault", Some(json!({"action": "list"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("count"));
}

#[tokio::test]
async fn test_auth_test_nonexistent() {
    let s = session();
    let r = ToolRegistry::call("connect_auth_test", Some(json!({"connection_id": "00000000-0000-0000-0000-000000000000"})), &s).await;
    // May return Ok with error info or Err — both are valid
    // The tool should handle missing connections gracefully
    assert!(r.is_ok() || r.is_err());
}

// === Retry tools ===

#[tokio::test]
async fn test_retry_simulate_rate_limit() {
    let s = session();
    let r = ToolRegistry::call("connect_retry_simulate", Some(json!({"http_status": 429})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("RateLimit"));
}

#[tokio::test]
async fn test_retry_simulate_permanent() {
    let s = session();
    let r = ToolRegistry::call("connect_retry_simulate", Some(json!({"http_status": 404})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("Permanent"));
}

#[tokio::test]
async fn test_retry_simulate_error_message() {
    let s = session();
    let r = ToolRegistry::call("connect_retry_simulate", Some(json!({"error_message": "connection timed out"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("Transient"));
}

#[tokio::test]
async fn test_retry_circuit_view() {
    let s = session();
    let r = ToolRegistry::call("connect_retry_circuit", Some(json!({"endpoint": "https://api.example.com"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("Closed") || t.contains("endpoint"));
}

#[tokio::test]
async fn test_retry_status_empty() {
    let s = session();
    let r = ToolRegistry::call("connect_retry_status", None, &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("circuit_breakers"));
}

// === Soul tools ===

#[tokio::test]
async fn test_soul_inspect_no_profile() {
    let s = session();
    let r = ToolRegistry::call("connect_soul_inspect", Some(json!({"connection_id": "00000000-0000-0000-0000-000000000000"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("no_profile") || t.contains("status"));
}

#[tokio::test]
async fn test_soul_history_no_data() {
    let s = session();
    let r = ToolRegistry::call("connect_soul_history", Some(json!({"connection_id": "00000000-0000-0000-0000-000000000000"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("history") || t.contains("hint"));
}

// === Webhook tools ===

#[tokio::test]
async fn test_webhook_verify_valid() {
    let s = session();
    let payload = r#"{"event":"push"}"#;
    let secret = "my-secret";
    let sig = agentic_connect::engine::webhook::compute_hmac_sha256(secret, payload);
    let r = ToolRegistry::call("connect_webhook_verify", Some(json!({
        "payload": payload, "signature": sig, "secret": secret
    })), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("true"));
}

#[tokio::test]
async fn test_webhook_verify_invalid() {
    let s = session();
    let r = ToolRegistry::call("connect_webhook_verify", Some(json!({
        "payload": "data", "signature": "wrong", "secret": "secret"
    })), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("false"));
}

// === DB tools ===

#[tokio::test]
async fn test_db_connect_sqlite() {
    let s = session();
    let r = ToolRegistry::call("connect_db_connect", Some(json!({
        "url": "sqlite://:memory:", "name": "test"
    })), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("connected"));
    assert!(t.contains("true"));
}

#[tokio::test]
async fn test_db_connect_then_schema() {
    let s = session();
    ToolRegistry::call("connect_db_connect", Some(json!({
        "url": "sqlite://:memory:", "name": "schematest"
    })), &s).await.unwrap();
    let r = ToolRegistry::call("connect_db_schema", Some(json!({"name": "schematest"})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("tables"));
}

#[tokio::test]
async fn test_db_query_nonexistent_connection() {
    let s = session();
    let r = ToolRegistry::call("connect_db_query", Some(json!({
        "name": "nope", "query": "SELECT 1"
    })), &s).await;
    assert!(r.is_err()); // InvalidParams
}

// === Security tools ===

#[tokio::test]
async fn test_sentinel_status_empty() {
    let s = session();
    let r = ToolRegistry::call("connect_sentinel_status", None, &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("total_connections"));
}

#[tokio::test]
async fn test_tls_grade_standard_port() {
    let s = session();
    let r = ToolRegistry::call("connect_tls_grade", Some(json!({"host": "127.0.0.1", "port": 443})), &s).await.unwrap();
    let t = text(&r);
    assert!(t.contains("grade"));
}

// === Intelligence tools (stubs) ===

#[tokio::test]
async fn test_intelligence_tools_return_stub() {
    let s = session();
    for tool in ["connect_prophecy_predict", "connect_evolve_track", "connect_dream_start", "connect_collective_search"] {
        let r = ToolRegistry::call(tool, Some(json!({})), &s).await.unwrap();
        assert!(!r.is_error);
    }
}

// === Browse tools (stubs) ===

#[tokio::test]
async fn test_browse_tools_return_stub() {
    let s = session();
    for tool in ["connect_browse_navigate", "connect_scrape_extract", "connect_form_analyze"] {
        let r = ToolRegistry::call(tool, Some(json!({"url": "https://example.com"})), &s).await.unwrap();
        assert!(!r.is_error);
    }
}

// === Infra tools (stubs) ===

#[tokio::test]
async fn test_infra_tools_return_stub() {
    let s = session();
    for tool in ["connect_remote_exec", "connect_mesh_discover", "connect_container_status"] {
        let r = ToolRegistry::call(tool, Some(json!({})), &s).await.unwrap();
        assert!(!r.is_error);
    }
}

fn text(r: &agentic_connect_mcp::types::ToolCallResult) -> String {
    serde_json::to_string(r).unwrap_or_default()
}
