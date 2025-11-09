# Publishing to crates.io

Guide for publishing Praxis crates to crates.io.

## Prerequisites

1. **Create crates.io account**: https://crates.io/
2. **Get API token**: https://crates.io/me
3. **Login to cargo**:
   ```bash
   cargo login <your-api-token>
   ```

## Publication Order

Crates must be published in dependency order:

```
1. praxis-llm       (no internal deps)
2. praxis-types     (depends on: praxis-llm)
3. praxis-mcp       (depends on: praxis-llm)
4. praxis-persist   (depends on: praxis-llm, praxis-types)
5. praxis-graph     (depends on: praxis-types, praxis-llm, praxis-mcp)
```

## Commands

### Dry Run (test before publishing)

```bash
# Test each crate
cargo publish --dry-run -p praxis-llm
cargo publish --dry-run -p praxis-types
cargo publish --dry-run -p praxis-mcp
cargo publish --dry-run -p praxis-persist
cargo publish --dry-run -p praxis-graph
```

### Actual Publication

**IMPORTANT:** Once published, versions cannot be unpublished (only yanked). Double-check everything!

```bash
# Publish in order
cargo publish -p praxis-llm
cargo publish -p praxis-types
cargo publish -p praxis-mcp
cargo publish -p praxis-persist
cargo publish -p praxis-graph
```

**Note:** Wait a few minutes between publications for crates.io to index each crate before publishing the next one that depends on it.

## Pre-Publication Checklist

- [ ] All tests pass: `cargo test --workspace`
- [ ] All crates compile: `cargo check --workspace`
- [ ] README.md exists in all crates
- [ ] Cargo.toml has complete metadata:
  - [ ] description
  - [ ] keywords (max 5)
  - [ ] categories
  - [ ] license
  - [ ] repository
  - [ ] homepage
  - [ ] documentation
  - [ ] readme
- [ ] All internal dependencies have versions specified
- [ ] Git is clean (committed)
- [ ] Version numbers are correct (0.1.0)

## After Publication

1. **Verify on crates.io:**
   - https://crates.io/crates/praxis-llm
   - https://crates.io/crates/praxis-types
   - https://crates.io/crates/praxis-mcp
   - https://crates.io/crates/praxis-persist
   - https://crates.io/crates/praxis-graph

2. **Test installation:**
   ```bash
   cargo new test-praxis
   cd test-praxis
   cargo add praxis-graph praxis-llm praxis-mcp praxis-types praxis-persist
   cargo build
   ```

3. **Update main README:**
   ```bash
   # Add crates.io badges
   [![Crates.io](https://img.shields.io/crates/v/praxis-graph.svg)](https://crates.io/crates/praxis-graph)
   ```

4. **Tag release:**
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

5. **Create GitHub Release:**
   - Go to GitHub Releases
   - Create release from tag v0.1.0
   - Add release notes

## Version Updates (Future)

When bumping versions:

1. Update version in workspace `Cargo.toml`
2. Update internal dependency versions in each crate
3. Commit changes
4. Publish in dependency order
5. Tag release

## Troubleshooting

### "error: failed to verify package tarball"
- Run `cargo package --list -p <crate>` to see what's included
- Check .gitignore isn't excluding required files

### "error: crate depends on <crate> but <crate> does not exist"
- The dependency hasn't been published yet
- Publish dependencies first
- Wait for crates.io to index (2-5 minutes)

### "error: version 0.1.0 already exists"
- Can't re-publish same version
- Bump version number
- Or yank old version first (not recommended)

## Notes

- **First publication is permanent** - choose names carefully
- **Versions are permanent** - can't delete, only yank
- **Crates.io takes 2-5 minutes** to index new publications
- **Documentation is auto-generated** from doc comments by docs.rs
- **Examples are included** in the package automatically

## Links

- [crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io Policy](https://crates.io/policies)
- [Semantic Versioning](https://semver.org/)

