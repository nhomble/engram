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
- `REVIEW` - memory shown to agent
- `EXPIRE` - memory GC'd
- `PROMOTE` - memory promoted to higher generation

This event log enables projections and analytics.

### The Workflow

1. **Agent stores memories** during sessions (facts, decisions, patterns)
2. **You observe** what's being stored and used via `engram log`, `engram stats`
3. **Patterns emerge** - some memories get tapped repeatedly, others never
4. **You promote** valuable patterns to CLAUDE.md (permanent long-term memory)
5. **GC cleans up** low-engagement memories automatically

Engram is the "short-term memory" that informs what goes into "long-term memory" (your docs).

### Generational Model

Memories have generations based on engagement:
- **Gen0**: New, untested. Expires if not tapped.
- **Gen1**: Survived multiple taps. Candidate for CLAUDE.md.
- **Gen2**: Heavily used. Definitely belongs in permanent docs.

### Scoping

Memories are scoped to projects:
- `project:<path>` — project-specific facts (default)
- `global` — rare, truly universal preferences

## Architecture

### Storage

SQLite with WAL mode. Location: `~/.engram/engram.db`

Two tables:
- `memories` - current state (projection)
- `events` - append-only event log (source of truth)

### CLI

```bash
# Memory operations
engram add "<content>" --scope "project:$PWD"
engram list [--scope=<scope>] [--gen=<0|1|2>]
engram show <id>
engram remove <id>
engram tap <id>

# Observability
engram log [--action=<action>] [--limit=<n>]
engram stats

# Maintenance
engram gc [--dry-run]
```

### Claude Hooks Integration

Session start hook runs GC and loads memories:
```bash
engram gc 2>/dev/null
engram init --scope global --scope "project:$PWD"
```

### Projections (Future)

SQL views over the event log:
- Hot memories (most tapped recently)
- Activity timeline
- Engagement ratios
- Decay candidates

## Implementation Status

- [x] Core CRUD (add, list, show, remove)
- [x] Tap recording
- [x] GC with ratio-based expiry
- [x] Event log
- [x] `engram log` command
- [ ] Projections/views
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
