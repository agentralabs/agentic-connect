# AgenticConnect Concepts

## Connection

A configured endpoint to an external system. Has a protocol, host, port, optional auth, and tags. Connections are stored in SQLite and persist across sessions.

## Connection Soul (Capability 3)

Every connection accumulates knowledge — OS fingerprint, performance baseline, error history. The "soul" learns from every interaction so future connections are smarter.

## Protocol Omniscience (Capability 1)

Give AgenticConnect a URL and it auto-detects the protocol, selects the right handler, and connects. Supports 18 protocols: HTTP, HTTPS, WebSocket, gRPC, SSH, FTP, SFTP, SMTP, IMAP, DNS, MQTT, AMQP, Redis, PostgreSQL, MySQL, TCP, UDP.

## Intelligent Retry (Capability 4)

Failures are classified — transient (503), permanent (404), rate-limited (429), auth (401). Each class gets the right retry strategy. Circuit breakers prevent cascade failures.

## Credential Vault (Capability 2)

Credentials are stored encrypted (AES-256-GCM with PBKDF2 key derivation). Supports Basic, Bearer, API Key, OAuth2, SSH Key, mTLS. Auto-refreshes expiring tokens.

## MCP Protocol

AgenticConnect speaks MCP (Model Context Protocol) over stdio. Any LLM that supports MCP can use all 123 tools through a single server.
