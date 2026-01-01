# How to Set Up Claude Code Hooks

This guide shows you how to configure Claude Code to automatically load engram memories at session start.

## Prerequisites

- [Engram CLI installed](install-cli.md) and available in your PATH
- Claude Code installed

## Steps

1. Create the hooks directory:

```bash
mkdir -p .claude/hooks
```

2. Create the session start script at `.claude/hooks/session-start.sh`:

```sh
#!/bin/sh
# Session start hook for Claude Code

set -e

# Run GC silently (clean up low-engagement memories)
engram gc 2>/dev/null || true

# Load memories for this session
engram init --scope global --scope "project:$PWD"
```

3. Make the script executable:

```bash
chmod +x .claude/hooks/session-start.sh
```

4. Create `.claude/hooks.json` to register the hook:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "type": "command",
        "command": ".claude/hooks/session-start.sh"
      }
    ]
  }
}
```

5. Create `.claude/settings.json` to auto-approve engram commands:

```json
{
  "allowedTools": [
    "Bash(engram:*)"
  ]
}
```

6. (Optional) Copy the skill file to help Claude understand how to use engram:

```bash
mkdir -p .claude/skills
cp /path/to/engram/.claude/skills/engram.md .claude/skills/
```

7. Start a new Claude Code session to verify:

```bash
claude
```

You should see your memories loaded in the context.

## Troubleshooting

**Memories not loading?**
- Check that `engram` is in your PATH: `which engram`
- Verify the hook script is executable: `ls -la .claude/hooks/`
- Try running manually: `.claude/hooks/session-start.sh`

**Permission errors?**
- Ensure `~/.engram/` directory exists and is writable
- Ensure the hook script is executable: `chmod +x .claude/hooks/session-start.sh`
