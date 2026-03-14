//! Data channel tools — Inventions 16-18: Database, Queue, Cloud.

use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Invention 16: Database Connection Intelligence
        def("connect_db_connect", "Connect to database with auto-type detection", json!({
            "type": "object", "properties": {
                "url": { "type": "string", "description": "Connection string (sqlite:// path)" },
                "name": { "type": "string", "description": "Connection name for reference" }
            }, "required": ["url", "name"]
        })),
        def("connect_db_query", "Execute SQL query with result formatting", json!({
            "type": "object", "properties": {
                "name": { "type": "string", "description": "Database connection name" },
                "query": { "type": "string", "description": "SQL query to execute" },
                "limit": { "type": "integer", "default": 100, "description": "Max rows to return" }
            }, "required": ["name", "query"]
        })),
        def("connect_db_schema", "Discover database schema", json!({
            "type": "object", "properties": {
                "name": { "type": "string", "description": "Database connection name" },
                "table": { "type": "string", "description": "Specific table (omit for all)" }
            }, "required": ["name"]
        })),
        def("connect_db_health", "Check database health and connection status", json!({
            "type": "object", "properties": {
                "name": { "type": "string" }
            }, "required": ["name"]
        })),
        def("connect_db_optimize", "Suggest query optimizations", json!({
            "type": "object", "properties": {
                "name": { "type": "string" }, "query": { "type": "string" }
            }, "required": ["name", "query"]
        })),
        def("connect_db_migrate", "Detect schema changes", json!({
            "type": "object", "properties": {
                "name": { "type": "string" },
                "action": { "type": "string", "enum": ["detect","plan"], "default": "detect" }
            }, "required": ["name"]
        })),
        // Invention 17: Message Queue Bridge
        def("connect_queue_publish", "Publish message to any queue system", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "topic": { "type": "string" },
                "message": { "type": ["string","object"] }
            }, "required": ["connection_id", "topic", "message"]
        })),
        def("connect_queue_consume", "Consume messages from any queue", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" }, "topic": { "type": "string" },
                "max_messages": { "type": "integer", "default": 10 }
            }, "required": ["connection_id", "topic"]
        })),
        def("connect_queue_status", "View queue depth and throughput", json!({
            "type": "object", "properties": { "connection_id": { "type": "string" } },
            "required": ["connection_id"]
        })),
        def("connect_queue_dead_letter", "View or replay dead letter messages", json!({
            "type": "object", "properties": {
                "connection_id": { "type": "string" },
                "action": { "type": "string", "enum": ["list","replay","purge"], "default": "list" }
            }, "required": ["connection_id"]
        })),
        def("connect_queue_configure", "Configure queue connection", json!({
            "type": "object", "properties": {
                "url": { "type": "string" }, "name": { "type": "string" },
                "type": { "type": "string", "enum": ["kafka","rabbitmq","sqs","redis_pubsub","auto"] }
            }, "required": ["url"]
        })),
        // Invention 18: Cloud Fabric
        def("connect_cloud_storage", "Object storage operations on any cloud", json!({
            "type": "object", "properties": {
                "action": { "type": "string", "enum": ["upload","download","list","delete"] },
                "bucket": { "type": "string" }, "key": { "type": "string" }
            }, "required": ["action", "bucket"]
        })),
        def("connect_cloud_compute", "VM/instance operations", json!({
            "type": "object", "properties": {
                "action": { "type": "string", "enum": ["list","start","stop","status","create"] }
            }, "required": ["action"]
        })),
        def("connect_cloud_detect", "Detect cloud provider and region", json!({
            "type": "object", "properties": {}
        })),
        def("connect_cloud_cost", "Estimate operation cost", json!({
            "type": "object", "properties": {
                "operation": { "type": "string" },
                "provider": { "type": "string", "enum": ["aws","gcp","azure"] }
            }, "required": ["operation"]
        })),
        def("connect_cloud_resource", "List and manage cloud resources", json!({
            "type": "object", "properties": {
                "provider": { "type": "string", "enum": ["aws","gcp","azure","auto"], "default": "auto" }
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
        "connect_db_connect" => Some(exec_db_connect(args, session).await),
        "connect_db_query" => Some(exec_db_query(args, session).await),
        "connect_db_schema" => Some(exec_db_schema(args, session).await),
        "connect_db_health" => Some(exec_db_health(args, session).await),
        "connect_db_optimize" => Some(exec_db_optimize(args, session).await),
        n if n.starts_with("connect_queue_") || n.starts_with("connect_cloud_") || n == "connect_db_migrate" => {
            Some(Ok(ToolCallResult::json(&json!({ "tool": name, "status": "stub" }))))
        }
        _ => None,
    }
}

async fn exec_db_connect(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P { url: String, name: String }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let mut sess = session.lock().await;
    sess.open_db(&p.name, &p.url).map_err(|e| McpError::Internal(e.to_string()))?;
    let db = sess.get_db(&p.name).unwrap();
    let schema = db.discover_schema().map_err(|e| McpError::Internal(e.to_string()))?;
    Ok(ToolCallResult::json(&json!({
        "connected": true, "name": p.name, "db_type": format!("{:?}", db.db_type()),
        "tables": schema.len(),
        "schema_summary": schema.iter().map(|t| json!({
            "table": t.name, "columns": t.columns.len(), "rows": t.row_count
        })).collect::<Vec<_>>(),
    })))
}

async fn exec_db_query(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P { name: String, query: String, limit: Option<usize> }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let sess = session.lock().await;
    let db = sess.get_db(&p.name).ok_or_else(|| McpError::InvalidParams(format!("No database connection named '{}'", p.name)))?;

    // Add LIMIT if not present and it's a SELECT
    let sql = if p.query.trim().to_uppercase().starts_with("SELECT") && !p.query.to_uppercase().contains("LIMIT") {
        format!("{} LIMIT {}", p.query.trim_end_matches(';'), p.limit.unwrap_or(100))
    } else {
        p.query.clone()
    };

    let result = db.query(&sql).map_err(|e| McpError::Internal(e.to_string()))?;
    Ok(ToolCallResult::json(&json!({
        "columns": result.columns, "rows": result.rows, "row_count": result.row_count,
    })))
}

async fn exec_db_schema(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let table_filter = args.get("table").and_then(|v| v.as_str());
    let sess = session.lock().await;
    let db = sess.get_db(name).ok_or_else(|| McpError::InvalidParams(format!("No DB '{}'", name)))?;

    if let Some(tbl) = table_filter {
        let cols = db.table_columns(tbl).map_err(|e| McpError::Internal(e.to_string()))?;
        Ok(ToolCallResult::json(&json!({ "table": tbl, "columns": cols })))
    } else {
        let schema = db.discover_schema().map_err(|e| McpError::Internal(e.to_string()))?;
        Ok(ToolCallResult::json(&json!({ "tables": schema })))
    }
}

async fn exec_db_health(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let sess = session.lock().await;
    let db = sess.get_db(name).ok_or_else(|| McpError::InvalidParams(format!("No DB '{}'", name)))?;
    let schema = db.discover_schema().map_err(|e| McpError::Internal(e.to_string()))?;
    let total_rows: i64 = schema.iter().filter_map(|t| t.row_count).sum();
    Ok(ToolCallResult::json(&json!({
        "name": name, "healthy": true, "db_type": format!("{:?}", db.db_type()),
        "tables": schema.len(), "total_rows": total_rows, "path": db.path(),
    })))
}

async fn exec_db_optimize(args: Value, session: &Arc<Mutex<SessionManager>>) -> McpResult<ToolCallResult> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let sess = session.lock().await;
    let db = sess.get_db(name).ok_or_else(|| McpError::InvalidParams(format!("No DB '{}'", name)))?;
    let plan = db.query(&format!("EXPLAIN QUERY PLAN {}", query)).map_err(|e| McpError::Internal(e.to_string()))?;
    Ok(ToolCallResult::json(&json!({
        "query": query, "plan": plan.rows,
        "suggestions": analyze_plan(&plan.rows),
    })))
}

fn analyze_plan(rows: &[std::collections::HashMap<String, Value>]) -> Vec<String> {
    let mut suggestions = Vec::new();
    for row in rows {
        let detail = row.get("detail").and_then(|v| v.as_str()).unwrap_or("");
        if detail.contains("SCAN TABLE") && !detail.contains("USING INDEX") {
            let table = detail.strip_prefix("SCAN TABLE ").unwrap_or(detail).split(' ').next().unwrap_or("");
            suggestions.push(format!("Full table scan on '{}' — consider adding an index", table));
        }
    }
    if suggestions.is_empty() {
        suggestions.push("Query plan looks efficient — no obvious improvements".into());
    }
    suggestions
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
