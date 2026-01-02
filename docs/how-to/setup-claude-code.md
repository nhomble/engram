# How to Set Up Engram for Claude Code

This guide shows you how to configure Claude Code to use engram for memory observability.

## Prerequisites

- [Engram CLI installed](install-cli.md) and available in your PATH
- Claude Code installed

## Steps

1. Create the skills directory in your project:

```bash
mkdir -p .claude/skills
```

2. Copy the engram skill file:

```bash
cp /path/to/engram/.claude/skills/engram.md .claude/skills/
```

3. Create `.claude/settings.json` to auto-approve engram commands:

```json
{
  "allowedTools": [
    "Bash(engram:*)"
  ]
}
```

4. Start a Claude Code session:

```bash
claude
```

The agent will now proactively store and tap memories. You can observe with:

```bash
# Watch memories in real-time (in another terminal)
watch engram list

# View event log
engram log

# View statistics
engram stats
```

## Optional: Periodic GC

Run garbage collection periodically to clean up unused memories:

```bash
# Manual
engram gc

# Or add to crontab (daily at midnight)
0 0 * * * engram gc
```

## Troubleshooting

**Commands not auto-approved?**
- Check `.claude/settings.json` has the `allowedTools` entry
- Restart Claude Code session

**Engram not found?**
- Verify `engram` is in your PATH: `which engram`
