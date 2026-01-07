#!/bin/bash
# Engram SessionEnd hook
# Prompts to review if any new memories should be added

set -e

# Check if engram is available
if ! command -v engram &> /dev/null; then
    exit 0
fi

echo "📝 Session ending. Consider adding memories for:"
echo "   - User corrections or preferences discovered"
echo "   - Technical patterns or architecture decisions"
echo "   - Error solutions with context"
echo
echo "Use: engram add \"your memory content\""
