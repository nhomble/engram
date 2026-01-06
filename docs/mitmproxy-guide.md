# Mitmproxy Integration Guide

## Overview

The `engram mitmproxy` command watches mitmproxy for Claude API traffic and automatically extracts memories from conversations. This provides autonomous memory capture without requiring the working agent to self-report.

## Architecture

```
┌──────────────┐
│ Claude CLI   │ ← User runs Claude with proxy configured
└──────┬───────┘
       │ HTTP/HTTPS via proxy
       ▼
┌──────────────┐
│ mitmproxy    │ ← Intercepts and decrypts HTTPS traffic
│ (web mode)   │
└──────┬───────┘
       │ HTTP API
       ▼
┌──────────────┐
│ engram       │ ← Polls /flows endpoint
│ mitmproxy    │
└──────┬───────┘
       │ Batched requests
       ▼
┌──────────────┐
│ Claude Haiku │ ← Analyzes conversation for memories
│ (analyzer)   │
└──────┬───────┘
       │ engram add commands
       ▼
┌──────────────┐
│ Engram DB    │ ← Stores extracted memories
└──────────────┘
```

## Setup

### 1. Install mitmproxy

```bash
# macOS
brew install mitmproxy

# Linux
pip install mitmproxy

# Windows
pip install mitmproxy
```

### 2. Trust mitmproxy's CA certificate

**First run** (to generate certificate):
```bash
mitmproxy
# Exit with Ctrl+C
```

**macOS:**
```bash
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.mitmproxy/mitmproxy-ca-cert.pem
```

**Linux:**
```bash
sudo cp ~/.mitmproxy/mitmproxy-ca-cert.pem /usr/local/share/ca-certificates/mitmproxy.crt
sudo update-ca-certificates
```

**Windows:**
Import `%USERPROFILE%\.mitmproxy\mitmproxy-ca-cert.p12` to "Trusted Root Certification Authorities"

### 3. Start mitmproxy in web mode

```bash
mitmweb --web-host localhost --web-port 8081 --listen-port 8080
```

This starts:
- Proxy on port 8080 (for traffic interception)
- Web interface on http://localhost:8081 (for API access)

### 4. Start engram watcher

In another terminal:

```bash
engram mitmproxy
```

Or with custom options:

```bash
engram mitmproxy --url http://localhost:8081 --interval 5 --batch 10
```

Options:
- `--url`: Mitmproxy web API URL (default: `http://localhost:8081`)
- `--interval`: Poll interval in seconds (default: `5`)
- `--batch`: Number of requests before running analyzer (default: `10`)

### 5. Run Claude through the proxy

```bash
HTTP_PROXY=http://localhost:8080 HTTPS_PROXY=http://localhost:8080 claude
```

Or set globally:

```bash
export HTTP_PROXY=http://localhost:8080
export HTTPS_PROXY=http://localhost:8080

claude  # Now proxied automatically
```

## Usage

Once everything is running:

1. Use Claude normally
2. The watcher polls mitmproxy every 5 seconds
3. When 10 Claude API requests accumulate, the analyzer runs
4. Memories are automatically stored in engram
5. View memories: `engram list`

## Example Session

```bash
# Terminal 1: Start mitmproxy
$ mitmweb --web-host localhost --web-port 8081 --listen-port 8080

# Terminal 2: Start engram watcher
$ engram mitmproxy
🔍 Starting mitmproxy watcher
   URL: http://localhost:8081
   Poll interval: 5s
   Batch size: 10

# Terminal 3: Use Claude
$ HTTP_PROXY=http://localhost:8080 HTTPS_PROXY=http://localhost:8080 claude
> Help me debug this error...

# Terminal 2 (watcher output):
📥 Fetched 2 new flows
   → Found 1 Claude API requests
   → Captured 342 bytes of JSON

🧠 Running analyzer on 10 requests...
   → Extracted 2 memories
   ✓ Stored: User prefers concise responses (engram-abc123)
   ✓ Stored: OAuth requires HTTPS in production (engram-def456)
```

## Troubleshooting

### Proxy connection refused

**Problem:** `curl: (7) Failed to connect to localhost port 8080`

**Solution:** Ensure mitmproxy is running with `--listen-port 8080`

### TLS verification failed

**Problem:** `SSL: CERTIFICATE_VERIFY_FAILED`

**Solution:** Trust the mitmproxy CA certificate (see Setup step 2)

### Watcher not finding flows

**Problem:** Watcher shows "Fetched 0 new flows" even though you're using Claude

**Solution:**
- Verify proxy is configured: `echo $HTTP_PROXY $HTTPS_PROXY`
- Check mitmweb UI at http://localhost:8081 to see if flows are captured
- Ensure you're using `http://localhost:8081` for the watcher URL (web port, not proxy port)

### Analyzer not running

**Problem:** Requests accumulate but analyzer never runs

**Solution:** Wait for batch size (default: 10 requests) or reduce with `--batch 5`

## Cost Considerations

The analyzer uses Claude Haiku for memory extraction:
- Cost: ~$0.25 per 1M input tokens
- Average: ~10 requests = ~5000 tokens = $0.00125 per batch
- For 100 batches/day: ~$0.125/day

Adjust `--batch` to balance cost vs. responsiveness.

## Comparison with `engram prime`

| Feature | `engram prime` | `engram mitmproxy` |
|---------|----------------|-------------------|
| Setup complexity | None (built-in) | Moderate (proxy + watcher) |
| Agent compliance | Required | Not required |
| Real-time | Yes | Batched (5s poll) |
| Cost | Free | ~$0.125/day |
| Reliability | Agent-dependent | Infrastructure-dependent |
| Coverage | Only compliant agents | All Claude API traffic |

**When to use `engram prime`:** Quick setup, testing, agent is reliable

**When to use `engram mitmproxy`:** Production use, untrusted agents, want guaranteed capture

## Advanced: Filtering Specific Endpoints

Edit `src/mitm/client.rs` to customize which endpoints are captured:

```rust
pub fn filter_claude_flows(flows: &[Flow]) -> Vec<Flow> {
    flows
        .iter()
        .filter(|flow| {
            flow.request
                .as_ref()
                .map(|req| {
                    // Only capture /v1/messages endpoint
                    req.host.contains("anthropic.com") && req.path.contains("/v1/messages")
                })
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}
```

## Advanced: Custom Analyzer Prompts

Edit `src/mitm/analyzer.rs` to customize what the analyzer looks for:

```rust
const ANALYZER_PROMPT_TEMPLATE: &str = r#"You are a memory extraction agent.

Focus on:
- Technical decisions and trade-offs
- Error patterns and solutions
- User preferences and constraints

For each memory, output ONLY:
engram add "concise fact"
"#;
```

Rebuild: `cargo build --release && cp target/release/engram ~/.local/bin/`
