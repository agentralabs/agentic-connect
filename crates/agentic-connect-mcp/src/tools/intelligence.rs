//! Intelligence tools — Inventions 21-24: Prophecy, Evolution, Dream, Collective.

use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Invention 21: Connection Prophecy
        def("connect_prophecy_predict", "Predict likely connection failures", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "horizon": { "type": "string", "enum": ["1h","24h","7d","30d"], "default": "7d" }
            }
        })),
        def("connect_prophecy_trends", "View concerning trends across connections", json!({
            "type": "object", "properties": {
                "min_severity": { "type": "string", "enum": ["info","warning","critical"], "default": "warning" }
            }
        })),
        def("connect_prophecy_recommend", "Get proactive recommendations", json!({
            "type": "object", "properties": {
                "scope": { "type": "string", "enum": ["all","performance","security","reliability"], "default": "all" }
            }
        })),
        def("connect_prophecy_simulate", "Simulate failure scenarios", json!({
            "type": "object", "properties": {
                "scenario": { "type": "string", "enum": ["dns_failure","cert_expiry","rate_limit","cascade","network_partition"] },
                "connection_id": { "type": "string" }
            }, "required": ["scenario"]
        })),
        // Invention 22: Protocol Evolution
        def("connect_evolve_track", "Track API response schema over time", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "endpoint": { "type": "string" }
            }, "required": ["connection_id", "endpoint"]
        })),
        def("connect_evolve_drift", "Detect schema drift in API responses", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }
            }, "required": ["connection_id"]
        })),
        def("connect_evolve_deprecated", "Identify likely deprecated API fields", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }
            }, "required": ["connection_id"]
        })),
        def("connect_evolve_breaking", "Detect breaking changes in APIs", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "since": { "type": "string", "description": "ISO date to check changes since" }
            }, "required": ["connection_id"]
        })),
        def("connect_evolve_adapt", "Auto-adapt to detected API changes", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "change_id": { "type": "string" }
            }, "required": ["connection_id"]
        })),
        // Invention 23: Connection Dream State
        def("connect_dream_start", "Start idle connection maintenance", json!({
            "type": "object", "properties": {
                "scope": { "type": "string", "enum": ["all","critical","recent"], "default": "critical" }
            }
        })),
        def("connect_dream_insights", "Get discoveries from idle health checks", json!({
            "type": "object", "properties": {
                "since": { "type": "string", "description": "ISO date" }
            }
        })),
        def("connect_dream_refresh", "Refresh all connection profiles during idle", json!({
            "type": "object", "properties": {}
        })),
        def("connect_dream_health", "Get proactive health report from dream state", json!({
            "type": "object", "properties": {
                "format": { "type": "string", "enum": ["summary","detailed"], "default": "summary" }
            }
        })),
        // Invention 24: Connection Collective
        def("connect_collective_share", "Share a learned API behavior with community", json!({
            "type": "object", "properties": {
                "api_name": { "type": "string" },
                "behavior": { "type": "object", "description": "Learned behavior to share" }
            }, "required": ["api_name", "behavior"]
        })),
        def("connect_collective_search", "Search for known API behaviors", json!({
            "type": "object", "properties": {
                "api_name": { "type": "string" },
                "query": { "type": "string" }
            }, "required": ["api_name"]
        })),
        def("connect_collective_apply", "Apply community knowledge to a connection", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "knowledge_id": { "type": "string" }
            }, "required": ["connection_id", "knowledge_id"]
        })),
        def("connect_collective_rate", "Rate community knowledge accuracy", json!({
            "type": "object", "properties": {
                "knowledge_id": { "type": "string" },
                "accurate": { "type": "boolean" },
                "comment": { "type": "string" }
            }, "required": ["knowledge_id", "accurate"]
        })),
        def("connect_collective_private", "Verify no private data in shared knowledge", json!({
            "type": "object", "properties": {
                "knowledge_id": { "type": "string" }
            }, "required": ["knowledge_id"]
        })),
    ]
}

pub async fn try_execute(
    name: &str,
    _args: Value,
    _session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    if name.starts_with("connect_prophecy_") || name.starts_with("connect_evolve_")
        || name.starts_with("connect_dream_") || name.starts_with("connect_collective_")
    {
        Some(Ok(ToolCallResult::json(&json!({
            "tool": name, "status": "stub",
            "hint": "Intelligence tools require connection history data to function"
        }))))
    } else {
        None
    }
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
