# engram

Memory observability for Claude agents. Store what you learn, tap what you use, promote what matters.

## Install

```bash
cargo install --git https://github.com/nhomble/engram
```

## Claude Code Integration

Auto-configure hooks for Claude Code sessions:

```bash
# In Claude Code
/plugin marketplace add nhomble/engram
/plugin install engram
```

See [docs/claude-code-hooks.md](docs/claude-code-hooks.md) for details.

## Basic Usage

```bash
engram init              # Initialize engram in your project
engram add "content"     # Store a learning
engram list              # View memories
engram tap <id>          # Mark memory as used
engram ui                # Launch TUI
```

See `engram --help` for all commands.

