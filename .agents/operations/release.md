# Release Process

Step-by-step guide for creating and publishing a new doxx release.

## Pre-Release Checklist

### 1. Verify All Tests Pass
```bash
./scripts/check.sh
```

Ensure CI/CD is green on main branch.

### 2. Review Changes Since Last Release
```bash
git log v0.1.2..HEAD --oneline
```

Identify features, fixes, and breaking changes.

### 3. Check Dependencies
```bash
cargo outdated
cargo audit
```

Update if necessary, test thoroughly.

## Release Steps

### 1. Update Version in Cargo.toml

Edit `Cargo.toml`:
```toml
[package]
name = "doxx"
version = "0.2.0"  # <- Update this
```

**Versioning:**
- Patch (0.1.x): Bug fixes, minor improvements
- Minor (0.x.0): New features, non-breaking changes
- Major (x.0.0): Breaking changes

### 2. Update CHANGELOG.md

Add section for new version:

```markdown
## [0.2.0] - 2025-XX-XX

### Added
- Feature descriptions

### Changed
- Improvements

### Fixed
- Bug fixes

### Breaking Changes
- Any breaking changes (for major versions)
```

**Format:**
- Use conventional changelog format
- Link to issue numbers: `(#58)`
- Keep descriptions concise
- Prioritize user-facing changes

### 3. Update Documentation

Check if these need updates:
- README.md (features, installation, examples)
- CLAUDE.md (architecture, development notes)
- Inline code documentation

### 4. Commit Version Bump

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
```

**Important:** No signature blocks in commit message.

### 5. Create Git Tag

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
```

**Tag format:**
- Always prefix with `v`
- Use semantic versioning: `v0.2.0`
- Message format: `Release v0.2.0`

### 6. Push Tag to Trigger CI/CD

```bash
git push origin main
git push origin v0.2.0
```

**Critical:** The tag push triggers the release workflow in `.github/workflows/release.yml`.

## CI/CD Automated Release Process

Once tag is pushed, CI/CD automatically:

1. **Builds cross-platform binaries:**
   - Linux: x86_64-unknown-linux-musl
   - macOS: x86_64-apple-darwin, aarch64-apple-darwin
   - Windows: x86_64-pc-windows-msvc

2. **Creates GitHub Release:**
   - Attaches binaries
   - Extracts changelog section
   - Marks as latest release

3. **Publishes to crates.io:**
   - Requires CARGO_TOKEN secret
   - Automatic on successful build

4. **Updates package managers:**
   - Homebrew formula (automatic via tap)
   - Arch AUR (manual by maintainer)
   - Nix flake (automatic via flake.lock)

## Post-Release Verification

### 1. Verify GitHub Release
- Visit: https://github.com/bgreenwell/doxx/releases
- Check binaries attached
- Verify changelog displayed

### 2. Verify crates.io
- Visit: https://crates.io/crates/doxx
- Check new version published
- Test installation:
  ```bash
  cargo install doxx --force
  doxx --version
  ```

### 3. Test Binary Downloads
```bash
# Linux
curl -L https://github.com/bgreenwell/doxx/releases/latest/download/doxx-Linux-x86_64.tar.gz | tar xz
./doxx --version

# macOS
curl -L https://github.com/bgreenwell/doxx/releases/latest/download/doxx-Darwin-aarch64.tar.gz | tar xz
./doxx --version
```

### 4. Announce Release

**Channels:**
- GitHub Discussions
- Reddit: r/rust, r/commandline
- Twitter/X: @username
- Hacker News (for major releases)

## Troubleshooting

### Release Build Fails

**Symptom:** CI/CD fails on tag push

**Diagnosis:**
```bash
# Check workflow logs
# Visit: https://github.com/bgreenwell/doxx/actions

# Test release build locally
cargo build --release
```

**Fix:**
- Fix build issues
- Delete tag: `git tag -d v0.2.0`
- Delete remote tag: `git push origin :refs/tags/v0.2.0`
- Fix issues, commit
- Create tag again

### crates.io Publish Fails

**Symptom:** Binaries build but crates.io publish fails

**Common causes:**
- CARGO_TOKEN expired
- Version already exists
- Cargo.toml validation issues

**Fix:**
```bash
# Test publish locally
cargo publish --dry-run

# Check for errors
# Fix issues
# Manual publish if needed
cargo publish
```

### Binary Not Compatible

**Symptom:** Users report binary crashes or won't run

**Diagnosis:**
- Check platform architecture
- Verify musl vs glibc on Linux
- Check macOS code signing

**Fix:**
- Add platform to build matrix if missing
- Update release workflow

## Version History

- v0.1.0: Initial release (2025-08-01)
- v0.1.1: Bug fixes, ANSI export (2025-09-15)
- v0.1.2: Equations, strikethrough, search improvements (2025-10-21)
- v0.2.0: Planned - Equation positioning, text wrapping (TBD)

## Resources

- **Release Workflow**: `.github/workflows/release.yml`
- **Cargo Publishing**: https://doc.rust-lang.org/cargo/reference/publishing.html
- **Semantic Versioning**: https://semver.org/
- **Keep a Changelog**: https://keepachangelog.com/
