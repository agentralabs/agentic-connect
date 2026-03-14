#!/bin/bash
# Verify canonical sister kit requirements from CANONICAL_SISTER_KIT.md.
set -euo pipefail

ERRORS=0

echo "=== Canonical Sister Guardrails ==="

# Required files
for f in README.md LICENSE Cargo.toml scripts/install.sh; do
    if [ -f "$f" ]; then
        echo "PASS: $f exists"
    else
        echo "FAIL: $f missing"
        ERRORS=$((ERRORS + 1))
    fi
done

# Required CI workflows
for wf in ci.yml canonical-sister-guardrails.yml install-command-guardrails.yml; do
    if [ -f ".github/workflows/$wf" ]; then
        echo "PASS: .github/workflows/$wf exists"
    else
        echo "FAIL: .github/workflows/$wf missing"
        ERRORS=$((ERRORS + 1))
    fi
done

# Required doc pages
for doc in quickstart.md concepts.md api-reference.md; do
    if [ -f "docs/$doc" ] || [ -f "docs/public/$doc" ]; then
        echo "PASS: docs/$doc exists"
    else
        echo "WARN: docs/$doc missing (required before release)"
    fi
done

# MCP server key check
if grep -q '"agentic-connect"' README.md 2>/dev/null; then
    echo "PASS: README uses stable MCP server key"
else
    echo "FAIL: README missing MCP server key"
    ERRORS=$((ERRORS + 1))
fi

# Env var namespace check
if grep -q 'ACNX_' README.md 2>/dev/null; then
    echo "PASS: README uses ACNX_ env var prefix"
else
    echo "WARN: README should document ACNX_ env vars"
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "All canonical sister checks passed."
else
    echo "$ERRORS check(s) failed."
    exit 1
fi
