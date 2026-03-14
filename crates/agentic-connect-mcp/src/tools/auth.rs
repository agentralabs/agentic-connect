//! Auth tools — Invention 2: Adaptive Authentication.

use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};
use agentic_connect::types::auth::AuthMethod;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "connect_auth_configure".into(),
            description: Some("Configure authentication for a connection".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connection_id": { "type": "string", "description": "Connection UUID" },
                    "auth_type": {
                        "type": "string",
                        "enum": ["none","basic","bearer","api_key","oauth2","ssh_key","ssh_password","mtls"],
                        "description": "Authentication method type"
                    },
                    "credentials": { "type": "object", "description": "Auth-type-specific credential fields" }
                },
                "required": ["connection_id", "auth_type"]
            }),
        },
        ToolDefinition {
            name: "connect_auth_test".into(),
            description: Some("Test if authentication credentials are valid".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connection_id": { "type": "string", "description": "Connection UUID to test auth for" }
                },
                "required": ["connection_id"]
            }),
        },
        ToolDefinition {
            name: "connect_auth_refresh".into(),
            description: Some("Force token refresh for OAuth2 connections".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connection_id": { "type": "string", "description": "Connection UUID" }
                },
                "required": ["connection_id"]
            }),
        },
        ToolDefinition {
            name: "connect_auth_rotate".into(),
            description: Some("Rotate credentials for a connection".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connection_id": { "type": "string", "description": "Connection UUID" },
                    "new_credentials": { "type": "object", "description": "New credential values" }
                },
                "required": ["connection_id"]
            }),
        },
        ToolDefinition {
            name: "connect_auth_vault".into(),
            description: Some("Manage the encrypted credential vault".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list", "store", "retrieve", "delete"],
                        "description": "Vault operation"
                    },
                    "name": { "type": "string", "description": "Credential name (for store/retrieve/delete)" },
                    "credentials": { "type": "object", "description": "Credentials to store (for store action)" }
                },
                "required": ["action"]
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
        "connect_auth_configure" => Some(execute_configure(args, session).await),
        "connect_auth_test" => Some(execute_test(args, session).await),
        "connect_auth_refresh" => Some(execute_refresh(args, session).await),
        "connect_auth_rotate" => Some(execute_rotate(args, session).await),
        "connect_auth_vault" => Some(execute_vault(args, session).await),
        _ => None,
    }
}

async fn execute_configure(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct Params { connection_id: String, auth_type: String, credentials: Option<Value> }
    let params: Params = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let id = uuid::Uuid::parse_str(&params.connection_id)
        .map_err(|e| McpError::InvalidParams(format!("Invalid UUID: {}", e)))?;

    let sess = session.lock().await;
    let conn = sess.store().get_connection(&id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidParams(format!("Connection not found: {}", id)))?;

    let auth = match params.auth_type.as_str() {
        "none" => AuthMethod::None,
        "bearer" => {
            let token = params.credentials.as_ref()
                .and_then(|c| c.get("token")).and_then(|t| t.as_str())
                .ok_or_else(|| McpError::InvalidParams("bearer requires 'token' field".into()))?;
            AuthMethod::Bearer { token: token.to_string() }
        }
        "basic" => {
            let creds = params.credentials.as_ref().ok_or_else(|| McpError::InvalidParams("basic requires credentials".into()))?;
            AuthMethod::Basic {
                username: creds.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                password: creds.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            }
        }
        "api_key" => {
            let creds = params.credentials.as_ref().ok_or_else(|| McpError::InvalidParams("api_key requires credentials".into()))?;
            AuthMethod::ApiKey {
                key: creds.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                header_name: creds.get("header_name").and_then(|v| v.as_str()).map(String::from),
                query_param: creds.get("query_param").and_then(|v| v.as_str()).map(String::from),
            }
        }
        other => return Err(McpError::InvalidParams(format!("Unsupported auth type: {}", other))),
    };

    let mut updated = conn;
    updated.auth = Some(auth);
    sess.store().save_connection(&updated).map_err(|e| McpError::Internal(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "configured": true,
        "connection_id": params.connection_id,
        "auth_type": params.auth_type,
    })))
}

async fn execute_test(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct Params { connection_id: String }
    let params: Params = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let id = uuid::Uuid::parse_str(&params.connection_id)
        .map_err(|e| McpError::InvalidParams(format!("Invalid UUID: {}", e)))?;
    let sess = session.lock().await;
    let conn = sess.store().get_connection(&id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidParams("Connection not found".into()))?;

    let has_auth = conn.auth.is_some();
    let auth_type = conn.auth.as_ref().map(|a| a.method_name()).unwrap_or("none");
    let expired = conn.auth.as_ref().map_or(false, |a| a.is_expired());

    Ok(ToolCallResult::json(&json!({
        "connection_id": params.connection_id,
        "has_auth": has_auth,
        "auth_type": auth_type,
        "expired": expired,
        "needs_refresh": conn.auth.as_ref().map_or(false, |a| a.needs_refresh()),
    })))
}

async fn execute_refresh(_args: Value, _session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    Ok(ToolCallResult::json(&json!({
        "refreshed": false,
        "reason": "OAuth2 token refresh not yet implemented — requires HTTP client"
    })))
}

async fn execute_rotate(_args: Value, _session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    Ok(ToolCallResult::json(&json!({
        "rotated": false,
        "reason": "Credential rotation not yet implemented"
    })))
}

async fn execute_vault(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct Params { action: String, name: Option<String> }
    let params: Params = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    match params.action.as_str() {
        "list" => {
            let sess = session.lock().await;
            let conns = sess.store().list_connections(None).map_err(|e| McpError::Internal(e.to_string()))?;
            let with_auth: Vec<_> = conns.iter()
                .filter(|c| c.auth.is_some())
                .map(|c| json!({ "name": c.name, "auth_type": c.auth.as_ref().map(|a| a.method_name()) }))
                .collect();
            Ok(ToolCallResult::json(&json!({ "credentials": with_auth, "count": with_auth.len() })))
        }
        other => Ok(ToolCallResult::json(&json!({
            "action": other,
            "status": "not_implemented",
            "hint": "Vault store/retrieve/delete require encryption setup"
        }))),
    }
}
