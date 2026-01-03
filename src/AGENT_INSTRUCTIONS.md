## Engram Memory Protocol

Your context dies when the session ends. Memories are how insights survive for future sessions.

# 🧠 AUTONOMOUS MEMORY CAPTURE PROTOCOL 🧠

**CRITICAL**: Store memories AUTONOMOUSLY during work - don't wait for user to ask.

**Core Rule**: Capture knowledge in `engram`, not in code comments, TODOs, or mental notes.

## Session Start Checklist

```bash
engram list    # ALWAYS run this first - see what past sessions learned
```

Review each memory. If relevant, tap it:
```bash
engram tap <id>
engram tap --match "keyword"  # tap multiple by pattern
```

### When to Store (triggers)

You have rich context - files read, code patterns discovered, errors encountered, dependencies understood. Most of this valuable knowledge dies when your session ends. **AUTONOMOUSLY** capture it **AS IT HAPPENS**.

**IMMEDIATELY** store when:
- **User corrects you** → store the correction
- **You discover a non-obvious workflow** → store the steps
- **You hit an error and solve it** → store the fix AND the context
- **You learn a project convention** → store the pattern
- **User states a preference** → store it
- **🏗️ ARCHITECTURE DISCUSSIONS** → **CRITICAL - store layering, patterns, design decisions**
- **You discover how components interact** → store the relationship (don't just think it, STORE IT)
- **You find configuration patterns** → store them
- **You learn about dependencies/tools used** → store the context
- **You understand why code is structured a certain way** → store it
- **You refactor code structure** → store the new architecture
- **You establish naming conventions** → store them
- **You make technology choices** → store the decision AND rationale

Examples of rich captures:
```bash
engram add "Auth uses JWT with 15min expiry, refresh tokens in HTTP-only cookie"
engram add "DB migrations run via diesel CLI, check diesel.toml for config"
engram add "Tests mock S3 with minio container, see docker-compose.test.yml"
engram add "User prefers functional style - use iterators over for loops"
engram add "API errors use custom Error enum in src/error.rs with context wrapping"
```

Don't just store WHAT you did - store WHY and HOW the system works.

## 🧠 MEMORY CAPTURE CHECKLIST 🧠

Before finishing a significant task, verify you captured:

```
[ ] Architecture decisions or layer interactions
[ ] Error solutions with full context
[ ] User corrections or preferences stated
[ ] Non-obvious workflows discovered
[ ] Configuration patterns learned
[ ] Technology/library choices made
```

**If any box is unchecked but applicable, STOP and store the memory NOW.**

### When to Tap

**Tap in real-time as you work** - don't wait until end of session:

- **Before following a workflow** → tap the memory first, then follow it
- **When making a decision informed by a memory** → tap immediately
- **When a memory prevents an error** → tap it (it just saved you)
- **When explaining something you learned from a memory** → tap it

```bash
# Example: Following stored workflow
engram tap 9e7f2f55  # "Development workflow: create branch..."
git checkout -b new-feature  # Now following the workflow

# Or tap by pattern match
engram tap --match "test"
```

**Tapping is active acknowledgment** - like marking a task `in_progress`. Do it when the memory is actively informing your work, not retrospectively.

### When NOT to Store

- Already in CLAUDE.md or project docs (don't duplicate)
- Obvious things any agent would know
- One-time facts unlikely to matter again

### Promotion Flow

When user asks you to promote a memory:

1. Run `engram promote <id>` - outputs markdown bullet
2. Immediately append the output to CLAUDE.md under `## Learned Facts`
3. No additional confirmation needed - just do it

Engram = working memory. CLAUDE.md = permanent knowledge.

### Forgetting

When a memory is wrong or stale:
```bash
engram forget <id>    # mark as discarded
```

### What Makes a Good Memory

Self-contained and actionable. A future agent with no context should understand.

| Bad | Good |
|-----|------|
| "Uses Divio" | "Docs follow Divio: tutorials, how-to, reference, explanation separate" |
| "Prefers short" | "User wants concise responses, no preamble" |
| "Fixed the bug" | "OAuth callback must use HTTPS in production, not HTTP" |

### Commands Reference

```bash
engram list              # show all memories (run at session start!)
engram add "content"     # store new memory
engram tap <id>          # signal you used a memory
engram tap --match "X"   # tap memories matching X
engram show <id>         # view memory details
engram edit <id> "new"   # update content
engram promote <id>      # graduate to CLAUDE.md (terminal)
engram forget <id>       # discard as stale/wrong (terminal)
engram log               # view event stream
```
