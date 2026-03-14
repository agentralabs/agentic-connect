//! MCP tool implementations — 127 tools across 24 capabilities.

pub mod protocol;      // Capability 1: Protocol Omniscience (4 tools)
pub mod auth;          // Capability 2: Adaptive Authentication (5 tools)
pub mod soul;          // Capability 3: Connection Soul (5 tools)
pub mod retry;         // Capability 4: Intelligent Retry Fabric (5 tools)
pub mod browse;        // Capabilities 5-7: Browser, Scraping, Forms (16 tools)
pub mod api;           // Capabilities 8-9: API Comprehension, GraphQL (11 tools)
pub mod infra;         // Capabilities 10-12: Remote, Mesh, Container (16 tools)
pub mod comms;         // Capabilities 13-15: Email, Telephony, Webhooks (16 tools)
pub mod data;          // Capabilities 16-18: Database, Queue, Cloud (16 tools)
pub mod security;      // Capabilities 19-20: TLS, Sentinel (11 tools)
pub mod intelligence;  // Capabilities 21-24: Prophecy, Evolution, Dream, Collective (17 tools)
pub mod registry;

pub use registry::ToolRegistry;
