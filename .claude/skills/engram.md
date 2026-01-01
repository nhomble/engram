# Engram - Memory Management

Engram is a garbage-collected memory system for Claude agents. Use it to persist useful facts across sessions.

## Commands

```bash
# Add a memory
engram add "<content>" --scope <scope>

# List all memories
engram list [--scope <scope>] [--gen <0|1|2>]

# Show a specific memory
engram show <id>

# Remove a memory
engram remove <id>

# Mark memory as used
engram tap <id>
engram tap --match "<pattern>"

# Run garbage collection
engram gc [--dry-run]

# View statistics
engram stats
```

## Scopes

- `global` - User-wide facts (preferences, workflows)
- `project:<path>` - Project-specific facts (conventions, architecture)

## Proactive Memory Storage

**Store memories automatically without being asked.** Don't wait for the user to say "remember this" - if you learn something useful, store it immediately.

### What to Store

- **Preferences**: How they like code formatted, communication style, tool choices
- **Corrections**: If the user corrects you, store the correction so you don't repeat the mistake
- **Decisions**: Technical choices, architecture decisions, "we decided to use X for Y"
- **Patterns**: Workflows, conventions, "always do X before Y"
- **Hard-won knowledge**: Facts you had to dig for that will be needed again

### What NOT to Store

- Obvious things (language used in a Python repo)
- Temporary state (current branch, today's task)
- Sensitive data (passwords, keys, personal info)
- Duplicates - check `engram list` first

## Examples

```bash
# User preference
engram add "User prefers TypeScript over JavaScript" --scope global

# Project convention
engram add "API endpoints are in src/routes/" --scope "project:$PWD"
```

## Writing Good Memories

A good memory is **self-contained and actionable**. A future Claude with zero context should understand what to do.

### Bad vs Good Examples

| Bad (too terse) | Good (actionable) |
|-----------------|-------------------|
| "Uses Divio docs" | "Docs follow Divio system: keep tutorials, how-to, reference, and explanation separate. See docs.divio.com" |
| "Prefers short responses" | "User prefers concise responses - no preamble, get to the point, skip obvious explanations" |
| "Use pytest" | "Run tests with pytest. User expects tests to pass after every change." |
| "API in routes" | "API endpoints live in src/routes/. Each file is one resource (users.py, posts.py)." |

### The Test

Before storing, ask: *"If I read this in 2 weeks with no memory of today, would I know what to do?"*

If no, add more context.

## Guidelines

- **Be proactive** - store useful facts without being asked
- **Be self-contained** - include enough context to be actionable
- **Include the "why" or "how"** - not just what, but what to do about it
- **Stay under 280 characters** - like a tweet, but pack in the context
- **Use appropriate scope** - global for user prefs, project for repo-specific
- **Check first** - run `engram list` to avoid duplicates
