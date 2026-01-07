# Release Process

Engram has two components that need versioning:
1. Binary (Cargo.toml)
2. Plugin (.claude-plugin/plugin.json)

## Cutting a Release

### 1. Bump versions

Update both files with the new version (use semver):

**Cargo.toml**:
```toml
[package]
version = "0.2.0"
```

**. claude-plugin/plugin.json**:
```json
{
  "version": "0.2.0"
}
```

### 2. Run tests

```bash
cargo test
```

### 3. Commit and tag

```bash
git add Cargo.toml .claude-plugin/plugin.json
git commit -m "Bump version to 0.2.0"
git tag v0.2.0
git push origin main --tags
```

### 4. Create GitHub release

```bash
gh release create v0.2.0 --generate-notes
```

Or create manually at https://github.com/nhomble/engram/releases/new

## User Upgrade Process

### Binary
```bash
cargo install --git https://github.com/nhomble/engram --force
```

### Plugin
```bash
/plugin marketplace update
/plugin update engram
```

## Version Strategy

Follow semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes to CLI, database schema, or plugin API
- **MINOR**: New features, backward-compatible
- **PATCH**: Bug fixes, documentation updates

**Keep binary and plugin versions in sync** - they should always match.
