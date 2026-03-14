//! Protocol tools — Invention 1: Protocol Omniscience.

use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};
use agentic_connect::Protocol;

/// Return all protocol tool definitions.
pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "connect_protocol_detect".into(),
            description: Some("Detect protocol from URL or host:port".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "URL, host:port, or hostname to detect protocol for"
                    }
                },
                "required": ["target"]
            }),
        },
        ToolDefinition {
            name: "connect_protocol_list".into(),
            description: Some("List all supported protocols with capabilities".into()),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        ToolDefinition {
            name: "connect_protocol_test".into(),
            description: Some("Test if a protocol endpoint is reachable".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target": { "type": "string", "description": "URL or host:port to test" },
                    "timeout_ms": { "type": "integer", "default": 5000, "description": "Timeout in milliseconds" }
                },
                "required": ["target"]
            }),
        },
        ToolDefinition {
            name: "connect_protocol_caps".into(),
            description: Some("Get capabilities of a specific protocol".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "protocol": {
                        "type": "string",
                        "enum": ["http","https","websocket","wss","grpc","ssh","ftp","sftp",
                                 "smtp","imap","dns","mqtt","amqp","redis","postgres","mysql","tcp","udp"],
                        "description": "Protocol to get capabilities for"
                    }
                },
                "required": ["protocol"]
            }),
        },
    ]
}

/// Try to execute a protocol tool. Returns None if name doesn't match.
pub async fn try_execute(
    name: &str,
    args: Value,
    _session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    match name {
        "connect_protocol_detect" => Some(execute_detect(args).await),
        "connect_protocol_list" => Some(execute_list().await),
        "connect_protocol_test" => Some(execute_test(args).await),
        "connect_protocol_caps" => Some(execute_caps(args).await),
        _ => None,
    }
}

async fn execute_detect(args: Value) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct Params { target: String }
    let params: Params = serde_json::from_value(args)
        .map_err(|e| McpError::InvalidParams(e.to_string()))?;

    // Try URL parsing first
    if let Ok(url) = url::Url::parse(&params.target) {
        if let Some(proto) = Protocol::from_scheme(url.scheme()) {
            return Ok(ToolCallResult::json(&json!({
                "detected": true,
                "protocol": proto.name(),
                "host": url.host_str(),
                "port": url.port().or_else(|| proto.default_port()),
                "capabilities": proto.capabilities(),
            })));
        }
    }

    // Try host:port pattern
    if let Some((host, port_str)) = params.target.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            let proto = detect_by_port(port);
            return Ok(ToolCallResult::json(&json!({
                "detected": proto.is_some(),
                "protocol": proto.map(|p| p.name()),
                "host": host,
                "port": port,
                "method": "port_detection",
            })));
        }
    }

    Ok(ToolCallResult::json(&json!({
        "detected": false,
        "target": params.target,
        "hint": "Provide a URL with scheme (e.g., https://...) or host:port"
    })))
}

async fn execute_list() -> McpResult<ToolCallResult> {
    let protocols = [
        Protocol::Http, Protocol::Https, Protocol::WebSocket, Protocol::Wss,
        Protocol::Grpc, Protocol::Ssh, Protocol::Ftp, Protocol::Sftp,
        Protocol::Smtp, Protocol::Imap, Protocol::Dns, Protocol::Mqtt,
        Protocol::Amqp, Protocol::Redis, Protocol::Postgres, Protocol::Mysql,
        Protocol::Tcp, Protocol::Udp,
    ];
    let list: Vec<_> = protocols.iter().map(|p| json!({
        "name": p.name(),
        "default_port": p.default_port(),
        "supports_tls": p.supports_tls(),
        "capabilities": p.capabilities(),
    })).collect();
    Ok(ToolCallResult::json(&json!({ "protocols": list, "count": list.len() })))
}

async fn execute_test(args: Value) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct Params {
        target: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    }
    fn default_timeout() -> u64 { 5000 }

    let params: Params = serde_json::from_value(args)
        .map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(params.timeout_ms);

    // Try TCP connection as basic reachability test
    let result = if let Ok(url) = url::Url::parse(&params.target) {
        let host = url.host_str().unwrap_or("localhost");
        let proto = Protocol::from_scheme(url.scheme());
        let port = url.port().or_else(|| proto.and_then(|p| p.default_port())).unwrap_or(80);
        let addr = format!("{}:{}", host, port);
        match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await {
            Ok(Ok(_)) => json!({ "reachable": true, "latency_ms": start.elapsed().as_millis() }),
            Ok(Err(e)) => json!({ "reachable": false, "error": e.to_string() }),
            Err(_) => json!({ "reachable": false, "error": "timeout" }),
        }
    } else {
        json!({ "reachable": false, "error": "invalid URL" })
    };

    Ok(ToolCallResult::json(&result))
}

async fn execute_caps(args: Value) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct Params { protocol: String }
    let params: Params = serde_json::from_value(args)
        .map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let proto: Protocol = serde_json::from_value(json!(params.protocol))
        .map_err(|_| McpError::InvalidParams(format!("Unknown protocol: {}", params.protocol)))?;

    Ok(ToolCallResult::json(&json!({
        "protocol": proto.name(),
        "default_port": proto.default_port(),
        "capabilities": proto.capabilities(),
    })))
}

fn detect_by_port(port: u16) -> Option<Protocol> {
    match port {
        80 => Some(Protocol::Http),
        443 => Some(Protocol::Https),
        22 => Some(Protocol::Ssh),
        21 => Some(Protocol::Ftp),
        25 | 587 => Some(Protocol::Smtp),
        993 => Some(Protocol::Imap),
        53 => Some(Protocol::Dns),
        1883 => Some(Protocol::Mqtt),
        5672 => Some(Protocol::Amqp),
        6379 => Some(Protocol::Redis),
        5432 => Some(Protocol::Postgres),
        3306 => Some(Protocol::Mysql),
        _ => None,
    }
}
