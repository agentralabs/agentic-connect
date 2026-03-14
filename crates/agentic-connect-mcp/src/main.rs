//! AgenticConnect MCP Server — entry point.
//!
//! Speaks MCP (JSON-RPC over stdio) to give LLMs access to all external systems.

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use agentic_connect_mcp::session::{resolve_data_path, SessionManager};
use agentic_connect_mcp::tools::ToolRegistry;
use agentic_connect_mcp::types::{
    JsonRpcRequest, JsonRpcResponse, McpError, TOOL_NOT_FOUND_CODE,
};

#[derive(Parser)]
#[command(name = "agentic-connect-mcp", about = "AgenticConnect MCP Server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start MCP server on stdio (default)
    Serve {
        /// Data directory path
        #[arg(long)]
        data: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(io::stderr)
        .init();

    let cli = Cli::parse();
    let data_path = match &cli.command {
        Some(Command::Serve { data }) => resolve_data_path(data.clone()),
        None => resolve_data_path(None),
    };

    let session = Arc::new(Mutex::new(SessionManager::new(data_path)?));

    tracing::info!("AgenticConnect MCP server starting on stdio");
    run_stdio(session).await
}

/// Run the MCP server on stdio (line-delimited JSON-RPC).
async fn run_stdio(session: Arc<Mutex<SessionManager>>) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = handle_message(&line, &session).await;
        let response_json = serde_json::to_string(&response)?;

        let mut out = stdout.lock();
        writeln!(out, "{}", response_json)?;
        out.flush()?;
    }

    Ok(())
}

async fn handle_message(
    line: &str,
    session: &Arc<Mutex<SessionManager>>,
) -> JsonRpcResponse {
    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
        }
    };

    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => handle_initialize(id, request.params),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(id, request.params, session).await,
        "notifications/initialized" => {
            // Client notification, no response needed for notifications
            // but we still return a response for stdio protocol
            JsonRpcResponse::success(id, json!({}))
        }
        _ => JsonRpcResponse::error(
            id,
            -32601,
            format!("Method not found: {}", request.method),
        ),
    }
}

fn handle_initialize(id: Option<Value>, _params: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": { "listChanged": false }
            },
            "serverInfo": {
                "name": "agentic-connect",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    let tools = ToolRegistry::list_tools();
    let tools_json: Vec<Value> = tools
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })
        })
        .collect();

    JsonRpcResponse::success(id, json!({ "tools": tools_json }))
}

async fn handle_tools_call(
    id: Option<Value>,
    params: Option<Value>,
    session: &Arc<Mutex<SessionManager>>,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(id, -32602, "Missing params".into());
        }
    };

    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if tool_name.is_empty() {
        return JsonRpcResponse::error(id, -32602, "Missing tool name".into());
    }

    let arguments = params.get("arguments").cloned();

    match ToolRegistry::call(tool_name, arguments, session).await {
        Ok(result) => {
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap_or(json!({})))
        }
        Err(McpError::ToolNotFound(msg)) => {
            JsonRpcResponse::error(id, TOOL_NOT_FOUND_CODE, msg)
        }
        Err(McpError::InvalidParams(msg)) => {
            JsonRpcResponse::error(id, -32602, msg)
        }
        Err(McpError::Internal(msg)) => {
            JsonRpcResponse::error(id, -32603, msg)
        }
    }
}
