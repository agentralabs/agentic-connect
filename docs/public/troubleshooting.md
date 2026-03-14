# Troubleshooting

## Installation Issues

### Binary not found after install
```bash
# Add to PATH
export PATH="$HOME/.local/bin:$PATH"
# Verify
which agentic-connect-mcp
```

### Build from source fails
```bash
# Ensure Rust is installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Clone sibling SDK dependency
git clone https://github.com/agentralabs/agentic-sdk.git ../agentic-sdk
# Build
cargo build --release -j 1
```

## MCP Server Issues

### Server doesn't respond
- Check the binary is in PATH: `which agentic-connect-mcp`
- Test directly: `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | agentic-connect-mcp`
- Check logs: `RUST_LOG=debug agentic-connect-mcp 2>connect.log`

### Tool returns error -32803
This means the tool name is unknown. Run `tools/list` to see available tools.

### Circuit breaker blocking requests
```bash
# Check circuit state via MCP
{"method":"tools/call","params":{"name":"connect_retry_circuit","arguments":{"endpoint":"YOUR_URL","action":"view"}}}

# Reset circuit
{"method":"tools/call","params":{"name":"connect_retry_circuit","arguments":{"endpoint":"YOUR_URL","action":"reset"}}}
```

## Common Errors

### "Connection refused"
The target host is not accepting connections on that port. Verify the host and port are correct.

### "Rate limited"
The target API returned HTTP 429. The retry engine will wait automatically. Check `connect_retry_status` for details.

### "Circuit breaker open"
Too many consecutive failures to this endpoint. The circuit breaker protects against cascade failures. Reset with `connect_retry_circuit action=reset` or wait for the timeout.

## Performance

### Slow schema discovery
Schema discovery queries every table. For databases with many tables, this is expected. Use the `table` parameter to discover a single table.

### High memory usage
Reduce connection history retention. The error history is capped at 100 per connection and failure history at 500 total.
