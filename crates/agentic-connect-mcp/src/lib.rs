//! AgenticConnect MCP Server — universal LLM access to external systems.

pub mod session;
pub mod tools;
pub mod types;

pub use session::SessionManager;
pub use tools::ToolRegistry;
