# Claude Code Hooks Setup

Quick setup guide for using engram with Claude Code hooks.

## Install

```bash
cargo install --git https://github.com/nhomble/engram
```

## Local Setup (This Project)

The hooks are already configured in `.claude/hooks/`. Just initialize engram:

```bash
engram init
```

Start a Claude Code session and you'll see:
```
## 🧠 Engram Memory Protocol

Run `engram list` at session start to see all memories.
Tap memories when you use them: `engram tap <id>`
Store learnings immediately: `engram add "content"`

## ACTIVE MEMORIES (from database)
...
```

## Global Setup (All Projects)

To use hooks across all Claude Code projects:

### 1. Copy hooks globally

```bash
mkdir -p ~/.claude/hooks
cp .claude/hooks/*.sh ~/.claude/hooks/
chmod +x ~/.claude/hooks/*.sh
```

### 2. Configure global settings

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

### 3. Initialize engram in each project

```bash
cd /your/project
engram init
```

That's it. The hooks will now run in all Claude Code sessions.
