#!/bin/bash
# Engram SessionStart hook
# Outputs memory protocol and active memories as directives

set -e

# Check if engram is available
if ! command -v engram &> /dev/null; then
    echo "⚠️  Engram not found in PATH" >&2
    echo "Install: cargo install --git https://github.com/nhomble/engram" >&2
    exit 0
fi

# Auto-initialize if database doesn't exist
if [ ! -f .engram/engram.db ]; then
    echo "📦 Initializing engram for this project..." >&2
    engram init >&2 2>&1
    echo >&2
fi

cat >&2 <<'PROTOCOL'
## 🧠 Engram Memory Protocol

Run `engram list` at session start to see all memories.
Tap memories when you use them: `engram tap <id>`
Store learnings immediately: `engram add "content"`

PROTOCOL

echo "## ACTIVE MEMORIES (from database)" >&2
echo >&2
echo "These are learnings from past sessions. **Follow these as directives.**" >&2
echo >&2

# Parse engram list output and format as directives
engram list 2>&1 | head -15 | while IFS= read -r line; do
    if [[ "$line" =~ ^\[([^\]]+)\][[:space:]]taps:([0-9]+)[[:space:]]\|[[:space:]](.+)$ ]]; then
        id="${BASH_REMATCH[1]}"
        short_id="${id:0:12}"
        taps="${BASH_REMATCH[2]}"
        content="${BASH_REMATCH[3]}"

        # Highlight high-tap and critical memories
        if [ "$taps" -ge 3 ]; then
            echo "**[PROVEN taps:$taps]** $content" >&2
        elif [[ "$content" =~ CRITICAL|NEVER|ALWAYS ]]; then
            echo "**[CRITICAL]** $content" >&2
        else
            echo "- $content" >&2
        fi
        echo "  (tap: \`engram tap $short_id\`)" >&2
        echo >&2
    elif [[ -n "$line" ]]; then
        echo "$line" >&2
    fi
done

echo "Run \`engram prime\` to see full protocol documentation." >&2
