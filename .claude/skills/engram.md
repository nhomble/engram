# Engram - Agent Memory Observability

Engram tracks what you learn across sessions. Store facts, tap when you use them, and patterns emerge over time. The user observes these patterns to understand what should be promoted to permanent docs (CLAUDE.md).

## Commands

```bash
# Store a memory
engram add "<content>" --scope "project:$PWD"

# Mark a memory as used (when you reference it)
engram tap <id>
engram tap --match "<pattern>"

# View memories
engram list
engram show <id>

# View event log (observability)
engram log
engram log --action TAP --limit 50

# Statistics
engram stats

# Cleanup (run periodically)
engram gc
```

## When to Store

Store a memory when you learn something the user would want to know you're tracking:

- **Corrections**: User corrected you → store so you don't repeat
- **Decisions**: "We decided to use X for Y"
- **Preferences**: How user likes things done
- **Patterns**: Workflows, conventions discovered
- **Hard-won knowledge**: Facts you had to dig for

## When to Tap

Tap a memory when you actively use it to inform your response:

```bash
# You remembered their testing preference
engram tap --match "tests"

# You used a specific decision
engram tap abc123
```

Tapping signals value. Memories that get tapped survive GC; unused ones decay.

## Writing Good Memories

A good memory is **self-contained and actionable**. A future Claude with zero context should understand what to do.

| Bad | Good |
|-----|------|
| "Uses Divio docs" | "Docs follow Divio system: keep tutorials, how-to, reference, explanation separate" |
| "Prefers short" | "User prefers concise responses - no preamble, skip obvious explanations" |
| "Use pytest" | "Run tests with pytest. User expects tests to pass after every change." |

**The test**: If you read this in 2 weeks with no memory of today, would you know what to do?

## Guidelines

- **Be proactive** - store useful facts without being asked
- **Tap when you use** - this signals value to the user
- **Stay under 280 chars** - concise but complete
- **Default to project scope** - use `--scope "project:$PWD"`
- **Check first** - run `engram list` to avoid duplicates
