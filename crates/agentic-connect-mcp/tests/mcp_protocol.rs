//! MCP protocol compliance tests — JSON-RPC parsing, error codes, tool dispatch.

use serde_json::{json, Value};

// Test helpers — simulate what main.rs does without stdio
fn parse_request(line: &str) -> Result<Value, String> {
    serde_json::from_str(line).map_err(|e| format!("Parse error: {}", e))
}

fn make_request(method: &str, params: Option<Value>) -> String {
    let mut req = json!({ "jsonrpc": "2.0", "id": 1, "method": method });
    if let Some(p) = params {
        req["params"] = p;
    }
    serde_json::to_string(&req).unwrap()
}

#[test]
fn test_parse_valid_request() {
    let line = make_request("tools/list", None);
    let parsed = parse_request(&line).unwrap();
    assert_eq!(parsed["method"], "tools/list");
    assert_eq!(parsed["jsonrpc"], "2.0");
}

#[test]
fn test_parse_invalid_json() {
    let result = parse_request("not json at all");
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_line() {
    let result = parse_request("");
    assert!(result.is_err());
}

#[test]
fn test_initialize_request_format() {
    let req = make_request("initialize", Some(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": { "name": "test", "version": "1.0" }
    })));
    let parsed: Value = serde_json::from_str(&req).unwrap();
    assert_eq!(parsed["method"], "initialize");
    assert!(parsed["params"]["protocolVersion"].is_string());
}

#[test]
fn test_tools_list_request_format() {
    let req = make_request("tools/list", None);
    let parsed: Value = serde_json::from_str(&req).unwrap();
    assert_eq!(parsed["method"], "tools/list");
}

#[test]
fn test_tools_call_request_format() {
    let req = make_request("tools/call", Some(json!({
        "name": "connect_protocol_list",
        "arguments": {}
    })));
    let parsed: Value = serde_json::from_str(&req).unwrap();
    assert_eq!(parsed["params"]["name"], "connect_protocol_list");
}

#[test]
fn test_tool_call_missing_name() {
    let req = make_request("tools/call", Some(json!({})));
    let parsed: Value = serde_json::from_str(&req).unwrap();
    let name = parsed["params"].get("name").and_then(|v| v.as_str()).unwrap_or("");
    assert!(name.is_empty());
}

// Test the MCP types directly
use agentic_connect_mcp::types::*;

#[test]
fn test_tool_call_result_text() {
    let result = ToolCallResult::text("hello");
    assert!(!result.is_error);
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["content"][0]["text"], "hello");
}

#[test]
fn test_tool_call_result_error() {
    let result = ToolCallResult::error("something failed");
    assert!(result.is_error);
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["isError"], true);
}

#[test]
fn test_tool_call_result_json() {
    let result = ToolCallResult::json(&json!({"key": "value"}));
    assert!(!result.is_error);
    let json = serde_json::to_value(&result).unwrap();
    let text = json["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("key"));
}

#[test]
fn test_jsonrpc_response_success() {
    let resp = JsonRpcResponse::success(Some(json!(1)), json!({"tools": []}));
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 1);
    assert!(json["result"].is_object());
    assert!(json.get("error").is_none());
}

#[test]
fn test_jsonrpc_response_error() {
    let resp = JsonRpcResponse::error(Some(json!(1)), -32803, "Unknown tool: foo".into());
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["error"]["code"], -32803);
    assert!(json["error"]["message"].as_str().unwrap().contains("foo"));
}

#[test]
fn test_tool_not_found_code() {
    assert_eq!(TOOL_NOT_FOUND_CODE, -32803);
}

#[test]
fn test_tool_definition_serialization() {
    let def = ToolDefinition {
        name: "test_tool".into(),
        description: Some("Test tool description".into()),
        input_schema: json!({"type": "object", "properties": {}}),
    };
    let json = serde_json::to_value(&def).unwrap();
    assert_eq!(json["name"], "test_tool");
    assert_eq!(json["inputSchema"]["type"], "object");
}
