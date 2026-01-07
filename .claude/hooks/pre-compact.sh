#!/bin/bash
# Engram PreCompact hook
# Outputs memory context before Claude's context is compacted

set -e

# Check if engram is available
if ! command -v engram &> /dev/null; then
    exit 0
fi

# Output engram prime context for preservation
echo "🧠 Engram Context Recovery:"
echo
engram prime 2>&1
