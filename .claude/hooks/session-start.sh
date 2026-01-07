#!/bin/bash
# Engram SessionStart hook
# Shows available memories at the start of each Claude Code session

set -e

# Check if engram is available
if ! command -v engram &> /dev/null; then
    echo "⚠️  Engram not found in PATH" >&2
    exit 0
fi

# List recent memories
echo "📚 Engram Memories Available:"
echo
engram list 2>&1 | head -10

echo
echo "Run 'engram tap <id>' to mark memories as used"
echo "Run 'engram add \"content\"' to store new learnings"
