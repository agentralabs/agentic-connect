//! Intelligence tools — Inventions 21-24: Prophecy, Evolution, Dream, Collective.

use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
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
        def("connect_evolve_track", "Track API response schema over time", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "endpoint": { "type": "string" }
            }, "required": ["connection_id", "endpoint"]
        })),
        def("connect_evolve_drift", "Detect schema drift in API responses", json!({
            "type": "object", "properties": { "connection_id": { "type": "string" } },
            "required": ["connection_id"]
        })),
        def("connect_evolve_deprecated", "Identify likely deprecated API fields", json!({
            "type": "object", "properties": { "connection_id": { "type": "string" } },
            "required": ["connection_id"]
        })),
        def("connect_evolve_breaking", "Detect breaking changes in APIs", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "since": { "type": "string" }
            }, "required": ["connection_id"]
        })),
        def("connect_evolve_adapt", "Auto-adapt to detected API changes", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "change_id": { "type": "string" }
            }, "required": ["connection_id"]
        })),
        def("connect_dream_start", "Start idle connection maintenance", json!({
            "type": "object", "properties": {
                "scope": { "type": "string", "enum": ["all","critical","recent"], "default": "critical" }
            }
        })),
        def("connect_dream_insights", "Get discoveries from idle health checks", json!({
            "type": "object", "properties": { "since": { "type": "string" } }
        })),
        def("connect_dream_refresh", "Refresh all connection profiles during idle", json!({
            "type": "object", "properties": {}
        })),
        def("connect_dream_health", "Get proactive health report from dream state", json!({
            "type": "object", "properties": {
                "format": { "type": "string", "enum": ["summary","detailed"], "default": "summary" }
            }
        })),
        def("connect_collective_share", "Share a learned API behavior with community", json!({
            "type": "object", "properties": {
                "api_name": { "type": "string" }, "behavior": { "type": "object" }
            }, "required": ["api_name", "behavior"]
        })),
        def("connect_collective_search", "Search for known API behaviors", json!({
            "type": "object", "properties": {
                "api_name": { "type": "string" }, "query": { "type": "string" }
            }, "required": ["api_name"]
        })),
        def("connect_collective_apply", "Apply community knowledge to a connection", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "knowledge_id": { "type": "string" }
            }, "required": ["connection_id", "knowledge_id"]
        })),
        def("connect_collective_rate", "Rate community knowledge accuracy", json!({
            "type": "object", "properties": {
                "knowledge_id": { "type": "string" }, "accurate": { "type": "boolean" }
            }, "required": ["knowledge_id", "accurate"]
        })),
        def("connect_collective_private", "Verify no private data in shared knowledge", json!({
            "type": "object", "properties": { "knowledge_id": { "type": "string" } },
            "required": ["knowledge_id"]
        })),
    ]
}

pub async fn try_execute(
    name: &str,
    args: Value,
    session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    match name {
        "connect_prophecy_predict" => Some(exec_predict(args, session).await),
        "connect_prophecy_trends" => Some(exec_trends(session).await),
        "connect_prophecy_recommend" => Some(exec_recommend(session).await),
        "connect_prophecy_simulate" => Some(exec_simulate(args)),
        "connect_dream_health" => Some(exec_dream_health(session).await),
        "connect_dream_refresh" => Some(exec_dream_refresh(session).await),
        "connect_evolve_drift" => Some(exec_drift(args, session).await),
        n if n.starts_with("connect_prophecy_") || n.starts_with("connect_evolve_")
            || n.starts_with("connect_dream_") || n.starts_with("connect_collective_") =>
        {
            Some(Ok(ToolCallResult::json(&json!({
                "tool": name, "status": "requires_history_data",
                "hint": "Accumulate connection data first, then intelligence features activate"
            }))))
        }
        _ => None,
    }
}

async fn exec_predict(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let conns = sess.store().list_connections(None).map_err(|e| crate::types::McpError::Internal(e.to_string()))?;
    let mut predictions = Vec::new();
    for conn in &conns {
        if let Ok(Some(profile)) = sess.store().get_profile(&conn.id) {
            if profile.baseline.error_rate > 0.1 {
                predictions.push(json!({"connection": conn.name, "risk": "high_error_rate",
                    "severity": "warning", "detail": format!("Error rate {:.1}%", profile.baseline.error_rate * 100.0)}));
            }
            if profile.baseline.avg_latency_ms > 1000.0 {
                predictions.push(json!({"connection": conn.name, "risk": "high_latency",
                    "severity": "warning", "detail": format!("Avg latency {:.0}ms", profile.baseline.avg_latency_ms)}));
            }
            if profile.error_history.len() > 50 {
                predictions.push(json!({"connection": conn.name, "risk": "frequent_errors",
                    "severity": "critical", "detail": format!("{} errors recorded", profile.error_history.len())}));
            }
        }
        if !sess.retry().should_allow(&conn.url()) {
            predictions.push(json!({"connection": conn.name, "risk": "circuit_open",
                "severity": "critical", "detail": "Circuit breaker is open — connection failing"}));
        }
    }
    Ok(ToolCallResult::json(&json!({"predictions": predictions, "count": predictions.len()})))
}

async fn exec_trends(session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let failures = sess.retry().recent_failures(50);
    let mut endpoint_counts = std::collections::HashMap::new();
    for f in &failures {
        *endpoint_counts.entry(f.endpoint.clone()).or_insert(0u32) += 1;
    }
    let mut trends: Vec<_> = endpoint_counts.iter().filter(|(_, &c)| c > 2).map(|(ep, &count)| {
        json!({"endpoint": ep, "failure_count": count, "severity": if count > 10 { "critical" } else { "warning" }})
    }).collect();
    trends.sort_by(|a, b| b["failure_count"].as_u64().cmp(&a["failure_count"].as_u64()));
    Ok(ToolCallResult::json(&json!({"trends": trends, "total_failures": failures.len()})))
}

async fn exec_recommend(session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let mut recs = Vec::new();
    let conns = sess.store().list_connections(None).map_err(|e| crate::types::McpError::Internal(e.to_string()))?;
    for conn in &conns {
        if conn.auth.is_none() { recs.push(json!({"connection": conn.name, "recommendation": "Add authentication", "priority": "medium"})); }
        if !sess.retry().should_allow(&conn.url()) { recs.push(json!({"connection": conn.name, "recommendation": "Reset circuit breaker or investigate failures", "priority": "high"})); }
    }
    if conns.is_empty() { recs.push(json!({"recommendation": "No connections configured — add connections to enable intelligence", "priority": "info"})); }
    Ok(ToolCallResult::json(&json!({"recommendations": recs, "count": recs.len()})))
}

fn exec_simulate(args: Value) -> McpResult<ToolCallResult> {
    let scenario = args.get("scenario").and_then(|v| v.as_str()).unwrap_or("unknown");
    let impact = match scenario {
        "dns_failure" => json!({"affected": "all_connections", "severity": "critical", "recovery": "Wait for DNS resolution or switch to IP addresses"}),
        "cert_expiry" => json!({"affected": "tls_connections", "severity": "high", "recovery": "Renew certificates before expiry"}),
        "rate_limit" => json!({"affected": "single_endpoint", "severity": "medium", "recovery": "Backoff and retry after window resets"}),
        "cascade" => json!({"affected": "dependent_services", "severity": "critical", "recovery": "Circuit breakers isolate failures"}),
        "network_partition" => json!({"affected": "remote_connections", "severity": "critical", "recovery": "Failover to backup endpoints"}),
        _ => json!({"affected": "unknown", "severity": "unknown"}),
    };
    Ok(ToolCallResult::json(&json!({"scenario": scenario, "simulated_impact": impact})))
}

async fn exec_dream_health(session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let conns = sess.store().list_connections(None).map_err(|e| crate::types::McpError::Internal(e.to_string()))?;
    let stats = sess.store().stats().map_err(|e| crate::types::McpError::Internal(e.to_string()))?;
    let open_circuits = sess.retry().all_circuits().values().filter(|c| !c.should_allow()).count();
    Ok(ToolCallResult::json(&json!({
        "total_connections": stats.connection_count,
        "total_profiles": stats.profile_count,
        "health_checks": stats.health_check_count,
        "open_circuits": open_circuits,
        "status": if open_circuits == 0 { "healthy" } else { "degraded" },
    })))
}

async fn exec_dream_refresh(session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let sess = session.lock().await;
    let conns = sess.store().list_connections(None).map_err(|e| crate::types::McpError::Internal(e.to_string()))?;
    let mut refreshed = 0;
    for conn in &conns {
        if let Ok(Some(mut profile)) = sess.store().get_profile(&conn.id) {
            profile.last_updated = chrono::Utc::now();
            profile.connection_count += 1;
            let _ = sess.store().save_profile(&profile);
            refreshed += 1;
        }
    }
    Ok(ToolCallResult::json(&json!({"refreshed": refreshed, "total": conns.len()})))
}

async fn exec_drift(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let conn_id = args.get("connection_id").and_then(|v| v.as_str()).unwrap_or("");
    let id = uuid::Uuid::parse_str(conn_id).map_err(|e| crate::types::McpError::Internal(e.to_string()))?;
    let sess = session.lock().await;
    let profile = sess.store().get_profile(&id).map_err(|e| crate::types::McpError::Internal(e.to_string()))?;
    match profile {
        Some(p) => Ok(ToolCallResult::json(&json!({
            "connection_id": conn_id,
            "sample_count": p.baseline.sample_count,
            "avg_latency_ms": p.baseline.avg_latency_ms,
            "error_count": p.error_history.len(),
            "drift_detected": p.error_history.len() > 10,
        }))),
        None => Ok(ToolCallResult::json(&json!({"connection_id": conn_id, "drift_detected": false, "hint": "No profile data yet"}))),
    }
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
