//! MCP tool implementations — 127 tools across 24 inventions.

pub mod protocol;      // Invention 1: Protocol Omniscience (4 tools)
pub mod auth;          // Invention 2: Adaptive Authentication (5 tools)
pub mod soul;          // Invention 3: Connection Soul (5 tools)
pub mod retry;         // Invention 4: Intelligent Retry Fabric (5 tools)
pub mod browse;        // Inventions 5-7: Browser, Scraping, Forms (16 tools)
pub mod api;           // Inventions 8-9: API Comprehension, GraphQL (11 tools)
pub mod infra;         // Inventions 10-12: Remote, Mesh, Container (16 tools)
pub mod comms;         // Inventions 13-15: Email, Telephony, Webhooks (16 tools)
pub mod data;          // Inventions 16-18: Database, Queue, Cloud (16 tools)
pub mod security;      // Inventions 19-20: TLS, Sentinel (11 tools)
pub mod intelligence;  // Inventions 21-24: Prophecy, Evolution, Dream, Collective (17 tools)
pub mod registry;

pub use registry::ToolRegistry;
