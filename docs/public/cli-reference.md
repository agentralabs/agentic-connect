# AgenticConnect CLI Reference

## Binary: `acnx`

### Global Options

```
acnx [COMMAND]
```

### Commands

#### `acnx ping <target>`
Test connectivity to a target URL or host:port.

```bash
acnx ping https://api.github.com
acnx ping server.example.com:22
```

#### `acnx list`
List all configured connections.

```bash
acnx list
```

#### `acnx version`
Show version information.

```bash
acnx version
```

## MCP Server Binary: `agentic-connect-mcp`

### Commands

#### `agentic-connect-mcp` (default)
Start MCP server on stdio.

#### `agentic-connect-mcp serve --data /path`
Start with custom data directory.

### Quick Test

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | agentic-connect-mcp
```
