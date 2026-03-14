# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in AgenticConnect, please report it responsibly:

1. **Do NOT** open a public issue
2. Email: security@agentralabs.tech
3. Include: description, reproduction steps, impact assessment
4. Expected response: within 48 hours

## Security Architecture

- **Credential Vault**: AES-256-GCM encryption with PBKDF2 key derivation (100,000 iterations)
- **No plaintext secrets**: Credentials encrypted at rest, never logged
- **MCP input validation**: All tool parameters validated before execution
- **Circuit breakers**: Prevent cascade failures from compromised endpoints
- **HMAC-SHA256**: Webhook signature verification prevents payload tampering

## Scope

Security issues in the following areas are in scope:
- Credential vault encryption/decryption
- MCP protocol handling (injection, overflow)
- Authentication credential leakage
- TLS inspection (certificate validation bypass)
- SQL injection in database tools
