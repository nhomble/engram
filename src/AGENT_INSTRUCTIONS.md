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

You have rich context - files read, code patterns discovered, errors encountered, dependencies understood. Most of this valuable knowledge dies when your session ends. Be AGGRESSIVE about capturing it.

Store immediately when:
- **User corrects you** → store the correction
- **You discover a non-obvious workflow** → store the steps
- **You hit an error and solve it** → store the fix AND the context
- **You learn a project convention** → store the pattern
- **User states a preference** → store it
- **You read important architecture/design decisions** → store them
- **You discover how components interact** → store the relationship
- **You find configuration patterns** → store them
- **You learn about dependencies/tools used** → store the context
- **You understand why code is structured a certain way** → store it

Examples of rich captures:
```bash
engram add "Auth uses JWT with 15min expiry, refresh tokens in HTTP-only cookie"
engram add "DB migrations run via diesel CLI, check diesel.toml for config"
engram add "Tests mock S3 with minio container, see docker-compose.test.yml"
engram add "User prefers functional style - use iterators over for loops"
engram add "API errors use custom Error enum in src/error.rs with context wrapping"
```

Don't just store WHAT you did - store WHY and HOW the system works.

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
