//! Infrastructure tools — Inventions 10-12: Remote, Mesh, Container.

use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        def("connect_remote_exec", "Execute command with pre/post condition checks", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "command": { "type": "string" },
                "check_before": { "type": "string" },
                "check_after": { "type": "string" }
            }, "required": ["connection_id", "command"]
        })),
        def("connect_remote_plan", "Create multi-step execution plan", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "steps": { "type": "array", "items": { "type": "object" } }
            }, "required": ["connection_id", "steps"]
        })),
        def("connect_remote_verify", "Verify system state matches expectations", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "checks": { "type": "array", "items": { "type": "object" } }
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
        def("connect_mesh_discover", "Discover services and dependencies", json!({
            "type": "object", "properties": {
                "entry_point": { "type": "string" }
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
                "tail": { "type": "integer", "default": 100 }
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
    args: Value,
    session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    match name {
        "connect_remote_exec" => Some(exec_remote(args, session).await),
        "connect_remote_plan" => Some(exec_plan(args)),
        "connect_remote_verify" => Some(exec_verify(args)),
        "connect_mesh_health" => Some(exec_mesh_health(session).await),
        "connect_mesh_cascade" => Some(exec_cascade(session).await),
        "connect_mesh_topology" => Some(exec_topology(args, session).await),
        "connect_container_status" => Some(exec_container_status(args)),
        "connect_container_logs" => Some(exec_container_logs(args)),
        n if n.starts_with("connect_remote_") || n.starts_with("connect_mesh_")
            || n.starts_with("connect_container_") =>
        {
            Some(Ok(ToolCallResult::json(&json!({
                "tool": name, "status": "requires_ssh_feature",
                "hint": "Full implementation requires SSH or Docker runtime"
            }))))
        }
        _ => None,
    }
}

async fn exec_remote(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let conn_id = args.get("connection_id").and_then(|v| v.as_str()).unwrap_or("");
    let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let check_before = args.get("check_before").and_then(|v| v.as_str());
    let check_after = args.get("check_after").and_then(|v| v.as_str());

    let sess = session.lock().await;
    let id = uuid::Uuid::parse_str(conn_id).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let conn = sess.store().get_connection(&id)
        .map_err(|e| McpError::Internal(e.to_string()))?
        .ok_or_else(|| McpError::InvalidParams("Connection not found".into()))?;

    Ok(ToolCallResult::json(&json!({
        "connection": conn.name,
        "host": conn.host,
        "command": command,
        "pre_check": check_before,
        "post_check": check_after,
        "status": "planned",
        "hint": format!("Would execute '{}' on {} via {:?}", command, conn.host, conn.protocol),
    })))
}

fn exec_plan(args: Value) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P { connection_id: String, steps: Vec<Value> }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let plan: Vec<_> = p.steps.iter().enumerate().map(|(i, s)| {
        json!({ "step": i + 1, "command": s.get("command"), "status": "pending" })
    }).collect();
    Ok(ToolCallResult::json(&json!({
        "connection_id": p.connection_id, "plan": plan, "step_count": plan.len()
    })))
}

fn exec_verify(args: Value) -> McpResult<ToolCallResult> {
    let checks = args.get("checks").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let results: Vec<_> = checks.iter().map(|c| {
        json!({ "check": c, "status": "not_verified", "hint": "Requires SSH connection" })
    }).collect();
    Ok(ToolCallResult::json(&json!({ "checks": results, "verified": 0, "total": results.len() })))
}

async fn exec_mesh_health(session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let conns = sess.store().list_connections(None).map_err(|e| McpError::Internal(e.to_string()))?;
    let services: Vec<_> = conns.iter().map(|c| {
        let allowed = sess.retry().should_allow(&c.url());
        json!({ "name": c.name, "host": c.host, "circuit": if allowed { "closed" } else { "open" } })
    }).collect();
    Ok(ToolCallResult::json(&json!({ "services": services, "total": services.len() })))
}

async fn exec_cascade(session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let failures = sess.retry().recent_failures(20);
    let cascades: Vec<_> = failures.iter().map(|f| {
        json!({ "endpoint": f.endpoint, "class": format!("{:?}", f.failure_class), "time": f.timestamp.to_rfc3339() })
    }).collect();
    Ok(ToolCallResult::json(&json!({ "potential_cascades": cascades, "count": cascades.len() })))
}

async fn exec_topology(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("json");
    let sess = session.lock().await;
    let conns = sess.store().list_connections(None).map_err(|e| McpError::Internal(e.to_string()))?;
    match format {
        "mermaid" => {
            let mut diagram = "graph TD\n".to_string();
            for c in &conns {
                diagram.push_str(&format!("  {}[\"{}\\n{}\"]\n", c.name.replace('-', "_"), c.name, c.host));
            }
            Ok(ToolCallResult::text(diagram))
        }
        _ => {
            let nodes: Vec<_> = conns.iter().map(|c| json!({"name": c.name, "host": c.host, "protocol": format!("{:?}", c.protocol)})).collect();
            Ok(ToolCallResult::json(&json!({ "nodes": nodes, "edges": [], "format": format })))
        }
    }
}

fn exec_container_status(args: Value) -> McpResult<ToolCallResult> {
    let name = args.get("name").and_then(|v| v.as_str());
    let platform = args.get("platform").and_then(|v| v.as_str()).unwrap_or("auto");
    // Try docker ps
    let output = std::process::Command::new("docker").args(["ps", "--format", "{{.Names}}\t{{.Status}}\t{{.Image}}"]).output();
    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let containers: Vec<_> = stdout.lines().filter(|l| {
                name.map_or(true, |n| l.contains(n))
            }).map(|l| {
                let parts: Vec<&str> = l.split('\t').collect();
                json!({"name": parts.get(0), "status": parts.get(1), "image": parts.get(2)})
            }).collect();
            Ok(ToolCallResult::json(&json!({"containers": containers, "platform": "docker", "count": containers.len()})))
        }
        _ => Ok(ToolCallResult::json(&json!({"containers": [], "platform": platform, "hint": "Docker not available or not running"}))),
    }
}

fn exec_container_logs(args: Value) -> McpResult<ToolCallResult> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let tail = args.get("tail").and_then(|v| v.as_u64()).unwrap_or(100);
    let output = std::process::Command::new("docker").args(["logs", "--tail", &tail.to_string(), name]).output();
    match output {
        Ok(o) if o.status.success() => {
            let logs = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let combined = if logs.is_empty() { stderr.to_string() } else { logs.to_string() };
            let truncated = if combined.len() > 4000 {
                format!("{}...[truncated]", &combined[..4000])
            } else { combined };
            Ok(ToolCallResult::text(truncated))
        }
        Ok(o) => Ok(ToolCallResult::error(format!("docker logs failed: {}", String::from_utf8_lossy(&o.stderr)))),
        Err(e) => Ok(ToolCallResult::error(format!("Docker not available: {}", e))),
    }
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
