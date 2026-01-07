# How-to: Customize Hook Scripts

Modify engram hooks to fit your workflow.

## Problem

You want to change what information hooks display or when they run.

## Solution

Edit the hook shell scripts in `.claude/hooks/` (or `~/.claude/hooks/` for global installation).

## Show only high-value memories

Filter memories by tap count in `session-start.sh`:

```bash
#!/bin/bash
# session-start.sh
set -e

if ! command -v engram &> /dev/null; then
    echo "⚠️  Engram not found in PATH" >&2
    exit 0
fi

echo "📚 High-Value Memories (3+ taps):"
echo
engram list 2>&1 | awk -F'taps:' '$2 >= 3'

echo
echo "Run 'engram tap <id>' to mark memories as used"
```

## Show recent activity

Include hot memories in `session-start.sh`:

```bash
#!/bin/bash
set -e

if ! command -v engram &> /dev/null; then
    exit 0
fi

echo "📚 Recent Memories:"
engram list 2>&1 | head -5

echo
echo "🔥 Hot Memories (recently tapped):"
engram hot --limit 3

echo
echo "Run 'engram tap <id>' to mark memories as used"
```

## Auto-tap based on git branch

Automatically tap memories matching the current branch:

```bash
#!/bin/bash
# session-start.sh
set -e

if ! command -v engram &> /dev/null; then
    exit 0
fi

# Get current branch
BRANCH=$(git branch --show-current 2>/dev/null)

if [[ -n "$BRANCH" ]]; then
    echo "📚 Auto-tapping memories for branch: $BRANCH"
    # Note: Requires --match flag implementation
    engram tap --match "$BRANCH" 2>&1 || true
fi

engram list 2>&1 | head -10
echo
echo "Run 'engram tap <id>' to mark memories as used"
```

## Filter by context

Show memories relevant to current work:

```bash
#!/bin/bash
# session-start.sh
set -e

if ! command -v engram &> /dev/null; then
    exit 0
fi

# Get recently modified files
CURRENT_FILE=$(git diff --name-only HEAD 2>/dev/null | head -1)

if [[ -n "$CURRENT_FILE" ]]; then
    CONTEXT=$(basename "$CURRENT_FILE" .*)
    echo "📚 Memories related to: $CONTEXT"
    # Filter output by context keyword
    engram list 2>&1 | grep -i "$CONTEXT" || engram list 2>&1 | head -5
else
    engram list 2>&1 | head -10
fi

echo
echo "Run 'engram tap <id>' to mark memories as used"
```

## Adjust hook timeouts

Edit `.claude/settings.json` to increase timeout for slow operations:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh",
            "timeout": 20
          }
        ]
      }
    ]
  }
}
```

## Add project-specific context

Include project information in `session-start.sh`:

```bash
#!/bin/bash
set -e

if ! command -v engram &> /dev/null; then
    exit 0
fi

echo "📚 Engram Memories - $(basename $(pwd))"
echo

# Show project statistics
echo "Database stats:"
engram stats 2>&1

echo
echo "Recent memories:"
engram list 2>&1 | head -5

echo
echo "Run 'engram tap <id>' to mark memories as used"
```

## Testing changes

After modifying a hook script, test it manually:

```bash
bash .claude/hooks/session-start.sh
```

Then start a Claude session to see it in action.

## See also

- **how-to/create-custom-hooks.md** - Add new hook events
- **reference/claude-hooks-reference.md** - Available engram commands
- **explanation/claude-hooks-engram-integration.md** - Hook architecture
