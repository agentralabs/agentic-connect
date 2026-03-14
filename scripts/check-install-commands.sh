#!/bin/bash
# Verify install script and README install commands are consistent.
set -euo pipefail

ERRORS=0

echo "=== Install Command Guardrails ==="

# Check install script exists
if [ ! -f scripts/install.sh ]; then
    echo "FAIL: scripts/install.sh not found"
    ERRORS=$((ERRORS + 1))
else
    echo "PASS: scripts/install.sh exists"
fi

# Check README mentions all install profiles
for profile in desktop terminal server; do
    if grep -q "install/connect/$profile" README.md 2>/dev/null; then
        echo "PASS: README mentions $profile profile"
    else
        echo "FAIL: README missing $profile profile install command"
        ERRORS=$((ERRORS + 1))
    fi
done

# Check standalone guarantee
if grep -q "Standalone guarantee" README.md 2>/dev/null || grep -q "standalone" README.md 2>/dev/null; then
    echo "PASS: README includes standalone guarantee"
else
    echo "FAIL: README missing standalone guarantee"
    ERRORS=$((ERRORS + 1))
fi

# Check install script has profile support
if grep -q "PROFILE" scripts/install.sh 2>/dev/null; then
    echo "PASS: Install script supports profiles"
else
    echo "FAIL: Install script missing profile support"
    ERRORS=$((ERRORS + 1))
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "All install command checks passed."
else
    echo "$ERRORS check(s) failed."
    exit 1
fi
