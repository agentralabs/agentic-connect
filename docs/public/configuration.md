# AgenticConnect Configuration

## Environment Variables

| Variable | Default | Allowed Values | Effect |
|----------|---------|---------------|--------|
| `ACNX_DATA_DIR` | `~/.local/share/agentic-connect` | Any path | Data directory for SQLite and profiles |
| `ACNX_LOG_LEVEL` | `warn` | trace, debug, info, warn, error | Logging verbosity |
| `RUST_LOG` | — | Module-level filters | Alternative log control |

## MCP Server Configuration

### Claude Desktop

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

### Cursor / Windsurf

Same format in `~/.cursor/mcp.json` or `~/.windsurf/mcp.json`.

## Runtime Modes

| Mode | Description |
|------|-------------|
| stdio (default) | MCP JSON-RPC over stdin/stdout |
| serve --data PATH | Custom data directory |

## Feature Flags (Build-Time)

| Feature | Default | Effect |
|---------|---------|--------|
| `http` | Yes | Enable HTTP client (reqwest) |
| `format` | Yes | Enable binary file format (LZ4 + BLAKE3) |
| `browser` | No | Enable headless browser tools |
| `ssh` | No | Enable SSH remote execution |
| `email` | No | Enable SMTP/IMAP email tools |
| `telephony` | No | Enable Twilio/Vonage phone tools |
| `full` | No | Enable all features |
