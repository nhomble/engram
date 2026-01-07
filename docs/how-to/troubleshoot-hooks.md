# How-to: Troubleshoot Hooks

Diagnose and fix common Claude hooks issues.

## Hooks not running

**Check if hooks are registered**

In Claude Code, run:

```
/hooks
```

This shows all configured hooks. You should see entries for SessionStart, PreCompact, and SessionEnd.

**Verify hook scripts exist**

```bash
ls -la .claude/hooks/
```

Expected output:
```
-rwxr-xr-x session-start.sh
-rwxr-xr-x pre-compact.sh
-rwxr-xr-x session-end.sh
```

If permissions are wrong:

```bash
chmod +x .claude/hooks/*.sh
```

**Test hooks manually**

Run each hook script directly:

```bash
bash .claude/hooks/session-start.sh
```

If this fails, the problem is in the script. If it succeeds, the problem is in Claude Code configuration.

**Check settings.json syntax**

Validate `.claude/settings.json`:

```bash
cat .claude/settings.json | python3 -m json.tool
```

If JSON is invalid, fix syntax errors.

## "engram: command not found"

**Verify engram is installed**

```bash
which engram
```

If not found, install it:

```bash
cargo build --release
cp target/release/engram ~/.local/bin/
```

**Add to PATH**

Edit your shell profile (`~/.zshrc` or `~/.bashrc`):

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Reload:

```bash
source ~/.zshrc
```

**Use absolute path in hooks**

Edit hook scripts to use full path:

```bash
#!/bin/bash
# session-start.sh
$HOME/.local/bin/engram list
```

## Hooks timing out

**Increase timeout in settings**

Edit `.claude/settings.json`:

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
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

**Optimize database**

Run SQLite vacuum:

```bash
sqlite3 .engram/engram.db "VACUUM;"
```

**Reduce output**

Limit the number of memories shown:

```bash
#!/bin/bash
# session-start.sh
engram list 2>&1 | head -5  # Only show 5 memories
```

## Database not found

**Initialize engram**

```bash
engram init
```

**Check database path**

Verify `.engram/engram.db` exists:

```bash
ls -la .engram/
```

**Set explicit path**

Set environment variable:

```bash
export ENGRAM_DB_PATH=/path/to/engram.db
```

Add to shell profile for persistence.

## Hook output not visible

**Check stderr vs stdout**

Hooks output to stderr to be visible in Claude Code:

```bash
#!/bin/bash
echo "This shows up" >&2
echo "This might not" >&1
```

**Verify hook exit code**

Hooks must exit 0 for success:

```bash
#!/bin/bash
engram list 2>&1
exit 0  # Always exit 0
```

## Hooks running but not showing memories

**Check if database has memories**

```bash
engram list
```

If empty, add a test memory:

```bash
engram add "Test memory"
```

**Verify hook output format**

Run hook manually and check output:

```bash
bash .claude/hooks/session-start.sh
```

## Permission denied errors

**Make scripts executable**

```bash
chmod +x .claude/hooks/*.sh
```

**Check file ownership**

```bash
ls -la .claude/hooks/
```

If owned by wrong user:

```bash
sudo chown $USER .claude/hooks/*.sh
```

## Global hooks not working

**Verify $HOME vs $CLAUDE_PROJECT_DIR**

Global hooks must use `$HOME`:

```json
{
  "type": "command",
  "command": "bash $HOME/.claude/hooks/session-start.sh"
}
```

Project hooks use `$CLAUDE_PROJECT_DIR`:

```json
{
  "type": "command",
  "command": "bash $CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh"
}
```

**Check global settings location**

Global settings should be at:

```
~/.claude/settings.json
```

Not:

```
./.claude/settings.json
```

## Database locked errors

**Close other connections**

Check for other engram processes:

```bash
ps aux | grep engram
```

Kill if necessary:

```bash
pkill engram
```

**Check WAL files**

```bash
ls -la .engram/
```

If `engram.db-wal` or `engram.db-shm` are large, run:

```bash
sqlite3 .engram/engram.db "PRAGMA wal_checkpoint(TRUNCATE);"
```

## See also

- **tutorials/getting-started-claude-hooks.md** - Basic setup
- **how-to/install-hooks-globally.md** - Global installation
- **reference/claude-hooks-reference.md** - Configuration reference
