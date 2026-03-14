# Changelog

All notable changes to AgenticConnect will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-14

### Added
- Core engine with 8 modules: ConnectionStore, RetryEngine, CredentialVault, DbConnection, HttpClient, ProtocolDetect, TlsInspect, WebhookEngine
- 123 MCP tools across 24 inventions
- Protocol detection for 18 protocol families (HTTP, SSH, gRPC, SMTP, etc.)
- Adaptive authentication with 8 methods (Basic, Bearer, OAuth2, SSH Key, mTLS, etc.)
- Connection Souls — persistent profiles that accumulate knowledge about remote systems
- Intelligent Retry Fabric — failure classification with circuit breakers
- Encrypted Credential Vault — AES-256-GCM with PBKDF2 key derivation
- Database intelligence — SQLite queries, schema discovery, EXPLAIN QUERY PLAN
- Webhook engine — HMAC-SHA256 sign/verify
- TLS inspection — certificate checking and grading
- MCP server (JSON-RPC over stdio) with error code -32803 for unknown tools
- CLI binary (acnx) for connection management
- FFI bindings (C-compatible cdylib + staticlib)
- 164+ tests including edge cases and stress tests
- 4 SVG assets in Agentra design system
- Research paper (LaTeX, two-column)
- Install script with 3 profiles (desktop, terminal, server)
- 5 CI workflows (ci, release, canonical, install, hardening)
- Full Canonical Sister Kit compliance
