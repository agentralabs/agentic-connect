//! Tool registry tests — tool discovery, dispatch, error handling.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use agentic_connect_mcp::tools::ToolRegistry;
use agentic_connect_mcp::SessionManager;
use agentic_connect_mcp::types::McpError;

fn make_session() -> Arc<Mutex<SessionManager>> {
    Arc::new(Mutex::new(SessionManager::in_memory().unwrap()))
}

#[test]
fn test_list_tools_returns_all() {
    let tools = ToolRegistry::list_tools();
    assert!(tools.len() >= 120, "Expected 120+ tools, got {}", tools.len());
}

#[test]
fn test_all_tools_have_names() {
    let tools = ToolRegistry::list_tools();
    for tool in &tools {
        assert!(!tool.name.is_empty(), "Tool has empty name");
        assert!(tool.name.starts_with("connect_"), "Tool name doesn't start with connect_: {}", tool.name);
    }
}

#[test]
fn test_all_tools_have_descriptions() {
    let tools = ToolRegistry::list_tools();
    for tool in &tools {
        assert!(tool.description.is_some(), "Tool {} has no description", tool.name);
        let desc = tool.description.as_ref().unwrap();
        assert!(!desc.is_empty(), "Tool {} has empty description", tool.name);
        // MCP Quality Standard: verb-first, no trailing period
        let first_char = desc.chars().next().unwrap();
        assert!(first_char.is_uppercase(), "Tool {} description doesn't start with capital: {}", tool.name, desc);
        assert!(!desc.ends_with('.'), "Tool {} description has trailing period: {}", tool.name, desc);
    }
}

#[test]
fn test_all_tools_have_input_schema() {
    let tools = ToolRegistry::list_tools();
    for tool in &tools {
        assert!(tool.input_schema.is_object(), "Tool {} has non-object input_schema", tool.name);
        assert_eq!(tool.input_schema["type"], "object", "Tool {} schema type is not 'object'", tool.name);
    }
}

#[test]
fn test_no_duplicate_tool_names() {
    let tools = ToolRegistry::list_tools();
    let mut names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    names.sort();
    for window in names.windows(2) {
        assert_ne!(window[0], window[1], "Duplicate tool name: {}", window[0]);
    }
}

#[tokio::test]
async fn test_dispatch_known_tool() {
    let session = make_session();
    let result = ToolRegistry::call("connect_protocol_list", None, &session).await;
    assert!(result.is_ok());
    let r = result.unwrap();
    assert!(!r.is_error);
}

#[tokio::test]
async fn test_dispatch_unknown_tool() {
    let session = make_session();
    let result = ToolRegistry::call("nonexistent_tool", None, &session).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        McpError::ToolNotFound(msg) => assert!(msg.contains("nonexistent_tool")),
        other => panic!("Expected ToolNotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_dispatch_protocol_detect() {
    let session = make_session();
    let result = ToolRegistry::call(
        "connect_protocol_detect",
        Some(json!({"target": "https://example.com"})),
        &session,
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_retry_simulate() {
    let session = make_session();
    let result = ToolRegistry::call(
        "connect_retry_simulate",
        Some(json!({"http_status": 429})),
        &session,
    ).await;
    assert!(result.is_ok());
    let r = result.unwrap();
    let json = serde_json::to_value(&r).unwrap();
    let text = json["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("RateLimit"));
}

#[tokio::test]
async fn test_dispatch_webhook_verify() {
    let session = make_session();
    let secret = "test-secret";
    let payload = r#"{"event":"test"}"#;
    let sig = agentic_connect::engine::webhook::compute_hmac_sha256(secret, payload);
    let result = ToolRegistry::call(
        "connect_webhook_verify",
        Some(json!({"payload": payload, "signature": sig, "secret": secret})),
        &session,
    ).await;
    assert!(result.is_ok());
    let r = result.unwrap();
    let json = serde_json::to_value(&r).unwrap();
    let text = json["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("true"));
}

#[tokio::test]
async fn test_dispatch_db_connect_and_query() {
    let session = make_session();
    // Connect to in-memory SQLite
    let result = ToolRegistry::call(
        "connect_db_connect",
        Some(json!({"url": "sqlite://:memory:", "name": "testdb"})),
        &session,
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_soul_inspect_no_profile() {
    let session = make_session();
    let result = ToolRegistry::call(
        "connect_soul_inspect",
        Some(json!({"connection_id": "00000000-0000-0000-0000-000000000000"})),
        &session,
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_sentinel_status() {
    let session = make_session();
    let result = ToolRegistry::call("connect_sentinel_status", None, &session).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_retry_status() {
    let session = make_session();
    let result = ToolRegistry::call("connect_retry_status", None, &session).await;
    assert!(result.is_ok());
}
