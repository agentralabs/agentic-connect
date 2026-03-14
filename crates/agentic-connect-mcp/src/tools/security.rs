//! Security tools — Inventions 19-20: TLS Consciousness, Network Sentinel.

use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Invention 19: TLS Consciousness
        def("connect_tls_inspect", "Inspect TLS configuration of any host", json!({
            "type": "object", "properties": {
                "host": { "type": "string" }, "port": { "type": "integer", "default": 443 }
            }, "required": ["host"]
        })),
        def("connect_tls_certificates", "List certificates with expiry monitoring", json!({
            "type": "object", "properties": {
                "expiring_within_days": { "type": "integer", "default": 30 }
            }
        })),
        def("connect_tls_audit", "Audit cipher suites and TLS versions", json!({
            "type": "object", "properties": {
                "host": { "type": "string" }, "port": { "type": "integer", "default": 443 }
            }, "required": ["host"]
        })),
        def("connect_tls_expiry", "Check certificate expiry for multiple hosts", json!({
            "type": "object", "properties": {
                "hosts": { "type": "array", "items": { "type": "string" } }
            }, "required": ["hosts"]
        })),
        def("connect_tls_grade", "Grade TLS configuration quality (A-F)", json!({
            "type": "object", "properties": {
                "host": { "type": "string" }, "port": { "type": "integer", "default": 443 }
            }, "required": ["host"]
        })),
        // Invention 20: Network Sentinel
        def("connect_sentinel_probe", "Health check any endpoint across protocols", json!({
            "type": "object", "properties": {
                "target": { "type": "string" },
                "timeout_ms": { "type": "integer", "default": 5000 }
            }, "required": ["target"]
        })),
        def("connect_sentinel_monitor", "Start or manage continuous monitoring", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "interval_secs": { "type": "integer", "default": 60 },
                "action": { "type": "string", "enum": ["start","stop","status"], "default": "start" }
            }, "required": ["connection_id"]
        })),
        def("connect_sentinel_status", "View aggregate health across all services", json!({
            "type": "object", "properties": {}
        })),
        def("connect_sentinel_trends", "View performance trends over time", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "period": { "type": "string", "enum": ["1h","24h","7d","30d"], "default": "24h" }
            }, "required": ["connection_id"]
        })),
        def("connect_sentinel_alert", "Configure alert thresholds", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "max_latency_ms": { "type": "number" },
                "min_availability_pct": { "type": "number" }
            }, "required": ["connection_id"]
        })),
        def("connect_sentinel_report", "Generate availability report", json!({
            "type": "object", "properties": {
                "period": { "type": "string", "enum": ["24h","7d","30d"], "default": "7d" }
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
        "connect_tls_inspect" => Some(exec_tls_inspect(args).await),
        "connect_tls_expiry" => Some(exec_tls_expiry(args).await),
        "connect_tls_grade" => Some(exec_tls_grade(args).await),
        "connect_sentinel_probe" => Some(exec_probe(args).await),
        "connect_sentinel_status" => Some(exec_sentinel_status(session).await),
        n if n.starts_with("connect_tls_") || n.starts_with("connect_sentinel_") => {
            Some(Ok(ToolCallResult::json(&json!({ "tool": name, "status": "stub" }))))
        }
        _ => None,
    }
}

async fn exec_tls_inspect(args: Value) -> McpResult<ToolCallResult> {
    let host = args.get("host").and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("Missing host".into()))?;
    let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(443) as u16;
    let info = agentic_connect::engine::tls_inspect::inspect_tls(host, port, 5000).await;
    Ok(ToolCallResult::json(&info))
}

async fn exec_tls_expiry(args: Value) -> McpResult<ToolCallResult> {
    let hosts = args.get("hosts").and_then(|v| v.as_array())
        .ok_or_else(|| McpError::InvalidParams("Missing hosts array".into()))?;

    let mut results = Vec::new();
    for h in hosts {
        let host_str = h.as_str().unwrap_or("");
        let (host, port) = if let Some((h, p)) = host_str.rsplit_once(':') {
            (h, p.parse::<u16>().unwrap_or(443))
        } else {
            (host_str, 443u16)
        };
        let info = agentic_connect::engine::tls_inspect::inspect_tls(host, port, 5000).await;
        results.push(json!({
            "host": host, "port": port, "reachable": info.reachable,
            "tls_available": info.tls_available, "grade": info.grade,
            "issues": info.issues,
        }));
    }
    Ok(ToolCallResult::json(&json!({ "results": results, "checked": results.len() })))
}

async fn exec_tls_grade(args: Value) -> McpResult<ToolCallResult> {
    let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("");
    let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(443) as u16;
    let info = agentic_connect::engine::tls_inspect::inspect_tls(host, port, 5000).await;
    Ok(ToolCallResult::json(&json!({
        "host": host, "port": port, "grade": info.grade,
        "tls_available": info.tls_available, "issues": info.issues,
    })))
}

async fn exec_probe(args: Value) -> McpResult<ToolCallResult> {
    let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
    let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(5000);

    // Parse target into host:port
    let (host, port) = if target.contains("://") {
        if let Ok(url) = url::Url::parse(target) {
            let h = url.host_str().unwrap_or("localhost").to_string();
            let p = url.port().unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
            (h, p)
        } else {
            (target.to_string(), 80u16)
        }
    } else if let Some((h, p)) = target.rsplit_once(':') {
        (h.to_string(), p.parse().unwrap_or(80))
    } else {
        (target.to_string(), 80u16)
    };

    let result = agentic_connect::engine::protocol_detect::probe_host(&host, port, timeout_ms).await;
    Ok(ToolCallResult::json(&result))
}

async fn exec_sentinel_status(session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let conns = sess.store().list_connections(None).map_err(|e| McpError::Internal(e.to_string()))?;
    let stats = sess.store().stats().map_err(|e| McpError::Internal(e.to_string()))?;

    let summary: Vec<_> = conns.iter().map(|c| {
        json!({
            "name": c.name, "protocol": format!("{:?}", c.protocol),
            "host": c.host, "last_used": c.last_used,
        })
    }).collect();

    Ok(ToolCallResult::json(&json!({
        "total_connections": stats.connection_count,
        "total_profiles": stats.profile_count,
        "total_health_checks": stats.health_check_count,
        "connections": summary,
    })))
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
