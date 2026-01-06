# Engram MITM Proxy Setup Guide

## Overview

The Engram MITM (Man-in-the-Middle) proxy provides an alternative approach to memory capture that doesn't rely on the working agent to self-interrogate. Instead, it intercepts Claude API traffic and uses a dedicated analyzer agent to extract memories.

## Why MITM vs `engram prime`?

### `engram prime` Approach

- **How it works**: The working agent is instructed via AGENT_INSTRUCTIONS.md to capture its own memories
- **Pros**: Zero additional infrastructure, works locally
- **Cons**: Depends on agent compliance, agents may forget to capture memories

### MITM Proxy Approach

- **How it works**: Proxy intercepts Claude API traffic → analyzer agent reviews conversation → extracts memories
- **Pros**: Independent of working agent, dedicated analyzer is more thorough
- **Cons**: Requires proxy process, analyzer API calls cost money, batched (not real-time)

### When to Use Which?

**Use `engram prime` when:**
- Working with Claude Code or Claude CLI
- Want zero additional infrastructure
- Agent is compliant and reliable
- Real-time memory capture is preferred

**Use `engram-mitm` when:**
- Working agent frequently forgets to store memories
- Want independent verification of learnings
- Willing to run background proxy
- Cost of analyzer calls is acceptable

## Installation

### 1. Build the Binary

```bash
cargo build --release --bin engram-mitm
cp target/release/engram-mitm ~/.local/bin/
```

### 2. Generate TLS Certificate (First Run)

On first run, `engram-mitm` will auto-generate a self-signed CA certificate:

```bash
engram-mitm
```

You'll see output like:

```
=== Engram MITM CA Certificate Generated ===
Certificate: /Users/yourname/.engram-mitm/ca.crt

To enable HTTPS interception, trust this certificate:
...
```

### 3. Trust the Certificate

**macOS:**
```bash
sudo security add-trusted-cert -d -r trustRoot \
  -k /Library/Keychains/System.keychain \
  ~/.engram-mitm/ca.crt
```

**Linux:**
```bash
sudo cp ~/.engram-mitm/ca.crt /usr/local/share/ca-certificates/engram-mitm.crt
sudo update-ca-certificates
```

**Windows:**
Import `~/.engram-mitm/ca.crt` to "Trusted Root Certification Authorities" via certmgr.msc

## Configuration

All configuration is via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `ENGRAM_MITM_PORT` | Proxy listen port | 8080 |
| `ENGRAM_MITM_BATCH_SIZE` | Messages before analysis | 10 |
| `ENGRAM_MITM_INTERVAL` | Seconds before analysis | 300 (5 min) |
| `ENGRAM_MITM_BUFFER_SIZE` | Max messages in buffer | 50 |
| `ENGRAM_DB_PATH` | Engram database path | `~/.engram/engram.db` |

Example:

```bash
export ENGRAM_MITM_PORT=9090
export ENGRAM_MITM_BATCH_SIZE=20
export ENGRAM_MITM_INTERVAL=600
engram-mitm
```

## Usage

### Terminal 1: Start Proxy

```bash
engram-mitm
```

Output:
```
Engram MITM - Memory extraction proxy

Configuration:
  Port: 8080
  Batch size: 10 messages
  Interval: 300s
  Buffer size: 50 messages

Engram MITM proxy listening on http://127.0.0.1:8080
Configure clients: HTTP_PROXY=http://127.0.0.1:8080 HTTPS_PROXY=http://127.0.0.1:8080
```

### Terminal 2: Use Claude Through Proxy

```bash
HTTP_PROXY=http://localhost:8080 HTTPS_PROXY=http://localhost:8080 claude
```

or with Claude Code:

```bash
HTTP_PROXY=http://localhost:8080 HTTPS_PROXY=http://localhost:8080 claude-code
```

### Memory Extraction

The proxy will automatically:
1. Buffer conversation messages
2. Analyze every N messages or M seconds (configurable)
3. Spawn `claude --model haiku` to extract memories
4. Add extracted memories to your engram database

Example analyzer output:

```
=== Analyzing conversation (15 messages) ===
Extracted 3 memories:
  1. User prefers concise responses without preamble
  2. Always run cargo test before creating commits
  3. Project uses beads for issue tracking
Added 3 memories to engram
  → engram-abc123
  → engram-def456
  → engram-ghi789
===========================================
```

## Troubleshooting

### Proxy connection refused

**Symptom**: Claude CLI fails with "connection refused"

**Fix**: Ensure `engram-mitm` is running before starting Claude

### Certificate trust issues

**Symptom**: SSL/TLS errors when accessing HTTPS sites

**Fix**: Follow certificate trust steps for your OS (see Installation section)

### No memories being extracted

**Symptom**: Analyzer runs but extracts 0 memories

**Possible causes**:
- Conversation too short (wait for batch_size messages)
- No learnings in conversation content
- Analyzer prompt needs tuning

**Debug**: Check analyzer stderr in proxy output

### Analyzer errors

**Symptom**: "Analysis failed" errors in proxy output

**Possible causes**:
- `claude` CLI not in PATH
- API key issues
- Rate limiting

**Fix**: Verify `claude --model haiku` works independently

## Cost Considerations

The analyzer uses Claude Haiku for memory extraction. Costs depend on:

- **Batch size**: Smaller batches → more API calls → higher cost
- **Conversation length**: Longer conversations → larger prompts → higher cost
- **Frequency**: More frequent analysis → more API calls → higher cost

**Cost optimization tips**:

1. Increase `ENGRAM_MITM_BATCH_SIZE` (e.g., 20-30 messages)
2. Increase `ENGRAM_MITM_INTERVAL` (e.g., 600-900 seconds)
3. Reduce `ENGRAM_MITM_BUFFER_SIZE` to minimize context sent to analyzer

**Example costs** (as of 2026, approximate):

- 10 messages every 5 minutes = ~12 analyses/hour = ~$0.10-0.20/hour
- 30 messages every 10 minutes = ~6 analyses/hour = ~$0.05-0.10/hour

Check current Anthropic pricing for exact rates.

## Advanced Usage

### Running in Background

```bash
engram-mitm > /tmp/engram-mitm.log 2>&1 &
```

### System-wide Proxy

Add to `~/.bashrc` or `~/.zshrc`:

```bash
export HTTP_PROXY=http://localhost:8080
export HTTPS_PROXY=http://localhost:8080
```

Then all CLI tools will route through the proxy.

### Selective Proxying

Only proxy specific commands:

```bash
alias claude-mitm='HTTP_PROXY=http://localhost:8080 HTTPS_PROXY=http://localhost:8080 claude'
```

## Architecture

```
┌──────────────┐
│ Claude CLI / │
│  IDE         │
└──────┬───────┘
       │ API calls
       ▼
┌──────────────┐
│ Proxy/MITM   │ ← captures requests/responses
└──────┬───────┘
       │ conversation log
       ▼
┌──────────────┐
│ Memory       │ ← separate Claude instance
│ Analyzer     │    (haiku model)
│ Agent        │
└──────┬───────┘
       │ engram add commands
       ▼
┌──────────────┐
│ Engram DB    │ ← same database as main engram
└──────────────┘
```

## Limitations (MVP)

Current implementation is an MVP with the following limitations:

1. **HTTPS interception not yet implemented**: Proxy rejects CONNECT requests
2. **Only logs Claude API traffic**: Doesn't forward requests yet
3. **No request/response parsing**: Doesn't extract actual messages yet
4. **Placeholder analyzer**: Always returns empty memories

These will be addressed in future iterations. The infrastructure is in place for:

- TLS certificate generation ✅
- Conversation buffering ✅
- Analyzer agent spawning ✅
- Memory extraction parsing ✅

## See Also

- [Engram README](../../README.md)
- [CLAUDE.md](../../CLAUDE.md) - Memory protocol guide
- [Beads Workflow](https://github.com/nhomble/beads) - Issue tracking
