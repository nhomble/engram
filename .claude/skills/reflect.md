# Reflect - Memory Extraction

Extract learnings from the current session and persist to engram.

## When to Use

Invoke this skill before ending a session where you learned something worth remembering about the project.

## Process

1. **Get existing memories**:
   ```bash
   ENGRAM_DB_PATH=./engram.db ./target/debug/engram list --scope "project:$PWD"
   ```

2. **Review the conversation** for facts worth persisting:
   - Project conventions discovered (naming, structure, patterns)
   - User corrections or preferences
   - Architecture decisions made
   - Gotchas or non-obvious behaviors learned

3. **Compare against existing memories** (fuzzy match):
   - Skip if already captured or substantially similar
   - Update if new info refines an existing memory (remove old, add new)

4. **Store new memories**:
   ```bash
   ENGRAM_DB_PATH=./engram.db ./target/debug/engram add "<fact>" --scope "project:$PWD"
   ```

## Guidelines

- Be concise: store facts, not conversations
- One concept per memory
- Prefer actionable facts over observations
- Don't store sensitive info (keys, passwords, personal data)
- Don't store obvious things derivable from code
