# Claude Code Hooks Setup

Quick setup guide for using engram with Claude Code hooks.

## Quick Install (Recommended)

### 1. Install engram binary

```bash
cargo install --git https://github.com/nhomble/engram
```

### 2. Install plugin

In Claude Code:

```bash
/plugin marketplace add nhomble/engram
/plugin install engram
```

That's it! The plugin auto-configures hooks and auto-initializes engram in each project.

When you start a Claude session, you'll see:
```
📦 Initializing engram for this project...

## 🧠 Engram Memory Protocol

Run `engram list` at session start to see all memories.
Tap memories when you use them: `engram tap <id>`
Store learnings immediately: `engram add "content"`

## ACTIVE MEMORIES (from database)
...
```

## Manual Setup (Alternative)

If you prefer not to use the plugin:

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

## Upgrading

### Upgrade binary

```bash
cargo install --git https://github.com/nhomble/engram --force
```

### Upgrade plugin

```bash
/plugin marketplace update
/plugin update engram
```

**Note**: Keep binary and plugin versions in sync for best compatibility.
