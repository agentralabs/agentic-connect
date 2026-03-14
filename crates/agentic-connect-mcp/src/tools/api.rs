//! API tools — Inventions 8-9: API Comprehension + GraphQL Depth.

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        def("connect_api_discover", "Discover API endpoints from base URL", json!({
            "type": "object", "properties": {
                "base_url": { "type": "string", "description": "Base URL to probe" },
                "probe_common": { "type": "boolean", "default": true }
            }, "required": ["base_url"]
        })),
        def("connect_api_spec", "Parse and understand API specification", json!({
            "type": "object", "properties": {
                "url": { "type": "string", "description": "URL to OpenAPI/Swagger spec" },
                "format": { "type": "string", "enum": ["openapi3","swagger2","graphql","auto"], "default": "auto" }
            }, "required": ["url"]
        })),
        def("connect_api_call", "Make HTTP API call with auth and retry", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "method": { "type": "string", "enum": ["GET","POST","PUT","PATCH","DELETE","HEAD"], "default": "GET" },
                "headers": { "type": "object", "description": "Request headers" },
                "body": { "type": ["object","string","null"], "description": "Request body (auto JSON-serialized if object)" },
                "timeout_ms": { "type": "integer", "default": 30000 }
            }, "required": ["url"]
        })),
        def("connect_api_profile", "View behavioral profile of an API", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }
            }, "required": ["connection_id"]
        })),
        def("connect_api_mock", "Generate mock server from API profile", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "port": { "type": "integer", "default": 8080 }
            }, "required": ["connection_id"]
        })),
        def("connect_api_test", "Test API endpoint against expectations", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "method": { "type": "string", "default": "GET" },
                "expected_status": { "type": "integer" },
                "expected_body_contains": { "type": "string" },
                "timeout_ms": { "type": "integer", "default": 10000 }
            }, "required": ["url"]
        })),
        // Invention 9: GraphQL
        def("connect_graphql_introspect", "Discover GraphQL schema", json!({
            "type": "object", "properties": {
                "url": { "type": "string" }
            }, "required": ["url"]
        })),
        def("connect_graphql_query", "Execute GraphQL query", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "query": { "type": "string" },
                "variables": { "type": "object" }
            }, "required": ["url", "query"]
        })),
        def("connect_graphql_build", "Build GraphQL query from natural language", json!({
            "type": "object", "properties": {
                "url": { "type": "string" }, "intent": { "type": "string" }
            }, "required": ["url", "intent"]
        })),
        def("connect_graphql_subscribe", "Manage real-time GraphQL subscriptions", json!({
            "type": "object", "properties": {
                "url": { "type": "string" }, "subscription": { "type": "string" },
                "action": { "type": "string", "enum": ["start","stop","list"], "default": "start" }
            }, "required": ["url"]
        })),
        def("connect_graphql_normalize", "Flatten nested GraphQL response data", json!({
            "type": "object", "properties": {
                "data": { "type": "object" }
            }, "required": ["data"]
        })),
    ]
}

pub async fn try_execute(
    name: &str,
    args: Value,
    session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    match name {
        "connect_api_call" => Some(execute_api_call(args, session).await),
        "connect_api_test" => Some(execute_api_test(args, session).await),
        "connect_api_discover" => Some(execute_discover(args).await),
        "connect_graphql_query" => Some(execute_graphql_query(args).await),
        "connect_graphql_normalize" => Some(execute_normalize(args)),
        n if n.starts_with("connect_api_") || n.starts_with("connect_graphql_") => {
            Some(Ok(ToolCallResult::json(&json!({ "tool": name, "status": "stub" }))))
        }
        _ => None,
    }
}

async fn execute_api_call(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P { url: String, method: Option<String>, headers: Option<HashMap<String, String>>,
               body: Option<Value>, timeout_ms: Option<u64> }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let method = p.method.as_deref().unwrap_or("GET");
    let timeout = p.timeout_ms.unwrap_or(30000);

    // Check circuit breaker
    {
        let sess = session.lock().await;
        if !sess.retry().should_allow(&p.url) {
            return Ok(ToolCallResult::json(&json!({
                "error": "circuit_breaker_open",
                "url": p.url,
                "hint": "Too many failures — circuit breaker is open. Use connect_retry_circuit to reset."
            })));
        }
    }

    let body_str = p.body.as_ref().map(|b| {
        if b.is_string() { b.as_str().unwrap_or("").to_string() }
        else { serde_json::to_string(b).unwrap_or_default() }
    });

    let resp = agentic_connect::engine::http_client::http_request(
        &p.url, method, p.headers.as_ref(), body_str.as_deref(), timeout,
    ).await;

    match resp {
        Ok(r) => {
            // Record success + update rate limits
            let mut sess = session.lock().await;
            sess.retry_mut().record_success(&p.url);
            if let Some((limit, remaining, reset)) =
                agentic_connect::engine::http_client::extract_rate_limit(&r.headers)
            {
                sess.retry_mut().update_rate_limit(&p.url, limit, remaining, reset);
            }
            // Update connection soul latency
            if let Ok(Some(conn)) = sess.store().list_connections(None)
                .map(|cs| cs.into_iter().find(|c| p.url.contains(&c.host)))
            {
                if let Ok(Some(mut profile)) = sess.store().get_profile(&conn.id) {
                    profile.record_latency(r.latency_ms as f64);
                    let _ = sess.store().save_profile(&profile);
                }
            }

            // Truncate body for display
            let display_body = if r.body.len() > 2000 {
                format!("{}... [truncated, {} bytes total]", &r.body[..2000], r.body.len())
            } else {
                r.body.clone()
            };

            Ok(ToolCallResult::json(&json!({
                "status": r.status,
                "latency_ms": r.latency_ms,
                "headers": r.headers,
                "body": display_body,
                "body_bytes": r.body.len(),
            })))
        }
        Err(e) => {
            let mut sess = session.lock().await;
            let class = agentic_connect::RetryEngine::classify_error(&e.to_string());
            sess.retry_mut().record_failure(&p.url, class, &e.to_string(), None);
            Ok(ToolCallResult::error(format!("HTTP request failed: {}", e)))
        }
    }
}

async fn execute_api_test(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let method = args.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
    let expected_status = args.get("expected_status").and_then(|v| v.as_u64());
    let expected_contains = args.get("expected_body_contains").and_then(|v| v.as_str());
    let timeout = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(10000);

    let resp = agentic_connect::engine::http_client::http_request(
        url, method, None, None, timeout,
    ).await;

    match resp {
        Ok(r) => {
            let mut passed = true;
            let mut checks = Vec::new();

            if let Some(expected) = expected_status {
                let ok = r.status as u64 == expected;
                checks.push(json!({ "check": "status", "expected": expected, "actual": r.status, "passed": ok }));
                if !ok { passed = false; }
            }
            if let Some(contains) = expected_contains {
                let ok = r.body.contains(contains);
                checks.push(json!({ "check": "body_contains", "expected": contains, "passed": ok }));
                if !ok { passed = false; }
            }

            Ok(ToolCallResult::json(&json!({
                "url": url, "passed": passed, "status": r.status, "latency_ms": r.latency_ms,
                "checks": checks,
            })))
        }
        Err(e) => Ok(ToolCallResult::json(&json!({
            "url": url, "passed": false, "error": e.to_string()
        }))),
    }
}

async fn execute_discover(args: Value) -> McpResult<ToolCallResult> {
    let base = args.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
    let common_paths = ["/api", "/v1", "/v2", "/graphql", "/health", "/status",
                        "/docs", "/swagger.json", "/openapi.json", "/.well-known"];
    let mut results = Vec::new();

    for path in &common_paths {
        let url = format!("{}{}", base.trim_end_matches('/'), path);
        match agentic_connect::engine::http_client::http_request(&url, "GET", None, None, 5000).await {
            Ok(r) if r.status < 400 => {
                results.push(json!({ "path": path, "status": r.status, "latency_ms": r.latency_ms }));
            }
            _ => {}
        }
    }

    Ok(ToolCallResult::json(&json!({
        "base_url": base, "discovered": results, "count": results.len()
    })))
}

async fn execute_graphql_query(args: Value) -> McpResult<ToolCallResult> {
    let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let variables = args.get("variables").cloned().unwrap_or(json!({}));

    let body = serde_json::to_string(&json!({ "query": query, "variables": variables }))
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    match agentic_connect::engine::http_client::http_request(url, "POST", Some(&headers), Some(&body), 30000).await {
        Ok(r) => {
            let parsed: Value = serde_json::from_str(&r.body).unwrap_or(json!({ "raw": r.body }));
            Ok(ToolCallResult::json(&json!({
                "status": r.status, "latency_ms": r.latency_ms, "data": parsed
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("GraphQL request failed: {}", e))),
    }
}

fn execute_normalize(args: Value) -> McpResult<ToolCallResult> {
    let data = args.get("data").cloned().unwrap_or(json!({}));
    let flat = flatten_json("", &data);
    Ok(ToolCallResult::json(&json!({ "flattened": flat, "field_count": flat.len() })))
}

fn flatten_json(prefix: &str, value: &Value) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                result.extend(flatten_json(&key, v));
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let key = format!("{}[{}]", prefix, i);
                result.extend(flatten_json(&key, v));
            }
        }
        _ => { result.insert(prefix.to_string(), value.clone()); }
    }
    result
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
