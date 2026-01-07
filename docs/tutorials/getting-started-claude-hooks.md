# Tutorial: Getting Started with Claude Hooks

This tutorial walks you through setting up engram's Claude Code hooks for the first time.

## What you'll learn

By the end of this tutorial, you'll have:
- Installed the engram CLI tool
- Initialized an engram database
- Verified that hooks run automatically
- Added and viewed your first memory

## Prerequisites

- Rust and Cargo installed
- Claude Code CLI
- This engram project cloned locally

## Step 1: Build and install engram

Build the release binary:

```bash
cargo build --release
```

Copy it to your PATH:

```bash
cp target/release/engram ~/.local/bin/
```

Verify installation:

```bash
engram --help
```

You should see the engram command reference.

## Step 2: Initialize the database

Create the engram database in this project:

```bash
engram init
```

This creates `.engram/engram.db` in your current directory.

## Step 3: Test hooks manually

The hooks are already configured in `.claude/hooks/`. Test each one:

```bash
bash .claude/hooks/session-start.sh
```

Expected output:
```
📚 Engram Memories Available:

No memories found.

Run 'engram tap <id>' to mark memories as used
Run 'engram add "content"' to store new learnings
```

Test the other hooks:

```bash
bash .claude/hooks/pre-compact.sh
bash .claude/hooks/session-end.sh
```

## Step 4: Add your first memory

Store a learning:

```bash
engram add "User prefers concise documentation"
```

List memories:

```bash
engram list
```

You'll see output like:
```
[engram-abc123] taps:0 | User prefers concise documentation
```

## Step 5: Start a Claude session

The hooks are configured in `.claude/settings.json` and run automatically.

When you start Claude Code in this project, you should see:

```
📚 Engram Memories Available:

[engram-abc123] taps:0 | User prefers concise documentation

Run 'engram tap <id>' to mark memories as used
Run 'engram add "content"' to store new learnings
```

## Step 6: Mark a memory as used

When a memory is relevant, tap it:

```bash
engram tap engram-abc123
```

This increments the tap count, helping you identify valuable memories.

## Next steps

Now that hooks are working:

- See **how-to/customize-hook-scripts.md** to modify hook behavior
- See **how-to/install-hooks-globally.md** to use hooks in all projects
- See **reference/claude-hooks-reference.md** for configuration details
- See **explanation/claude-hooks-engram-integration.md** to understand the architecture
