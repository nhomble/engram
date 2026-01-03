## Engram Memory Protocol

Your context dies when the session ends. Memories are how insights survive for future sessions.

### Session Start

```bash
engram list    # ALWAYS run this first - see what past sessions learned
```

Review each memory. If it's relevant to your current task, tap it:
```bash
engram tap <id>
```

### When to Store (triggers)

Store immediately when:
- **User corrects you** → store the correction
- **You discover a non-obvious workflow** → store the steps
- **You hit an error and solve it** → store the fix
- **You learn a project convention** → store the pattern
- **User states a preference** → store it

```bash
engram add "always run cargo test before committing"
engram add "user wants brief responses, no preamble"
```

### When to Tap

Tap when a memory informs your work - even partially:
```bash
engram tap <id>
engram tap --match "test"   # tap by content match
```

### When NOT to Store

- Already in CLAUDE.md or project docs (don't duplicate)
- Obvious things any agent would know
- One-time facts unlikely to matter again

### Promotion Flow

When a memory gets many taps, it's proven valuable. User may ask you to promote it:

```bash
engram promote <id>   # outputs markdown for CLAUDE.md
```

Then append the output to CLAUDE.md under `## Learned Facts`.

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
