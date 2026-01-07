# Reference: Claude Hooks Configuration

Technical reference for Claude Code hooks and engram integration.

## Hook Events

### SessionStart

Runs when Claude Code session starts or resumes.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "SessionStart"
}
```

**Typical use**: Display available memories, show project context.

**Default timeout**: 10 seconds

### SessionEnd

Runs when session ends.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "SessionEnd"
}
```

**Typical use**: Remind user to capture learnings.

**Default timeout**: 5 seconds

### PreCompact

Runs before context compression.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "PreCompact"
}
```

**Typical use**: Output critical context to preserve through compaction.

**Default timeout**: 10 seconds

### PreToolUse

Runs before tool execution. **Can block execution**.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "PreToolUse",
  "tool_name": "Bash|Edit|Write|Read|...",
  "tool_input": {
    "command": "...",
    "file_path": "...",
    ...
  }
}
```

**Typical use**: Validate commands, prevent dangerous operations.

**Exit code 2**: Block tool execution

**Default timeout**: 5 seconds

### PostToolUse

Runs after tool execution completes.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "PostToolUse",
  "tool_name": "Bash|Edit|Write|Read|...",
  "tool_input": {
    "command": "...",
    ...
  },
  "tool_result": {
    "exit_code": 0,
    "output": "...",
    ...
  }
}
```

**Typical use**: Suggest memory capture, analyze results.

**Default timeout**: 5 seconds

### UserPromptSubmit

Runs when user submits a message.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "UserPromptSubmit",
  "user_message": "user's input text"
}
```

**Typical use**: Auto-tap relevant memories, context detection.

**Default timeout**: 3 seconds

### Stop

Runs when Claude finishes a response.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "Stop"
}
```

**Typical use**: Post-response analysis, logging.

**Default timeout**: 5 seconds

### SubagentStop

Runs when a subagent task completes.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "SubagentStop",
  "subagent_type": "string"
}
```

**Typical use**: Capture subagent learnings.

**Default timeout**: 5 seconds

### Notification

Runs when Claude sends a notification.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "Notification",
  "notification": {
    "type": "...",
    "message": "..."
  }
}
```

**Typical use**: Log notifications, trigger actions.

**Default timeout**: 3 seconds

### PermissionRequest

Runs when permission dialog is shown.

**Input JSON**:
```json
{
  "session_id": "string",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/working/directory",
  "hook_event_name": "PermissionRequest",
  "permission": {
    "type": "...",
    "resource": "..."
  }
}
```

**Typical use**: Auto-approve trusted operations, logging.

**Default timeout**: 3 seconds

## Settings.json Schema

### Project-level configuration

Location: `.claude/settings.json`

```json
{
  "hooks": {
    "<EventName>": [
      {
        "matcher": "<regex>|<glob>|\"\"",
        "hooks": [
          {
            "type": "command",
            "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/<script>.sh",
            "timeout": <seconds>
          }
        ]
      }
    ]
  }
}
```

### Global configuration

Location: `~/.claude/settings.json`

```json
{
  "hooks": {
    "<EventName>": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash $HOME/.claude/hooks/<script>.sh",
            "timeout": <seconds>
          }
        ]
      }
    ]
  }
}
```

### Configuration fields

**EventName**: One of SessionStart, PreCompact, SessionEnd, PreToolUse, PostToolUse, UserPromptSubmit, Stop, SubagentStop, Notification, PermissionRequest

**matcher**:
- Empty string `""`: Match all
- Regex pattern: `".*\\.rs$"` (for files)
- Tool names: `"Bash|Edit|Write"`

**type**: Always `"command"`

**command**: Shell command to execute
- Use `$CLAUDE_PROJECT_DIR` for project hooks
- Use `$HOME` for global hooks
- Can include pipes, redirects, etc.

**timeout**: Maximum seconds to wait (integer)
- Default varies by event type
- Maximum recommended: 30 seconds
- Hook killed if exceeded

## Environment Variables

### $CLAUDE_PROJECT_DIR

Absolute path to the project directory where Claude Code is running.

**Usage**: Reference project-local hook scripts.

**Example**: `bash $CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh`

### $HOME

User's home directory.

**Usage**: Reference global hook scripts.

**Example**: `bash $HOME/.claude/hooks/session-start.sh`

### $ENGRAM_DB_PATH

Path to engram database file. Optional.

**Default**: `.engram/engram.db` in current directory

**Usage**: Use global database or custom location.

**Example**:
```bash
export ENGRAM_DB_PATH=~/.engram/global.db
```

## Engram Commands

### engram init

Initialize database in current directory.

**Usage**:
```bash
engram init [--scope <scope>]
```

**Options**:
- `--scope <scope>`: Add scope to memories (e.g., `project:$PWD`)

### engram add

Add a new memory.

**Usage**:
```bash
engram add "<content>" [--scope <scope>]
```

**Returns**: Memory ID (e.g., `engram-abc123`)

### engram list

List all memories.

**Usage**:
```bash
engram list [--scope <scope>]
```

**Output format**:
```
[engram-abc123] taps:5 | Memory content here
[engram-def456] taps:2 | Another memory
```

### engram tap

Increment tap count for a memory.

**Usage**:
```bash
engram tap <id>
```

**Example**:
```bash
engram tap engram-abc123
```

### engram show

Display detailed memory information.

**Usage**:
```bash
engram show <id>
```

### engram stats

Show database statistics.

**Usage**:
```bash
engram stats [--scope <scope>]
```

**Output**: Total memories, tap counts, scope information.

### engram hot

Show recently tapped memories.

**Usage**:
```bash
engram hot [--limit <n>]
```

**Options**:
- `--limit <n>`: Number of memories to show (default: 10)

### engram prime

Output memory context for Claude.

**Usage**:
```bash
engram prime
```

**Output**: Formatted context suitable for preserving through compaction.

### engram remove

Delete a memory.

**Usage**:
```bash
engram remove <id>
```

### engram log

Show event log.

**Usage**:
```bash
engram log [--limit <n>]
```

**Options**:
- `--limit <n>`: Number of events to show

## Exit Codes

### Hook exit codes

- `0`: Success, continue
- `2`: Block action (PreToolUse only)
- Other: Error (logged, non-blocking)

### Engram command exit codes

- `0`: Success
- `1`: Error (database, invalid arguments, etc.)
- `137`: Interrupted by user

## File Locations

### Project-level

```
<project-root>/
  .claude/
    settings.json          # Hook configuration
    hooks/
      session-start.sh
      pre-compact.sh
      session-end.sh
  .engram/
    engram.db             # SQLite database
    engram.db-wal         # Write-ahead log
    engram.db-shm         # Shared memory
```

### Global

```
~/
  .claude/
    settings.json         # Global hook configuration
    hooks/
      *.sh               # Global hook scripts
  .local/bin/
    engram               # engram binary
  .engram/
    global.db           # Optional global database
```

## Hook Script Template

```bash
#!/bin/bash
# Hook description
set -e

# Check if engram is available
if ! command -v engram &> /dev/null; then
    echo "⚠️  Engram not found" >&2
    exit 0
fi

# Read input from stdin (optional)
INPUT=$(cat)

# Your hook logic here
engram list 2>&1 | head -10

# Output to stderr for visibility
echo "Hook message" >&2

# Exit successfully
exit 0
```

## Performance Considerations

**Timeout recommendations**:
- Fast queries (list, stats): 5-10 seconds
- Database operations (add, tap): 3-5 seconds
- Complex analysis: 10-30 seconds

**Optimization**:
- Limit output with `head`
- Use `VACUUM` to optimize database
- Checkpoint WAL files periodically
- Avoid expensive operations in hooks

**WAL checkpoint**:
```bash
sqlite3 .engram/engram.db "PRAGMA wal_checkpoint(TRUNCATE);"
```

## See also

- **tutorials/getting-started-claude-hooks.md** - First-time setup
- **how-to/create-custom-hooks.md** - Add new hooks
- **explanation/claude-hooks-engram-integration.md** - Architecture
