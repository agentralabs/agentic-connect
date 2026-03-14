# AgenticConnect Quickstart

## Install

```bash
curl -fsSL https://agentralabs.tech/install/connect | bash
```

## Verify Installation

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | agentic-connect-mcp | head -c 200
```

## First API Call

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"connect_api_call","arguments":{"url":"https://httpbin.org/get","method":"GET"}}}' | agentic-connect-mcp
```

## Detect a Protocol

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"connect_protocol_detect","arguments":{"target":"https://api.github.com"}}}' | agentic-connect-mcp
```

## Check TLS

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"connect_tls_grade","arguments":{"host":"github.com"}}}' | agentic-connect-mcp
```

## Connect to a Database

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"connect_db_connect","arguments":{"url":"sqlite:///tmp/test.db","name":"mydb"}}}' | agentic-connect-mcp
```
