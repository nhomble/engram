#!/bin/sh
# Session start hook for Claude Code
# Runs garbage collection and loads memories into context

set -e

# Run GC silently (clean up low-engagement memories)
engram gc 2>/dev/null || true

# Load memories for this session
engram init --scope global --scope "project:$PWD"
