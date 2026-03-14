//! MCP protocol types for AgenticConnect.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP tool definition sent during tools/list.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<ContentBlock>,
    #[serde(rename = "isError", skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

/// Content block in a tool result.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

impl ToolCallResult {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: content.into(),
            }],
            is_error: false,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: message.into(),
            }],
            is_error: true,
        }
    }

    pub fn json(value: &impl Serialize) -> Self {
        let text = serde_json::to_string_pretty(value).unwrap_or_else(|e| e.to_string());
        Self::text(text)
    }
}

/// MCP error types.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// TOOL_NOT_FOUND JSON-RPC error code (per MCP Quality Standard).
pub const TOOL_NOT_FOUND_CODE: i64 = -32803;

pub type McpResult<T> = Result<T, McpError>;

/// JSON-RPC request.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC response.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}
