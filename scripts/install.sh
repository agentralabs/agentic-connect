#!/bin/bash
# AgenticConnect — one-liner install script
# Downloads pre-built binary and auto-configures detected MCP clients.
#
# Usage:
#   curl -fsSL https://agentralabs.tech/install/connect | bash
#
# Options:
#   --version=X.Y.Z   Pin a specific version (default: latest)
#   --dir=/path        Override install directory (default: ~/.local/bin)
#   --profile=<name>   Install profile: desktop | terminal | server (default: desktop)

set -euo pipefail

REPO="agentralabs/agentic-connect"
BINARY_NAME="agentic-connect-mcp"
SERVER_KEY="agentic-connect"
INSTALL_DIR="$HOME/.local/bin"
VERSION="latest"
PROFILE="${AGENTRA_INSTALL_PROFILE:-desktop}"

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --version=*) VERSION="${1#*=}"; shift ;;
        --dir=*) INSTALL_DIR="${1#*=}"; shift ;;
        --profile=*) PROFILE="${1#*=}"; shift ;;
        *) echo "Error: unknown option '$1'" >&2; exit 1 ;;
    esac
done

echo "AgenticConnect Installer"
echo "========================"
echo "Profile: $PROFILE"
echo "Install dir: $INSTALL_DIR"

# Detect platform
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Create install directory
mkdir -p "$INSTALL_DIR"

# Try release artifact first
if [ "$VERSION" = "latest" ]; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | grep '"tag_name"' | head -1 | sed 's/.*"v//' | sed 's/".*//' || echo "")
fi

ARTIFACT="agentic-connect-${VERSION}-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/v${VERSION}/${ARTIFACT}"

if [ -n "$VERSION" ] && curl -fsSL -o "/tmp/$ARTIFACT" "$DOWNLOAD_URL" 2>/dev/null; then
    echo "Downloaded release artifact: $ARTIFACT"
    tar xzf "/tmp/$ARTIFACT" -C "$INSTALL_DIR"
    rm -f "/tmp/$ARTIFACT"
else
    echo "Release artifact not available — building from source..."
    if ! command -v cargo &>/dev/null; then
        echo "Error: cargo not found. Install Rust: https://rustup.rs"
        exit 1
    fi
    TMPDIR=$(mktemp -d)
    git clone --depth 1 "https://github.com/$REPO.git" "$TMPDIR/agentic-connect"
    cd "$TMPDIR/agentic-connect"
    cargo build --release -j 1
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
    rm -rf "$TMPDIR"
fi

# Ensure install dir is in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.bashrc" 2>/dev/null || true
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.zshrc" 2>/dev/null || true
    export PATH="$INSTALL_DIR:$PATH"
fi

# Auto-configure MCP clients (merge-only, never destructive)
configure_mcp() {
    local config_file="$1"
    if [ ! -f "$config_file" ]; then return; fi
    if grep -q "\"$SERVER_KEY\"" "$config_file" 2>/dev/null; then
        echo "  Already configured in $(basename "$config_file")"
        return
    fi
    # Use python/node to merge JSON if available
    if command -v python3 &>/dev/null; then
        python3 -c "
import json, sys
with open('$config_file') as f: cfg = json.load(f)
cfg.setdefault('mcpServers', {})['$SERVER_KEY'] = {'command': '$BINARY_NAME', 'args': []}
with open('$config_file', 'w') as f: json.dump(cfg, f, indent=2)
" 2>/dev/null && echo "  Configured in $(basename "$config_file")" || true
    fi
}

echo ""
echo "Configuring MCP clients..."

# Claude Desktop
configure_mcp "$HOME/Library/Application Support/Claude/claude_desktop_config.json"
configure_mcp "$HOME/.config/claude/claude_desktop_config.json"

# Cursor
configure_mcp "$HOME/.cursor/mcp.json"

# Windsurf
configure_mcp "$HOME/.windsurf/mcp.json"

echo ""
echo "==========================="
echo "AgenticConnect installed!"
echo "==========================="
echo ""
echo "Binary: $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "Quick test:"
echo "  echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}' | $BINARY_NAME"
echo ""
echo "MCP clients configured: Claude Desktop, Cursor, Windsurf (if detected)"
echo ""
echo "For any MCP client, add this to your config:"
echo "  {\"mcpServers\": {\"$SERVER_KEY\": {\"command\": \"$BINARY_NAME\", \"args\": []}}}"
echo ""
echo "Restart your MCP client to activate AgenticConnect."
