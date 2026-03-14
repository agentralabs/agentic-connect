//! Communication tools — Inventions 13-15: Email, Telephony, Webhooks.

use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Invention 13: Email Intelligence
        def("connect_email_send", "Send email with template support", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "to": { "type": "array", "items": { "type": "string" } },
                "subject": { "type": "string" }, "body": { "type": "string" }
            }, "required": ["connection_id", "to", "subject", "body"]
        })),
        def("connect_email_read", "Read inbox with filters", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "folder": { "type": "string", "default": "INBOX" },
                "limit": { "type": "integer", "default": 20 }
            }, "required": ["connection_id"]
        })),
        def("connect_email_thread", "Reconstruct email thread", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "message_id": { "type": "string" }
            }, "required": ["connection_id", "message_id"]
        })),
        def("connect_email_actions", "Extract action items from emails", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "message_id": { "type": "string" }
            }, "required": ["connection_id", "message_id"]
        })),
        def("connect_email_draft", "Draft contextual reply", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "message_id": { "type": "string" },
                "intent": { "type": "string" }
            }, "required": ["connection_id", "message_id"]
        })),
        def("connect_email_search", "Search emails by content", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "query": { "type": "string" },
                "limit": { "type": "integer", "default": 20 }
            }, "required": ["connection_id", "query"]
        })),
        // Invention 14: Telephony Bridge
        def("connect_phone_call", "Place or manage a phone call", json!({
            "type": "object", "properties": {
                "action": { "type": "string", "enum": ["dial","hangup","hold","transfer"] },
                "number": { "type": "string" }
            }, "required": ["number"]
        })),
        def("connect_phone_voicemail", "Retrieve and transcribe voicemail", json!({
            "type": "object", "properties": { "limit": { "type": "integer", "default": 10 } }
        })),
        def("connect_phone_ivr", "Navigate IVR phone menu by intent", json!({
            "type": "object", "properties": {
                "number": { "type": "string" }, "intent": { "type": "string" }
            }, "required": ["number", "intent"]
        })),
        def("connect_phone_conference", "Create or manage conference call", json!({
            "type": "object", "properties": {
                "action": { "type": "string", "enum": ["create","add","remove","end"] },
                "participants": { "type": "array", "items": { "type": "string" } }
            }
        })),
        def("connect_phone_sms", "Send or receive SMS messages", json!({
            "type": "object", "properties": {
                "action": { "type": "string", "enum": ["send","receive","list"] },
                "to": { "type": "string" }, "message": { "type": "string" }
            }
        })),
        // Invention 15: Webhook Intelligence
        def("connect_webhook_register", "Register a webhook endpoint", json!({
            "type": "object", "properties": {
                "name": { "type": "string" }, "url": { "type": "string" },
                "events": { "type": "array", "items": { "type": "string" } },
                "secret": { "type": "string" }
            }, "required": ["name", "url"]
        })),
        def("connect_webhook_send", "Send webhook with HMAC signature and retry", json!({
            "type": "object", "properties": {
                "url": { "type": "string" }, "payload": { "type": "object" },
                "secret": { "type": "string" }
            }, "required": ["url", "payload"]
        })),
        def("connect_webhook_receive", "View received webhooks", json!({
            "type": "object", "properties": { "name": { "type": "string" }, "limit": { "type": "integer", "default": 20 } }
        })),
        def("connect_webhook_route", "Configure webhook routing rules", json!({
            "type": "object", "properties": {
                "name": { "type": "string" }, "rules": { "type": "array", "items": { "type": "object" } }
            }, "required": ["name", "rules"]
        })),
        def("connect_webhook_verify", "Verify webhook HMAC-SHA256 signature", json!({
            "type": "object", "properties": {
                "payload": { "type": "string" }, "signature": { "type": "string" },
                "secret": { "type": "string" }
            }, "required": ["payload", "signature", "secret"]
        })),
    ]
}

pub async fn try_execute(
    name: &str,
    args: Value,
    session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    match name {
        "connect_webhook_send" => Some(exec_webhook_send(args, session).await),
        "connect_webhook_verify" => Some(exec_webhook_verify(args)),
        n if n.starts_with("connect_email_") || n.starts_with("connect_phone_") => {
            Some(Ok(ToolCallResult::json(&json!({ "tool": name, "status": "requires_email_or_telephony_feature" }))))
        }
        n if n.starts_with("connect_webhook_") => {
            Some(Ok(ToolCallResult::json(&json!({ "tool": name, "status": "stub" }))))
        }
        _ => None,
    }
}

async fn exec_webhook_send(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let url = args.get("url").and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("Missing url".into()))?;
    let payload = args.get("payload").cloned().unwrap_or(json!({}));
    let secret = args.get("secret").and_then(|v| v.as_str());

    let delivery = agentic_connect::engine::webhook::send_webhook(url, &payload, secret).await
        .map_err(|e| McpError::Internal(e.to_string()))?;

    // Record in retry engine
    let mut sess = session.lock().await;
    if delivery.success {
        sess.retry_mut().record_success(url);
    } else {
        let class = agentic_connect::RetryEngine::classify_http_status(delivery.status);
        sess.retry_mut().record_failure(url, class, &format!("HTTP {}", delivery.status), Some(delivery.status));
    }

    Ok(ToolCallResult::json(&delivery))
}

fn exec_webhook_verify(args: Value) -> McpResult<ToolCallResult> {
    let payload = args.get("payload").and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("Missing payload".into()))?;
    let signature = args.get("signature").and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("Missing signature".into()))?;
    let secret = args.get("secret").and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("Missing secret".into()))?;

    let valid = agentic_connect::engine::webhook::verify_hmac_sha256(secret, payload, signature);
    let expected = agentic_connect::engine::webhook::compute_hmac_sha256(secret, payload);

    Ok(ToolCallResult::json(&json!({
        "valid": valid,
        "expected_signature": format!("sha256={}", expected),
        "provided_signature": signature,
    })))
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
