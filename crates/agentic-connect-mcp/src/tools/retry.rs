//! Retry tools — Invention 4: Intelligent Retry Fabric.

use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};
use agentic_connect::engine::retry_engine::RetryEngine;
use agentic_connect::types::retry::FailureClass;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        def("connect_retry_configure", "Configure retry policies per connection", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "max_attempts": { "type": "integer", "default": 3 },
                "backoff_base_ms": { "type": "integer", "default": 1000 },
                "backoff_max_ms": { "type": "integer", "default": 30000 }
            }, "required": ["connection_id"]
        })),
        def("connect_retry_status", "View retry and circuit breaker state for all connections", json!({
            "type": "object", "properties": {}
        })),
        def("connect_retry_patterns", "View learned failure patterns for a connection", json!({
            "type": "object", "properties": {
                "endpoint": { "type": "string", "description": "Endpoint to check patterns for" },
                "limit": { "type": "integer", "default": 20 }
            }, "required": ["endpoint"]
        })),
        def("connect_retry_circuit", "View or reset circuit breaker state", json!({
            "type": "object", "properties": {
                "endpoint": { "type": "string" },
                "action": { "type": "string", "enum": ["view", "reset"], "default": "view" }
            }, "required": ["endpoint"]
        })),
        def("connect_retry_simulate", "Simulate a failure to test retry classification", json!({
            "type": "object", "properties": {
                "failure_type": { "type": "string", "enum": ["transient","permanent","rate_limit","auth_failure","network_error","server_error"] },
                "http_status": { "type": "integer", "description": "HTTP status code to classify" },
                "error_message": { "type": "string", "description": "Error message to classify" }
            }
        })),
    ]
}

pub async fn try_execute(
    name: &str,
    args: Value,
    session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    match name {
        "connect_retry_configure" => Some(execute_configure(args)),
        "connect_retry_status" => Some(execute_status(session).await),
        "connect_retry_patterns" => Some(execute_patterns(args, session).await),
        "connect_retry_circuit" => Some(execute_circuit(args, session).await),
        "connect_retry_simulate" => Some(execute_simulate(args)),
        _ => None,
    }
}

fn execute_configure(args: Value) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P { connection_id: String, max_attempts: Option<u32>, backoff_base_ms: Option<u64>, backoff_max_ms: Option<u64> }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "configured": true,
        "connection_id": p.connection_id,
        "policy": {
            "max_attempts": p.max_attempts.unwrap_or(3),
            "backoff_base_ms": p.backoff_base_ms.unwrap_or(1000),
            "backoff_max_ms": p.backoff_max_ms.unwrap_or(30000),
        }
    })))
}

async fn execute_status(session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let circuits = sess.retry().all_circuits();
    let circuit_list: Vec<_> = circuits.values().map(|cb| json!({
        "endpoint": cb.endpoint,
        "state": format!("{:?}", cb.state),
        "failure_count": cb.failure_count,
        "threshold": cb.failure_threshold,
        "allowed": cb.should_allow(),
    })).collect();

    let recent = sess.retry().recent_failures(10);
    let failures: Vec<_> = recent.iter().map(|f| json!({
        "endpoint": f.endpoint,
        "class": format!("{:?}", f.failure_class),
        "message": f.message,
        "timestamp": f.timestamp.to_rfc3339(),
    })).collect();

    Ok(ToolCallResult::json(&json!({
        "circuit_breakers": circuit_list,
        "recent_failures": failures,
    })))
}

async fn execute_patterns(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let endpoint = args.get("endpoint").and_then(|v| v.as_str()).unwrap_or("");
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let sess = session.lock().await;
    let patterns = sess.retry().failure_patterns(endpoint);
    let list: Vec<_> = patterns.iter().take(limit).map(|f| json!({
        "class": format!("{:?}", f.failure_class),
        "status": f.http_status,
        "message": f.message,
        "timestamp": f.timestamp.to_rfc3339(),
    })).collect();

    Ok(ToolCallResult::json(&json!({
        "endpoint": endpoint,
        "pattern_count": patterns.len(),
        "patterns": list,
    })))
}

async fn execute_circuit(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let endpoint = args.get("endpoint").and_then(|v| v.as_str()).unwrap_or("");
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("view");

    let mut sess = session.lock().await;
    if action == "reset" {
        let reset = sess.retry_mut().reset_circuit(endpoint);
        return Ok(ToolCallResult::json(&json!({
            "endpoint": endpoint, "action": "reset", "success": reset
        })));
    }

    let cb = sess.retry_mut().get_circuit(endpoint);
    Ok(ToolCallResult::json(&json!({
        "endpoint": cb.endpoint,
        "state": format!("{:?}", cb.state),
        "failure_count": cb.failure_count,
        "threshold": cb.failure_threshold,
        "allowed": cb.should_allow(),
        "last_failure": cb.last_failure,
    })))
}

fn execute_simulate(args: Value) -> McpResult<ToolCallResult> {
    if let Some(status) = args.get("http_status").and_then(|v| v.as_u64()) {
        let class = RetryEngine::classify_http_status(status as u16);
        let strategy = RetryEngine::strategy_for(class);
        return Ok(ToolCallResult::json(&json!({
            "input": { "http_status": status },
            "classification": format!("{:?}", class),
            "strategy": format!("{:?}", strategy),
        })));
    }

    if let Some(msg) = args.get("error_message").and_then(|v| v.as_str()) {
        let class = RetryEngine::classify_error(msg);
        let strategy = RetryEngine::strategy_for(class);
        return Ok(ToolCallResult::json(&json!({
            "input": { "error_message": msg },
            "classification": format!("{:?}", class),
            "strategy": format!("{:?}", strategy),
        })));
    }

    Ok(ToolCallResult::json(&json!({
        "hint": "Provide http_status or error_message to simulate classification"
    })))
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
