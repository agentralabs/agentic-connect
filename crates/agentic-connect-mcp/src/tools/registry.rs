//! Tool registration and dispatch — central MCP tool router for 127 tools.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value;

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

pub struct ToolRegistry;

impl ToolRegistry {
    /// List all available tool definitions (127 tools across 24 inventions).
    pub fn list_tools() -> Vec<ToolDefinition> {
        let mut tools = Vec::with_capacity(130);

        tools.extend(super::protocol::definitions());      // 4
        tools.extend(super::auth::definitions());          // 5
        tools.extend(super::soul::definitions());          // 5
        tools.extend(super::retry::definitions());         // 5
        tools.extend(super::browse::definitions());        // 16
        tools.extend(super::api::definitions());           // 11
        tools.extend(super::infra::definitions());         // 16
        tools.extend(super::comms::definitions());         // 16
        tools.extend(super::data::definitions());          // 16
        tools.extend(super::security::definitions());      // 11
        tools.extend(super::intelligence::definitions());  // 17

        tools
    }

    /// Dispatch a tool call to the appropriate handler.
    pub async fn call(
        name: &str,
        arguments: Option<Value>,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let args = arguments.unwrap_or(Value::Object(serde_json::Map::new()));

        // Try each invention group in order (fast path: first match returns)
        if let Some(r) = super::protocol::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::auth::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::soul::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::retry::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::browse::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::api::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::infra::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::comms::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::data::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::security::try_execute(name, args.clone(), session).await { return r; }
        if let Some(r) = super::intelligence::try_execute(name, args, session).await { return r; }

        Err(McpError::ToolNotFound(format!("Unknown tool: {}", name)))
    }
}
