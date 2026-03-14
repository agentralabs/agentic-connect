//! Infrastructure tools — Inventions 10-12: Remote, Mesh, Container.

use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Invention 10: Remote Command Intelligence
        def("connect_remote_exec", "Execute command with pre/post condition checks", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "command": { "type": "string", "description": "Command to execute" },
                "check_before": { "type": "string", "description": "Pre-condition command (must return 0)" },
                "check_after": { "type": "string", "description": "Post-condition command (must return 0)" }
            }, "required": ["connection_id", "command"]
        })),
        def("connect_remote_plan", "Create multi-step execution plan", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "steps": { "type": "array", "items": { "type": "object" }, "description": "Ordered steps with commands" }
            }, "required": ["connection_id", "steps"]
        })),
        def("connect_remote_verify", "Verify system state matches expectations", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "checks": { "type": "array", "items": { "type": "object" }, "description": "State expectations to verify" }
            }, "required": ["connection_id", "checks"]
        })),
        def("connect_remote_rollback", "Execute rollback plan", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "plan_id": { "type": "string" }
            }, "required": ["connection_id"]
        })),
        def("connect_remote_transfer", "Transfer files with verification", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "source": { "type": "string" },
                "destination": { "type": "string" },
                "direction": { "type": "string", "enum": ["upload","download"], "default": "upload" }
            }, "required": ["connection_id", "source", "destination"]
        })),
        def("connect_remote_tunnel", "Create SSH tunnel for port forwarding", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "local_port": { "type": "integer" },
                "remote_host": { "type": "string", "default": "localhost" },
                "remote_port": { "type": "integer" }
            }, "required": ["connection_id", "local_port", "remote_port"]
        })),
        // Invention 11: Service Mesh Awareness
        def("connect_mesh_discover", "Discover services and dependencies", json!({
            "type": "object", "properties": {
                "entry_point": { "type": "string", "description": "Starting URL or host" }
            }, "required": ["entry_point"]
        })),
        def("connect_mesh_health", "Health check all services in parallel", json!({
            "type": "object", "properties": {
                "timeout_ms": { "type": "integer", "default": 5000 }
            }
        })),
        def("connect_mesh_trace", "Trace request through service chain", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "trace_id": { "type": "string" }
            }, "required": ["url"]
        })),
        def("connect_mesh_cascade", "Detect cascading failure patterns", json!({
            "type": "object", "properties": {}
        })),
        def("connect_mesh_topology", "View or export service topology map", json!({
            "type": "object", "properties": {
                "format": { "type": "string", "enum": ["json","dot","mermaid"], "default": "json" }
            }
        })),
        // Invention 12: Container Orchestration
        def("connect_container_deploy", "Deploy containers to any platform", json!({
            "type": "object", "properties": {
                "image": { "type": "string" },
                "name": { "type": "string" },
                "replicas": { "type": "integer", "default": 1 },
                "platform": { "type": "string", "enum": ["docker","kubernetes","compose","auto"], "default": "auto" }
            }, "required": ["image"]
        })),
        def("connect_container_status", "View container status across platforms", json!({
            "type": "object", "properties": {
                "name": { "type": "string" },
                "platform": { "type": "string", "enum": ["docker","kubernetes","compose","all"], "default": "all" }
            }
        })),
        def("connect_container_logs", "Stream container logs", json!({
            "type": "object", "properties": {
                "name": { "type": "string" },
                "tail": { "type": "integer", "default": 100 },
                "follow": { "type": "boolean", "default": false }
            }, "required": ["name"]
        })),
        def("connect_container_scale", "Scale containers up or down", json!({
            "type": "object", "properties": {
                "name": { "type": "string" },
                "replicas": { "type": "integer" }
            }, "required": ["name", "replicas"]
        })),
        def("connect_container_inspect", "Inspect running container details", json!({
            "type": "object", "properties": {
                "name": { "type": "string" }
            }, "required": ["name"]
        })),
    ]
}

pub async fn try_execute(
    name: &str,
    _args: Value,
    _session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    if name.starts_with("connect_remote_") || name.starts_with("connect_mesh_") || name.starts_with("connect_container_") {
        Some(Ok(ToolCallResult::json(&json!({
            "tool": name, "status": "stub",
            "hint": "Infrastructure tools require SSH/Docker feature flags"
        }))))
    } else {
        None
    }
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
