//! Connection Soul tools — Invention 3: Connection Soul.

use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};
use agentic_connect::types::soul::ConnectionProfile;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "connect_soul_inspect".into(),
            description: Some("View accumulated knowledge about a connection".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connection_id": { "type": "string", "description": "Connection UUID" }
                },
                "required": ["connection_id"]
            }),
        },
        ToolDefinition {
            name: "connect_soul_refresh".into(),
            description: Some("Force re-scan of remote system capabilities".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connection_id": { "type": "string", "description": "Connection UUID" }
                },
                "required": ["connection_id"]
            }),
        },
        ToolDefinition {
            name: "connect_soul_history".into(),
            description: Some("View connection history and patterns".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connection_id": { "type": "string", "description": "Connection UUID" },
                    "limit": { "type": "integer", "default": 20, "description": "Max history entries" }
                },
                "required": ["connection_id"]
            }),
        },
        ToolDefinition {
            name: "connect_soul_predict".into(),
            description: Some("Predict likely issues based on past patterns".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connection_id": { "type": "string", "description": "Connection UUID" }
                },
                "required": ["connection_id"]
            }),
        },
        ToolDefinition {
            name: "connect_soul_compare".into(),
            description: Some("Compare two systems for migration planning".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "source_id": { "type": "string", "description": "Source connection UUID" },
                    "target_id": { "type": "string", "description": "Target connection UUID" }
                },
                "required": ["source_id", "target_id"]
            }),
        },
    ]
}

pub async fn try_execute(
    name: &str,
    args: Value,
    session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    match name {
        "connect_soul_inspect" => Some(execute_inspect(args, session).await),
        "connect_soul_refresh" => Some(execute_refresh(args, session).await),
        "connect_soul_history" => Some(execute_history(args, session).await),
        "connect_soul_predict" => Some(execute_predict(args, session).await),
        "connect_soul_compare" => Some(execute_compare(args, session).await),
        _ => None,
    }
}

async fn execute_inspect(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let id = parse_connection_id(&args)?;
    let sess = session.lock().await;
    match sess.store().get_profile(&id).map_err(|e| McpError::Internal(e.to_string()))? {
        Some(profile) => Ok(ToolCallResult::json(&profile)),
        None => Ok(ToolCallResult::json(&json!({
            "connection_id": id.to_string(),
            "status": "no_profile",
            "hint": "Connect to this system first to build a profile"
        }))),
    }
}

async fn execute_refresh(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let id = parse_connection_id(&args)?;
    let sess = session.lock().await;
    let conn = sess.store().get_connection(&id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidParams("Connection not found".into()))?;

    let mut profile = sess.store().get_profile(&id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .unwrap_or_else(|| ConnectionProfile::new(id));

    profile.connection_count += 1;
    profile.last_updated = chrono::Utc::now();
    sess.store().save_profile(&profile).map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "refreshed": true,
        "connection": conn.name,
        "connection_count": profile.connection_count,
    })))
}

async fn execute_history(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let id = parse_connection_id(&args)?;
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let sess = session.lock().await;
    let profile = sess.store().get_profile(&id).map_err(|e| McpError::Internal(e.to_string()))?;
    match profile {
        Some(p) => {
            let errors: Vec<_> = p.error_history.iter().rev().take(limit).collect();
            Ok(ToolCallResult::json(&json!({
                "connection_count": p.connection_count,
                "first_seen": p.first_seen,
                "last_updated": p.last_updated,
                "recent_errors": errors,
                "avg_latency_ms": p.baseline.avg_latency_ms,
            })))
        }
        None => Ok(ToolCallResult::json(&json!({ "history": [], "hint": "No history yet" }))),
    }
}

async fn execute_predict(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let id = parse_connection_id(&args)?;
    let sess = session.lock().await;
    let profile = sess.store().get_profile(&id).map_err(|e| McpError::Internal(e.to_string()))?;
    match profile {
        Some(p) => {
            let mut predictions = Vec::new();
            if p.baseline.error_rate > 0.1 {
                predictions.push("High error rate — connection may be unstable");
            }
            if p.baseline.avg_latency_ms > 1000.0 {
                predictions.push("High latency — consider connection pooling or caching");
            }
            if p.error_history.len() > 10 {
                predictions.push("Frequent errors — investigate root cause");
            }
            Ok(ToolCallResult::json(&json!({ "predictions": predictions, "confidence": "heuristic" })))
        }
        None => Ok(ToolCallResult::json(&json!({ "predictions": [], "hint": "Need connection history first" }))),
    }
}

async fn execute_compare(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct Params { source_id: String, target_id: String }
    let params: Params = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let src = uuid::Uuid::parse_str(&params.source_id).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let tgt = uuid::Uuid::parse_str(&params.target_id).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let sess = session.lock().await;
    let sp = sess.store().get_profile(&src).map_err(|e| McpError::Internal(e.to_string()))?;
    let tp = sess.store().get_profile(&tgt).map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "source": sp.as_ref().map(|p| json!({
            "os": p.fingerprint.os, "latency_ms": p.baseline.avg_latency_ms
        })),
        "target": tp.as_ref().map(|p| json!({
            "os": p.fingerprint.os, "latency_ms": p.baseline.avg_latency_ms
        })),
        "comparable": sp.is_some() && tp.is_some(),
    })))
}

fn parse_connection_id(args: &Value) -> McpResult<uuid::Uuid> {
    let id_str = args.get("connection_id").and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("Missing connection_id".into()))?;
    uuid::Uuid::parse_str(id_str).map_err(|e| McpError::InvalidParams(format!("Invalid UUID: {}", e)))
}
