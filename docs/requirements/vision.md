# Engram

Observability tool for Claude agent memory.

## Problem

When working with Claude agents, it's hard to understand what they're "learning" and remembering across sessions. What patterns emerge? What facts get used repeatedly? What gets stored but never referenced?

Without visibility into agent memory behavior, you can't:
- Know what should be promoted to permanent docs (CLAUDE.md)
- Debug why an agent keeps making the same mistakes
- Understand what context is actually valuable vs noise

## Core Concept

Engram is a **teaching/observability tool** that helps you understand agent memory patterns. Think of it as a debugger for agent context.

### Event Log

Every memory operation is logged as an immutable event:
- `ADD` - memory created
- `TAP` - memory used
- `REMOVE` - memory deleted
- `EDIT` - memory modified

This event log enables projections and analytics.

### The Workflow

1. **Agent stores memories** during sessions (facts, decisions, patterns)
2. **Agent taps memories** when they inform responses
3. **You observe** what's being stored and used via `engram log`
4. **Patterns emerge** - some memories get tapped repeatedly, others never
5. **You promote** valuable patterns to CLAUDE.md (permanent long-term memory)
6. **You remove** memories that have been promoted or are no longer useful

Engram is the "short-term memory" that informs what goes into "long-term memory" (your docs).

### Tap-Based Value Signal

Memories are ranked by tap_count. More taps = more valuable. This is the only metric that matters:
- 0 taps: Never used, candidate for removal
- Many taps: Frequently useful, candidate for promotion to CLAUDE.md

### Per-Workspace Storage

Each workspace has its own `.engram/` directory with its own memories. No global storage, no scoping complexity.

## Architecture

### Storage

SQLite with WAL mode. Location: `.engram/engram.db` in current workspace.

Two tables:
- `memories` - current state (projection)
- `events` - append-only event log (source of truth)

### CLI

```bash
# Memory operations
engram add "<content>"
engram list
engram show <id>
engram edit <id> "<new content>"
engram remove <id>
engram tap <id>
engram tap --match "<substring>"

# Observability
engram log [--action=<action>] [--limit=<n>]

# Setup
engram init          # Create .engram/ directory
engram prompt        # Output protocol snippet for CLAUDE.md
```

### Agent Integration

Add protocol to CLAUDE.md via `engram prompt >> CLAUDE.md`. The agent then:
1. Runs `engram list` at session start
2. Taps memories when they inform responses
3. Stores new learnings as memories
4. Promotes valuable memories to CLAUDE.md, removes from engram

## Implementation Status

- [x] Core CRUD (add, list, show, edit, remove)
- [x] Tap recording
- [x] Event log
- [x] `engram log` command
- [x] `engram prompt` with full protocol
- [ ] TUI for real-time observability

## Future Vision

A TUI (like lazygit) for real-time observability:
- Watch memories being added/tapped live
- See engagement patterns emerge
- Identify candidates for CLAUDE.md promotion
- Debug agent behavior

## Non-Goals

- Context injection optimization (use CLAUDE.md for that)
- Sync across machines (separate problem)
- Encryption at rest
- Automatic GC (user decides what lives/dies)
