# AgenticConnect

<p align="center">
  <img src="assets/hero-agentic-connect.svg" alt="AgenticConnect" width="900"/>
</p>

**Universal external interface engine for AI agents.**

AgenticConnect handles all external communication — HTTP, WebSocket, SSH, databases, message queues, email, and every protocol that exists or will exist. One engine, one MCP interface, every external system.

<p align="center">
  <img src="assets/terminal-agentic-connect.svg" alt="Terminal" width="700"/>
</p>

## Install

```bash
# One-liner install (downloads binary + auto-configures MCP clients)
curl -fsSL https://agentralabs.tech/install/connect | bash

# Profile-specific installs
curl -fsSL https://agentralabs.tech/install/connect/desktop | bash
curl -fsSL https://agentralabs.tech/install/connect/terminal | bash
curl -fsSL https://agentralabs.tech/install/connect/server | bash
```

**Standalone guarantee:** AgenticConnect works independently — no other Agentra sister required.

### Build from Source

```bash
git clone https://github.com/agentralabs/agentic-connect.git
cd agentic-connect
cargo build --release -j 1
cp target/release/agentic-connect-mcp ~/.local/bin/
```

## Quickstart

```bash
# Test the MCP server
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | agentic-connect-mcp

# List all 123 tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | agentic-connect-mcp
```

## Architecture

<p align="center">
  <img src="assets/architecture-agentic-connect.svg" alt="Architecture" width="900"/>
</p>

## How It Works

AgenticConnect exposes 123 MCP tools across 24 inventions:

| Category | Inventions | Tools | What It Does |
|----------|-----------|-------|-------------|
| **Protocol** | 1-4 | 19 | Detect, connect, retry any protocol |
| **Browser** | 5-7 | 16 | Navigate, scrape, fill forms semantically |
| **API** | 8-9 | 11 | HTTP calls, GraphQL, API discovery |
| **Infrastructure** | 10-12 | 16 | SSH commands, service mesh, containers |
| **Communication** | 13-15 | 16 | Email, telephony, webhooks |
| **Data Channels** | 16-18 | 16 | Databases, queues, cloud storage |
| **Security** | 19-20 | 11 | TLS inspection, network monitoring |
| **Intelligence** | 21-24 | 18 | Prediction, evolution tracking, collective learning |

## MCP Configuration

### Claude Desktop / Cursor / Windsurf

Add to your MCP client config:

```json
{
  "mcpServers": {
    "agentic-connect": {
      "command": "agentic-connect-mcp",
      "args": []
    }
  }
}
```

### Any MCP Client

AgenticConnect speaks standard MCP over stdio. Any client that supports MCP can use it.

## Benchmarks

<p align="center">
  <img src="assets/benchmark-agentic-connect.svg" alt="Benchmarks" width="800"/>
</p>

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ACNX_DATA_DIR` | `~/.local/share/agentic-connect` | Data directory |
| `ACNX_LOG_LEVEL` | `warn` | Log level (trace, debug, info, warn, error) |
| `RUST_LOG` | — | Alternative log level control |

## License

MIT
