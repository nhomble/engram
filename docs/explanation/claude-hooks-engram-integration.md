# Explanation: Claude Hooks and Engram Integration

Understanding how engram uses Claude Code hooks for agent memory observability.

## What are Claude Hooks?

Claude Code hooks are shell commands that execute automatically at specific points in Claude's lifecycle. They enable external tools to:

- Provide context at session start
- Preserve information through context compaction
- Capture learnings at session end
- React to tool usage in real-time

Hooks are configured in `.claude/settings.json` and run as separate processes that receive JSON input via stdin and output to stderr.

## Why Hooks for Memory?

Engram addresses a fundamental challenge: **Claude agents learn during sessions but don't retain that knowledge**.

Without hooks:
- Users must manually review session transcripts to identify learnings
- Valuable context is lost during context compaction
- Patterns discovered in one session aren't available in the next

With hooks:
- Session start shows relevant memories automatically
- Pre-compact ensures context survives compression
- Session end prompts for capturing new learnings
- Post-tool-use can trigger automatic memory suggestions

## Architecture

### Hook Lifecycle

```
User starts Claude Code session
         ↓
SessionStart hook runs
         ↓
    engram list shows available memories
         ↓
User and Claude work together
         ↓
(Context grows large)
         ↓
PreCompact hook runs
         ↓
    engram prime outputs critical context
         ↓
Claude compacts context (memories preserved)
         ↓
Session continues...
         ↓
User ends session
         ↓
SessionEnd hook runs
         ↓
    Reminder to run: engram add "learnings"
```

### Data Flow

**Input**: JSON via stdin
```json
{
  "session_id": "abc123",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/current/directory",
  "hook_event_name": "SessionStart"
}
```

**Hook Script**: Shell command (bash, python, etc.)
```bash
#!/bin/bash
engram list 2>&1 | head -10
```

**Output**: Text to stderr (visible in Claude)
```
📚 Engram Memories Available:

[engram-abc123] taps:5 | User prefers concise responses
[engram-def456] taps:2 | Always run tests before commit
```

## Hook Types and Use Cases

### SessionStart: Memory Recall

**Purpose**: Surface relevant context at the beginning of a session.

**Why it matters**: Claude doesn't remember previous sessions. SessionStart hooks bridge that gap by showing:
- User preferences discovered in past sessions
- Technical patterns specific to this codebase
- Solutions to recurring problems

**Example workflow**:
1. User starts Claude Code
2. SessionStart hook runs `engram list`
3. Claude sees: "User prefers TypeScript over JavaScript"
4. Claude biases responses accordingly

### PreCompact: Context Preservation

**Purpose**: Prevent information loss during context compression.

**Why it matters**: Claude's context window is large but finite. When it fills up, Claude compacts the transcript to free space. Without intervention, important context can be lost.

**How engram helps**:
1. PreCompact hook runs before compaction
2. Hook executes `engram prime`
3. Critical memory context is output to transcript
4. Claude includes this in compacted summary
5. Memories survive the compaction

**What gets preserved**:
- High-tap-count memories (proven valuable)
- Recently added memories (current session learnings)
- Scoped memories (project-specific context)

### SessionEnd: Learning Capture

**Purpose**: Remind users to record session insights.

**Why it matters**: The best time to capture a learning is immediately after discovering it. SessionEnd prompts ensure valuable patterns don't get forgotten.

**What to capture**:
- User corrections ("Actually, we use X not Y")
- Architecture decisions ("Auth uses JWT tokens")
- Error solutions ("Bug was due to async race condition")
- Workflow preferences ("Run tests in docker, not locally")

### PostToolUse: Automatic Suggestions

**Purpose**: Trigger memory capture based on activity patterns.

**Example**: When a test failure is fixed, suggest capturing the solution.

**How it works**:
1. User runs `cargo test` → fails
2. Claude fixes the code
3. User runs `cargo test` → passes
4. PostToolUse hook detects: "test command succeeded after previous failure"
5. Hook outputs: "Consider: engram add 'solution description'"

**Advanced patterns**:
- Auto-tap memories when relevant files are edited
- Suggest memories based on error messages
- Capture build configuration changes

## Memory Lifecycle

### Creation

Memories are created via `engram add`:

```bash
engram add "User prefers functional programming style"
```

This:
1. Generates a unique ID (e.g., `engram-abc123`)
2. Stores content, timestamp, scope in SQLite
3. Logs the creation event
4. Returns the ID for reference

### Retrieval

Memories are retrieved via `engram list`:

```bash
engram list
```

Output:
```
[engram-abc123] taps:0 | User prefers functional programming style
```

Hook scripts typically:
- Limit output to top 5-10 most relevant
- Filter by scope, tap count, or recency
- Format for easy scanning

### Usage Tracking

When a memory proves useful, tap it:

```bash
engram tap engram-abc123
```

This:
1. Increments the tap counter
2. Updates `last_tapped_at` timestamp
3. Logs the tap event
4. Increases the memory's priority

High tap counts indicate:
- Frequently relevant information
- Patterns worth promoting to CLAUDE.md
- User preferences to preserve

### Promotion

When a memory is tapped repeatedly (e.g., 10+ times), consider promoting it to permanent documentation:

**From engram**:
```
[engram-xyz] taps:15 | Always use cargo build --release for hooks
```

**To CLAUDE.md**:
```markdown
## Build Instructions

Use release builds for hooks to ensure proper performance:

\`\`\`bash
cargo build --release
\`\`\`
```

This creates "long-term memory" that:
- Doesn't require hooks to surface
- Is visible to all contributors
- Becomes part of project knowledge

## Database Design

Engram uses SQLite with two tables:

### memories table (projection)

Current state of all memories:
```sql
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    scope TEXT,
    created_at INTEGER,
    taps INTEGER DEFAULT 0,
    last_tapped_at INTEGER
);
```

Fast queries for listing, filtering, sorting.

### events table (append-only log)

Complete history of all changes:
```sql
CREATE TABLE events (
    event_id INTEGER PRIMARY KEY,
    memory_id TEXT,
    event_type TEXT,
    event_data TEXT,
    timestamp INTEGER
);
```

Enables:
- Audit trail of all memory operations
- Reconstructing memory state at any point
- Analytics on memory usage patterns

**Why both?**: Projection for performance, events for history.

## Scope System

Scopes allow multiple projects to share a database:

```bash
# Initialize with project scope
engram init --scope "project:/Users/me/my-app"

# Add scoped memory
engram add "Uses React 18" --scope "project:/Users/me/my-app"

# List only current project's memories
engram list --scope "project:/Users/me/my-app"
```

**Use cases**:
- Global database at `~/.engram/global.db`
- Per-project memories kept separate
- Shared patterns across related projects

**Without scopes**: Each project has its own `.engram/engram.db` (simpler, recommended for most users).

## Performance Considerations

### WAL Mode

Engram uses SQLite's Write-Ahead Logging:

**Benefits**:
- Concurrent reads don't block writes
- Faster commits
- Better crash recovery

**Trade-offs**:
- Creates `engram.db-wal` and `engram.db-shm` files
- WAL file grows until checkpointed
- More disk I/O

**Optimization**:
```bash
sqlite3 .engram/engram.db "PRAGMA wal_checkpoint(TRUNCATE);"
```

### Hook Timeouts

Hooks must complete quickly to avoid blocking Claude:

**Recommended**:
- SessionStart: 10s (acceptable startup delay)
- PreCompact: 10s (compaction is infrequent)
- SessionEnd: 5s (user is leaving anyway)
- PostToolUse: 5s (runs frequently, must be fast)

**If hooks timeout**:
- Reduce output (use `head -5` instead of full list)
- Optimize database with `VACUUM`
- Increase timeout in settings.json (last resort)

### Memory Count

**Small projects** (< 50 memories): No optimization needed

**Medium projects** (50-200 memories): Filter by scope or tap count

**Large projects** (200+ memories): Consider:
- Removing low-value memories (`taps:0` for 90+ days)
- Splitting into multiple scoped databases
- Showing only `hot` memories (recently tapped)

## Design Principles

### 1. Non-invasive

Hooks run as separate processes, not library code. Benefits:
- Can't crash Claude
- Easy to disable (remove from settings.json)
- Works with any version of Claude Code

### 2. Transparent

All output to stderr is visible to both Claude and user. Benefits:
- No hidden behavior
- User sees what Claude sees
- Easy to debug

### 3. Fail-safe

If engram isn't installed, hooks exit cleanly. Benefits:
- Project works for contributors without engram
- Hooks don't break Claude sessions
- Graceful degradation

### 4. Context-aware

Hooks receive session metadata (cwd, tool names, etc.). Benefits:
- Filter memories by current context
- Auto-tap based on file being edited
- Suggest memories based on errors

## Integration Patterns

### Local Development

Hooks in `.claude/hooks/` + database in `.engram/`:
- Project-specific memories
- Team can share via git (if `.engram/` committed)
- Each contributor builds own memory graph

### Global Tool

Hooks in `~/.claude/hooks/` + database in `~/.engram/global.db`:
- All projects share same hooks
- Memories scoped by project path
- Personal knowledge base across all work

### Hybrid

Global hooks + per-project databases:
- Consistent hook behavior everywhere
- Project-specific memories stay isolated
- Best of both worlds

## Future Possibilities

### Automatic Memory Extraction

PostToolUse hooks that parse:
- User corrections in messages
- Successful error resolutions
- Repeated patterns in commands

### Semantic Search

Replace exact-match filtering with vector embeddings:
- Find conceptually similar memories
- Auto-tap based on semantic relevance
- Cluster related learnings

### Memory Sharing

Export/import memory sets:
- Share team knowledge bases
- Publish domain-specific memory packs
- Merge memories across projects

### Analytics Dashboard

Visualize memory usage:
- Which memories get tapped most
- When memories are created vs used
- Gaps in knowledge coverage

## Comparison to Alternatives

### vs. CLAUDE.md

**CLAUDE.md**: Static, comprehensive, version-controlled project documentation.

**Engram**: Dynamic, session-specific, personal learnings.

**Use together**: Promote high-value engram memories to CLAUDE.md.

### vs. Session Transcripts

**Transcripts**: Complete record of every session.

**Engram**: Curated highlights worth remembering.

**Use together**: Review transcripts to identify patterns worth capturing in engram.

### vs. Git History

**Git history**: What changed and when.

**Engram**: Why it changed and what we learned.

**Use together**: Reference git commits from memories, capture commit rationale.

## Benefits

**For users**:
- Don't repeat yourself across sessions
- Build personal knowledge base
- Capture "aha moments" immediately

**For teams**:
- Share project-specific patterns
- Onboard new members faster
- Document tribal knowledge

**For Claude**:
- Maintains continuity across sessions
- Learns user preferences
- Provides better, more contextual responses

## See also

- **tutorials/getting-started-claude-hooks.md** - First-time setup
- **how-to/customize-hook-scripts.md** - Modify hook behavior
- **reference/claude-hooks-reference.md** - Configuration reference
