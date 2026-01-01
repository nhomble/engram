# How to Set Up Claude Code Hooks

This guide shows you how to configure Claude Code to automatically load engram memories at session start.

## Prerequisites

- [Engram CLI installed](install-cli.md) and available in your PATH
- Claude Code installed

## Steps

1. Create the `.claude` directory in your project (or home directory for global config):

```bash
mkdir -p .claude
```

2. Create `.claude/hooks.json` with the session start hook:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "type": "command",
        "command": "engram gc 2>/dev/null; engram init --scope global --scope \"project:$PWD\""
      }
    ]
  }
}
```

This runs garbage collection and loads memories every time a Claude Code session starts.

3. Create `.claude/settings.json` to auto-approve engram commands:

```json
{
  "allowedTools": [
    "Bash(engram:*)"
  ]
}
```

4. (Optional) Copy the skill file to help Claude understand how to use engram:

```bash
mkdir -p .claude/skills
cp /path/to/engram/.claude/skills/engram.md .claude/skills/
```

5. Start a new Claude Code session to verify:

```bash
claude
```

You should see your memories loaded in the context.

## Troubleshooting

**Memories not loading?**
- Check that `engram` is in your PATH: `which engram`
- Verify the hook syntax in `.claude/hooks.json`
- Try running manually: `engram init --scope global --scope "project:$PWD"`

**Permission errors?**
- Ensure `~/.engram/` directory exists and is writable
