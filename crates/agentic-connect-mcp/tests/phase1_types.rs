//! Phase 1: Type foundation tests — MCP types, serialization, error codes.

use agentic_connect_mcp::types::*;
use serde_json::{json, Value};

#[test]
fn test_tool_definition_required_fields() {
    let def = ToolDefinition {
        name: "test".into(),
        description: Some("Test tool".into()),
        input_schema: json!({"type": "object", "properties": {}, "required": ["x"]}),
    };
    let v = serde_json::to_value(&def).unwrap();
    assert!(v["inputSchema"]["required"].is_array());
}

#[test]
fn test_tool_definition_optional_description() {
    let def = ToolDefinition { name: "x".into(), description: None, input_schema: json!({}) };
    let v = serde_json::to_string(&def).unwrap();
    assert!(!v.contains("description"));
}

#[test]
fn test_content_block_text_serialization() {
    let block = ContentBlock::Text { text: "hello".into() };
    let v = serde_json::to_value(&block).unwrap();
    assert_eq!(v["type"], "text");
    assert_eq!(v["text"], "hello");
}

#[test]
fn test_tool_call_result_non_error_omits_flag() {
    let r = ToolCallResult::text("ok");
    let v = serde_json::to_value(&r).unwrap();
    assert!(v.get("isError").is_none());
}

#[test]
fn test_tool_call_result_error_includes_flag() {
    let r = ToolCallResult::error("fail");
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["isError"], true);
}

#[test]
fn test_tool_call_result_json_pretty() {
    let data = json!({"a": 1, "b": [2,3]});
    let r = ToolCallResult::json(&data);
    let v = serde_json::to_value(&r).unwrap();
    let text = v["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("\"a\": 1"));
}

#[test]
fn test_mcp_error_display() {
    let e = McpError::InvalidParams("bad field".into());
    assert_eq!(format!("{}", e), "Invalid parameters: bad field");
    let e2 = McpError::ToolNotFound("foo".into());
    assert_eq!(format!("{}", e2), "Tool not found: foo");
}

#[test]
fn test_jsonrpc_request_parsing() {
    let raw = r#"{"jsonrpc":"2.0","id":42,"method":"tools/call","params":{"name":"test"}}"#;
    let req: JsonRpcRequest = serde_json::from_str(raw).unwrap();
    assert_eq!(req.method, "tools/call");
    assert_eq!(req.id, Some(json!(42)));
}

#[test]
fn test_jsonrpc_request_null_id() {
    let raw = r#"{"jsonrpc":"2.0","id":null,"method":"test"}"#;
    let req: JsonRpcRequest = serde_json::from_str(raw).unwrap();
    // JSON null can be Some(Null) or None depending on serde config
    // Either is valid per JSON-RPC spec
    assert!(req.id.is_none() || req.id == Some(Value::Null));
}

#[test]
fn test_jsonrpc_request_no_params() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    let req: JsonRpcRequest = serde_json::from_str(raw).unwrap();
    assert!(req.params.is_none());
}

#[test]
fn test_jsonrpc_response_success_serialization() {
    let r = JsonRpcResponse::success(Some(json!(1)), json!({"result": "ok"}));
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["jsonrpc"], "2.0");
    assert_eq!(v["id"], 1);
    assert!(v.get("error").is_none());
}

#[test]
fn test_jsonrpc_response_error_serialization() {
    let r = JsonRpcResponse::error(Some(json!(5)), -32803, "Not found".into());
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["error"]["code"], -32803);
    assert!(v.get("result").is_none());
}

#[test]
fn test_tool_not_found_code_value() {
    assert_eq!(TOOL_NOT_FOUND_CODE, -32803);
    assert_ne!(TOOL_NOT_FOUND_CODE, -32601); // Not METHOD_NOT_FOUND
    assert_ne!(TOOL_NOT_FOUND_CODE, -32602); // Not INVALID_PARAMS
}
