# AgenticConnect Architecture

## Workspace Structure

```
agentic-connect/
├── crates/
│   ├── agentic-connect/          Core library (types + engines)
│   ├── agentic-connect-mcp/      MCP server (JSON-RPC + tools)
│   ├── agentic-connect-cli/      CLI binary (acnx)
│   └── agentic-connect-ffi/      FFI bindings (C-compatible)
```

## Engine Layer

| Engine | Responsibility |
|--------|---------------|
| ConnectionStore | SQLite persistence for connections, profiles, health checks |
| RetryEngine | Circuit breakers, failure classification, rate limit tracking |
| CredentialVault | AES-256-GCM encrypted credential storage |
| DbConnection | SQLite query execution and schema discovery |
| HttpClient | HTTP requests with timing and rate limit extraction |
| ProtocolDetect | Banner + port-based protocol identification |
| TlsInspect | TLS certificate checking and quality grading |
| WebhookEngine | HMAC-SHA256 signing and verification |

## Data Flow

```
LLM (Claude/GPT) → MCP JSON-RPC → Tool Registry → Engine → External System
                                                  ↓
                                           ConnectionStore (SQLite)
                                           RetryEngine (circuit breakers)
                                           CredentialVault (encrypted)
```

## MCP Protocol

- Transport: stdio (line-delimited JSON-RPC 2.0)
- Methods: `initialize`, `tools/list`, `tools/call`
- Error codes: -32803 (tool not found), -32602 (invalid params), -32603 (internal)
- 123 tools across 24 inventions
