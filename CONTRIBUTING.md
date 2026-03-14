# Contributing to AgenticConnect

Thank you for your interest in contributing to AgenticConnect!

## Getting Started

```bash
git clone https://github.com/agentralabs/agentic-connect.git
cd agentic-connect
cargo build -j 1
cargo test -j 1
```

## Development Guidelines

- All `.rs` files must stay under 400 lines
- Run `cargo clippy` before submitting
- Run `cargo fmt` before submitting
- All new tools need tests in the corresponding `tests/phase*` file
- New engine modules need inline `#[cfg(test)]` tests

## Adding a New Protocol Adapter

1. Add the protocol variant to `types/protocol.rs`
2. Create the adapter in `engine/`
3. Add MCP tools in `tools/`
4. Wire into `tools/registry.rs`
5. Add tests

## MCP Tool Quality Standard

- Tool descriptions: verb-first imperative, no trailing periods
- Unknown tool: error code `-32803`
- Tool execution errors: `isError: true`
- All tools must have JSON Schema input validation

## Running Guardrails

```bash
bash scripts/check-install-commands.sh
bash scripts/check-canonical-sister.sh
bash scripts/check-runtime-hardening.sh
```

## Commit Style

Use conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `test:`

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
