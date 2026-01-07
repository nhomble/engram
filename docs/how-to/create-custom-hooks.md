# How-to: Create Custom Hooks

Add new hooks for different Claude Code events.

## Problem

You want hooks to run at different points in Claude's lifecycle (e.g., after tool use, when user submits prompts).

## Solution

Create new hook scripts and register them in `.claude/settings.json` for specific events.

## Available hook events

- **PreToolUse**: Before tool calls (can block)
- **PostToolUse**: After tool calls complete
- **UserPromptSubmit**: When user submits a prompt
- **Stop**: When Claude finishes responding
- **SubagentStop**: When subagent completes
- **PreCompact**: Before context compression
- **SessionStart**: Session begins
- **SessionEnd**: Session ends
- **Notification**: Claude sends notification
- **PermissionRequest**: Permission dialog shown

See **reference/claude-hooks-reference.md** for complete details.

## Create a PostToolUse hook

**Use case**: Suggest adding memories when errors are solved.

### 1. Create hook script

```bash
touch .claude/hooks/post-tool-use.sh
chmod +x .claude/hooks/post-tool-use.sh
```

### 2. Write hook logic

Edit `.claude/hooks/post-tool-use.sh`:

```bash
#!/bin/bash
# Post-tool-use hook for engram
set -e

if ! command -v engram &> /dev/null; then
    exit 0
fi

# Read hook input from stdin
INPUT=$(cat)

# Extract tool name and result
TOOL=$(echo "$INPUT" | jq -r '.tool_name // ""')
EXIT_CODE=$(echo "$INPUT" | jq -r '.tool_result.exit_code // 0')

# If bash command succeeded and looks like error fixing
if [[ "$TOOL" == "Bash" ]] && [[ $EXIT_CODE -eq 0 ]]; then
    COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // ""')

    if echo "$COMMAND" | grep -qE "fix|error|debug|test"; then
        echo "✓ Looks like you fixed something! Consider:" >&2
        echo "  engram add \"solution description\"" >&2
    fi
fi

exit 0
```

### 3. Register in settings

Edit `.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/post-tool-use.sh",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

### 4. Test the hook

Run a bash command that matches the pattern:

```bash
# In Claude Code
Run: cargo test --fix
```

You should see the suggestion to add a memory.

## Create a UserPromptSubmit hook

**Use case**: Auto-tap memories based on user question keywords.

### 1. Create script

```bash
#!/bin/bash
# .claude/hooks/user-prompt-submit.sh
set -e

if ! command -v engram &> /dev/null; then
    exit 0
fi

# Get user message from stdin
INPUT=$(cat)
USER_MESSAGE=$(echo "$INPUT" | jq -r '.user_message // ""')

# Extract keywords and tap matching memories
if echo "$USER_MESSAGE" | grep -qi "authentication\|auth"; then
    echo "🔍 Auto-tapping auth-related memories" >&2
    # Note: Requires --match flag implementation
    engram tap --match "auth" 2>&1 >&2 || true
fi

exit 0
```

### 2. Register in settings

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/user-prompt-submit.sh",
            "timeout": 3
          }
        ]
      }
    ]
  }
}
```

## Hook input format

All hooks receive JSON via stdin with this structure:

```json
{
  "session_id": "abc123",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/directory",
  "hook_event_name": "PostToolUse",
  "tool_name": "Bash",
  "tool_input": { "command": "cargo test" },
  "tool_result": { "exit_code": 0 }
}
```

Parse with `jq`:

```bash
INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id')
TOOL=$(echo "$INPUT" | jq -r '.tool_name // ""')
```

## Hook output

**Exit codes**:
- `0`: Success
- `2`: Block action (PreToolUse only)
- Other: Error (logged but non-blocking)

**Output to stderr** to show messages:

```bash
echo "⚠️  Warning: Large file detected" >&2
exit 0
```

**Blocking example** (PreToolUse):

```bash
#!/bin/bash
# Block dangerous commands

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // ""')

if echo "$COMMAND" | grep -q "rm -rf /"; then
    echo "🚫 Blocked dangerous command" >&2
    exit 2  # Block the action
fi

exit 0
```

## Matcher patterns

Filter when hooks run using matchers:

**Match specific tools**:

```json
{
  "matcher": "Write|Edit",
  "hooks": [...]
}
```

**Match file patterns**:

```json
{
  "matcher": ".*\\.rs$",
  "hooks": [...]
}
```

**Match all**:

```json
{
  "matcher": "",
  "hooks": [...]
}
```

## Testing custom hooks

**Test with mock input**:

```bash
echo '{"session_id":"test","tool_name":"Bash"}' | bash .claude/hooks/post-tool-use.sh
```

**Check exit code**:

```bash
bash .claude/hooks/my-hook.sh
echo $?  # Should be 0
```

**Verify in Claude Code**:

Run the action that should trigger the hook and check for output.

## Examples

### Auto-add memory for build failures

```bash
#!/bin/bash
# post-tool-use.sh

INPUT=$(cat)
TOOL=$(echo "$INPUT" | jq -r '.tool_name')
EXIT_CODE=$(echo "$INPUT" | jq -r '.tool_result.exit_code // 0')

if [[ "$TOOL" == "Bash" ]] && [[ $EXIT_CODE -ne 0 ]]; then
    COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command')

    if echo "$COMMAND" | grep -q "cargo build\|npm run build"; then
        ERROR=$(echo "$INPUT" | jq -r '.tool_result.output')
        echo "Build failed. Consider capturing the solution once fixed." >&2
    fi
fi
```

### Show context-relevant memories on file edits

```bash
#!/bin/bash
# post-tool-use.sh

INPUT=$(cat)
TOOL=$(echo "$INPUT" | jq -r '.tool_name')

if [[ "$TOOL" == "Edit" ]] || [[ "$TOOL" == "Write" ]]; then
    FILE=$(echo "$INPUT" | jq -r '.tool_input.file_path')
    BASENAME=$(basename "$FILE" .*)

    echo "📚 Memories related to $BASENAME:" >&2
    engram list 2>&1 | grep -i "$BASENAME" >&2 || true
fi
```

## See also

- **reference/claude-hooks-reference.md** - Complete hook event reference
- **how-to/customize-hook-scripts.md** - Modify existing hooks
- **explanation/claude-hooks-engram-integration.md** - Architecture
