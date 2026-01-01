# How to Install the Engram CLI

This guide walks you through building and installing the `engram` command-line tool.

## Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))

## Steps

1. Clone the repository:

```bash
git clone https://github.com/yourusername/engram.git
cd engram
```

2. Build the release binary:

```bash
cargo build --release
```

3. Copy the binary to a directory in your PATH:

```bash
cp target/release/engram ~/.local/bin/
```

4. Verify the installation:

```bash
engram --help
```

You should see the available commands listed.

## Next steps

- [Set up Claude Code hooks](install-hooks.md) to automatically load memories on session start
