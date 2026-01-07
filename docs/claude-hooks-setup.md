# Claude Code Hooks Setup for Engram

This guide explains how to install and use engram's Claude Code hooks integration for automatic memory assistance.

## What Are Claude Hooks?

Claude Code hooks are shell commands that run automatically at specific points in Claude's lifecycle. Engram provides hooks that:

- **SessionStart**: Show available memories when Claude starts
- **PreCompact**: Preserve memory context before context compression
- **SessionEnd**: Remind you to capture new learnings

## Local Installation (This Project)

The hooks are already configured in this repository! To use them:

### 1. Install engram CLI

```bash
cargo build --release
cp target/release/engram ~/.local/bin/
```

Verify installation:

```bash
engram --help
```

### 2. Initialize engram database

```bash
engram init
```

This creates `.engram/engram.db` in your project directory.

### 3. Test hooks manually

Run each hook script to verify they work:

```bash
# Test SessionStart hook
bash .claude/hooks/session-start.sh

# Test PreCompact hook
bash .claude/hooks/pre-compact.sh

# Test SessionEnd hook
bash .claude/hooks/session-end.sh
```

Expected output:
```
📚 Engram Memories Available:

No memories found.

Run 'engram tap <id>' to mark memories as used
Run 'engram add "content"' to store new learnings
```

### 4. Hooks activate automatically

Hooks are configured in `.claude/settings.json` and will run automatically when you use Claude Code in this project.

## Hook Behavior

### SessionStart Hook

Runs when Claude Code starts or resumes. Shows up to 10 recent memories:

```
📚 Engram Memories Available:

[engram-abc123] taps:5 | User prefers concise responses
[engram-def456] taps:2 | Always run tests before commit
[engram-xyz789] taps:1 | OAuth requires HTTPS in production

Run 'engram tap <id>' to mark memories as used
Run 'engram add "content"' to store new learnings
```

### PreCompact Hook

Runs before Claude's context is compressed. Outputs full memory context via `engram prime` so important information is preserved through compaction.

### SessionEnd Hook

Runs when the session ends. Reminds you to capture learnings:

```
📝 Session ending. Consider adding memories for:
   - User corrections or preferences discovered
   - Technical patterns or architecture decisions
   - Error solutions with context

Use: engram add "your memory content"
```

## Global Installation (All Projects)

To use engram hooks across all your Claude Code projects:

### 1. Copy hooks to global location

```bash
mkdir -p ~/.claude/hooks
cp .claude/hooks/*.sh ~/.claude/hooks/
chmod +x ~/.claude/hooks/*.sh
```

### 2. Update global settings

Edit `~/.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash $HOME/.claude/hooks/session-start.sh",
            "timeout": 10
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash $HOME/.claude/hooks/pre-compact.sh",
            "timeout": 10
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash $HOME/.claude/hooks/session-end.sh",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

**Important**: Use `$HOME` instead of `$CLAUDE_PROJECT_DIR` for global hooks.

### 3. Per-project databases

Each project needs its own engram database. Run `engram init` in each project directory.

Alternatively, use a global database with scoping:

```bash
# Set global database path
export ENGRAM_DB_PATH=~/.engram/global.db

# Initialize with project scope
engram init --scope "project:$(pwd)"

# Add to shell profile
echo 'export ENGRAM_DB_PATH=~/.engram/global.db' >> ~/.zshrc
```

## Customization

### Modify Hook Scripts

Edit the hook scripts in `.claude/hooks/` to customize behavior:

**Show only high-value memories** (tapped 3+ times):

```bash
#!/bin/bash
# session-start.sh
engram list | awk -F'taps:' '$2 >= 3 {print}'
```

**Auto-tap memories matching current work:**

```bash
#!/bin/bash
# session-start.sh
BRANCH=$(git branch --show-current 2>/dev/null)
if [[ -n "$BRANCH" ]]; then
    engram tap --match "$BRANCH"
fi
```

**Include recent activity:**

```bash
#!/bin/bash
# session-start.sh
echo "📚 Recent Memories:"
engram list | head -5

echo
echo "🔥 Hot Memories (recently tapped):"
engram hot --limit 3
```

### Adjust Timeouts

Edit `.claude/settings.json` to change hook timeouts:

```json
{
  "type": "command",
  "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh",
  "timeout": 20  // Increase if database is slow
}
```

## Troubleshooting

### Hooks not running

**Check if hooks are registered:**

In Claude Code, run `/hooks` to see all configured hooks.

**Verify hook scripts are executable:**

```bash
ls -la .claude/hooks/
# Should show: -rwxr-xr-x (executable)
```

If not:

```bash
chmod +x .claude/hooks/*.sh
```

**Test hooks manually:**

```bash
bash .claude/hooks/session-start.sh
```

### "engram: command not found"

**engram not in PATH**. Add to shell profile:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

Or use absolute path in hooks:

```bash
command: "$HOME/.local/bin/engram list"
```

### Hooks timing out

Increase timeout in `.claude/settings.json`:

```json
"timeout": 30  // Increase from default 10
```

Or optimize database:

```bash
sqlite3 .engram/engram.db "VACUUM;"
```

### Database not found

Initialize engram:

```bash
engram init
```

Or set explicit database path:

```bash
export ENGRAM_DB_PATH=/path/to/engram.db
```

## Hook Development

### Adding New Hooks

Create a new hook script:

```bash
touch .claude/hooks/my-hook.sh
chmod +x .claude/hooks/my-hook.sh
```

Register in `.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/my-hook.sh",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

### Available Hook Events

- **PreToolUse**: Before tool calls (can block)
- **PostToolUse**: After tool calls complete
- **UserPromptSubmit**: When user submits prompt
- **Stop**: When Claude finishes responding
- **SubagentStop**: When subagent completes
- **PreCompact**: Before context compression
- **SessionStart**: Session begins
- **SessionEnd**: Session ends
- **Notification**: Claude sends notification
- **PermissionRequest**: Permission dialog shown

### Hook Input

Hooks receive JSON via stdin:

```json
{
  "session_id": "abc123",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/directory",
  "hook_event_name": "SessionStart"
}
```

Parse with `jq`:

```bash
#!/bin/bash
SESSION_ID=$(jq -r '.session_id')
echo "Session: $SESSION_ID"
```

### Hook Output

Exit codes:
- `0`: Success
- `2`: Block action (PreToolUse only)
- Other: Error (non-blocking)

Output to stderr to show messages:

```bash
echo "⚠️ Warning: Large file" >&2
exit 0
```

## Examples

### Example: Auto-add memories on errors

Detect when errors are solved and suggest adding memories:

```bash
#!/bin/bash
# post-bash-hook.sh

# Get command and exit code from stdin
COMMAND=$(jq -r '.tool_input.command')
EXIT_CODE=$(jq -r '.tool_result.exit_code // 0')

if [[ $EXIT_CODE -eq 0 ]] && echo "$COMMAND" | grep -q "fix\|error\|debug"; then
    echo "✓ Error resolved. Consider: engram add \"solution description\"" >&2
fi
```

### Example: Show context-relevant memories

Filter memories by current file:

```bash
#!/bin/bash
# smart-session-start.sh

CURRENT_FILE=$(git diff --name-only HEAD 2>/dev/null | head -1)
if [[ -n "$CURRENT_FILE" ]]; then
    CONTEXT=$(basename "$CURRENT_FILE" .*)
    engram tap --match "$CONTEXT"
    echo "Memories related to: $CONTEXT"
fi
```

## Benefits

✅ **Automatic context**: Memories shown at session start
✅ **Survives compaction**: Context preserved through PreCompact
✅ **Capture reminders**: SessionEnd prompts for new learnings
✅ **Zero manual effort**: Hooks run automatically
✅ **Project-portable**: Uses $CLAUDE_PROJECT_DIR
✅ **Customizable**: Edit scripts to fit your workflow

## Next Steps

- Add your first memory: `engram add "your learning"`
- Start a Claude session and see hooks in action
- Customize hook scripts for your workflow
- Consider adding PostToolUse hooks for automatic capture
