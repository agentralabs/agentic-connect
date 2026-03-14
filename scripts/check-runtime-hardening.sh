#!/bin/bash
# Runtime hardening guardrails — validate MCP server safety.
set -uo pipefail

ERRORS=0
WARNS=0

echo "=== Runtime Hardening Guardrails ==="

# 1. No unwrap() on user input paths
UNWRAPS=$(grep -rn "\.unwrap()" crates/agentic-connect-mcp/src/tools/*.rs 2>/dev/null | grep -v "test" | grep -v "//" | wc -l | tr -d ' ')
if [ "$UNWRAPS" -gt 5 ]; then
    echo "WARN: $UNWRAPS unwrap() calls in tool handlers — prefer error propagation"
    WARNS=$((WARNS + 1))
else
    echo "PASS: Minimal unwrap() in tool handlers ($UNWRAPS)"
fi

# 2. Unknown tool returns -32803 (not -32601)
if grep -q "32803" crates/agentic-connect-mcp/src/types.rs; then
    echo "PASS: TOOL_NOT_FOUND uses error code -32803"
else
    echo "FAIL: Missing TOOL_NOT_FOUND_CODE = -32803"
    ERRORS=$((ERRORS + 1))
fi

# 3. No hardcoded credentials in source (exclude test code)
SECRETS=$(grep -rn "sk-\|secret_key" crates/agentic-connect/src/ crates/agentic-connect-mcp/src/ 2>/dev/null | grep -v "test\|Test\|cfg(test)\|mod tests\|example" | wc -l | tr -d ' ')
if [ "$SECRETS" -gt 0 ]; then
    echo "FAIL: Possible hardcoded secrets found in source ($SECRETS occurrences)"
    ERRORS=$((ERRORS + 1))
else
    echo "PASS: No hardcoded secrets in source"
fi

# 4. File size guard
OVERSIZE=$(find crates -name "*.rs" -exec sh -c 'lines=$(wc -l < "$1"); if [ "$lines" -gt 400 ]; then echo "$lines $1"; fi' _ {} \;)
if [ -n "$OVERSIZE" ]; then
    echo "WARN: Files over 400 lines:"
    echo "$OVERSIZE"
    WARNS=$((WARNS + 1))
else
    echo "PASS: All .rs files under 400 lines"
fi

# 5. MCP tool descriptions — no trailing periods
TRAILING_PERIODS=$(grep -rn 'description.*\."' crates/agentic-connect-mcp/src/tools/*.rs 2>/dev/null | wc -l | tr -d ' ')
if [ "$TRAILING_PERIODS" -gt 0 ]; then
    echo "WARN: $TRAILING_PERIODS tool descriptions may have trailing periods"
    WARNS=$((WARNS + 1))
else
    echo "PASS: No trailing periods in tool descriptions"
fi

# 6. Tests exist
TEST_COUNT=$(find crates -name "*.rs" -path "*/tests/*" | wc -l | tr -d ' ')
if [ "$TEST_COUNT" -ge 4 ]; then
    echo "PASS: $TEST_COUNT test files found"
else
    echo "FAIL: Insufficient test files ($TEST_COUNT < 4)"
    ERRORS=$((ERRORS + 1))
fi

# 7. Cargo.lock present
if [ -f "Cargo.lock" ]; then
    echo "PASS: Cargo.lock present"
else
    echo "FAIL: Cargo.lock missing (required for binary crates)"
    ERRORS=$((ERRORS + 1))
fi

# 8. No panic in production code
PANICS=$(grep -rn "panic!" crates/agentic-connect-mcp/src/ crates/agentic-connect/src/ 2>/dev/null | grep -v "test\|unreachable" | wc -l | tr -d ' ')
if [ "$PANICS" -gt 0 ]; then
    echo "WARN: $PANICS explicit panic! calls in production code"
    WARNS=$((WARNS + 1))
else
    echo "PASS: No explicit panic! in production code"
fi

echo ""
echo "Results: $ERRORS errors, $WARNS warnings"
if [ $ERRORS -eq 0 ]; then
    echo "All runtime hardening checks passed."
else
    echo "$ERRORS critical check(s) failed."
    exit 1
fi
