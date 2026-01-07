# How-to: Install Hooks Globally

Use engram hooks across all your Claude Code projects.

## Problem

You want engram memory assistance in every project, not just the engram repository.

## Solution

Install hooks to `~/.claude/hooks` and configure global settings.

## Steps

### 1. Copy hooks to global location

From the engram project directory:

```bash
mkdir -p ~/.claude/hooks
cp .claude/hooks/*.sh ~/.claude/hooks/
chmod +x ~/.claude/hooks/*.sh
```

### 2. Update global settings

Edit or create `~/.claude/settings.json`:

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

### 3. Choose database strategy

**Option A: Per-project databases** (recommended)

Initialize engram in each project:

```bash
cd /path/to/project
engram init
```

This creates `.engram/engram.db` in each project directory.

**Option B: Global database with scoping**

Use a single database with project scopes:

```bash
# Create global database directory
mkdir -p ~/.engram

# Set environment variable
export ENGRAM_DB_PATH=~/.engram/global.db

# Add to shell profile
echo 'export ENGRAM_DB_PATH=~/.engram/global.db' >> ~/.zshrc
source ~/.zshrc

# In each project, initialize with scope
cd /path/to/project
engram init --scope "project:$(pwd)"
```

### 4. Verify installation

Start Claude Code in any project:

```bash
cd /some/other/project
claude-code
```

You should see the SessionStart hook output:

```
📚 Engram Memories Available:

No memories found.

Run 'engram tap <id>' to mark memories as used
Run 'engram add "content"' to store new learnings
```

## Troubleshooting

**"engram: command not found"**

Add engram to your PATH in `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

**Hooks not running**

Verify hooks are executable:

```bash
ls -la ~/.claude/hooks/
```

Check Claude Code recognized the hooks:

```
/hooks
```

## See also

- **tutorials/getting-started-claude-hooks.md** - First-time setup
- **how-to/customize-hook-scripts.md** - Modify hook behavior
- **reference/claude-hooks-reference.md** - Configuration options
