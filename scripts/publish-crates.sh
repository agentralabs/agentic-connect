#!/bin/bash
# Publish all AgenticConnect crates to crates.io in dependency order.
# Run after rate limit window resets.
#
# Usage: bash scripts/publish-crates.sh
#
# Requires: CARGO_REGISTRY_TOKEN set or `cargo login` completed.

set -euo pipefail

echo "=== Publishing AgenticConnect to crates.io ==="
echo ""

# Order matters: core first, then dependents
CRATES=(
  "agentic-connect"
  "agentic-connect-ffi"
  "agentic-connect-mcp"
  "agentic-connect-cli"
)

for crate in "${CRATES[@]}"; do
  echo "Publishing $crate..."

  # Try up to 12 times with 20s waits for index propagation
  for attempt in $(seq 1 12); do
    output=$(cargo publish -p "$crate" 2>&1) && {
      echo "  ✓ $crate published successfully"
      break
    } || {
      if echo "$output" | grep -q "already uploaded\|already exists"; then
        echo "  ✓ $crate already published (skipping)"
        break
      elif echo "$output" | grep -q "failed to select a version\|no matching package\|could not find"; then
        echo "  ⏳ Waiting for index propagation (attempt $attempt/12)..."
        sleep 20
      elif echo "$output" | grep -q "Too Many Requests\|429"; then
        echo "  ⚠ Rate limited. Wait and retry later."
        echo "  $output"
        exit 1
      else
        echo "  ✗ Failed: $output"
        exit 1
      fi
    }
  done

  # Wait between crates for index propagation
  if [ "$crate" != "${CRATES[-1]}" ]; then
    echo "  Waiting 15s for index propagation..."
    sleep 15
  fi
done

echo ""
echo "=== All crates published ==="
echo ""
echo "Verify at:"
for crate in "${CRATES[@]}"; do
  echo "  https://crates.io/crates/$crate"
done
