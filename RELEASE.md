# Release guide for doxx

This document outlines the complete release process for doxx, including automated pipelines and manual steps.

## 🎯 Release overview

The release pipeline includes:
- ✅ **Cross-platform binaries** (Linux, macOS Intel/ARM, Windows)
- ✅ **GitHub releases** with automated changelog
- ✅ **crates.io publishing** for `cargo install doxx`
- ✅ **Homebrew formula** (automated updates)
- ✅ **Checksums** for security verification
- ✅ **Modern GitHub Actions** with proper error handling

## 🚀 Quick release process

### 1. Prepare release
```bash
# Make sure you're on main branch and working directory is clean
git checkout main
git pull origin main

# Run the automated release script
./scripts/release.sh [major|minor|patch]

# Example for patch release (0.1.0 -> 0.1.1)
./scripts/release.sh patch
```

### 2. The script will:
- ✅ Bump version in `Cargo.toml`
- ✅ Run tests to ensure everything works
- ✅ Update `Cargo.lock`
- ✅ Commit version bump
- ✅ Create and push git tag (e.g., `v0.1.1`)
- ✅ Trigger GitHub Actions automatically

### 3. GitHub Actions will:
- ✅ Build cross-platform binaries (Linux musl, macOS Intel/ARM, Windows)
- ✅ Create draft GitHub release with CHANGELOG.md
- ✅ Generate SHA256 checksums
- ✅ Publish to crates.io (when draft is published)
- ✅ Update Homebrew formula automatically

### 4. Manual steps
1. **Review draft release**
   - Go to [GitHub releases](https://github.com/bgreenwell/doxx/releases)
   - Edit the draft release created by Actions
   - Add release highlights and breaking changes if any
   
2. **Publish release**
   - Click "Publish release" to make it live
   - This triggers crates.io publishing
   - Homebrew formula gets updated automatically

## 📦 Package manager status

### ✅ Active package managers
- **crates.io**: `cargo install doxx` ✅ Automated
- **GitHub releases**: Direct binary downloads ✅ Automated  
- **Homebrew**: In progress 🚧 (Formula ready, tap needed)

### 🚧 Future package managers
- **Scoop** (Windows): Repository structure ready
- **Chocolatey** (Windows): Future consideration
- **Snap** (Linux): Future consideration
- **AUR** (Arch Linux): Community contribution welcome

## 🔍 Testing release pipeline

### Test without publishing
```bash
# Test packaging for crates.io (dry run)
cargo publish --dry-run

# Test binary builds locally
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target x86_64-apple-darwin

# Test CLI works correctly
./target/release/doxx --version
./target/release/doxx tests/fixtures/sample.docx --export text
```

### Verify release assets
After GitHub release is published:

```bash
# Download and verify checksums
wget https://github.com/bgreenwell/doxx/releases/latest/download/doxx-checksums.txt
wget https://github.com/bgreenwell/doxx/releases/latest/download/doxx-linux-x86_64.tar.gz

# Verify checksum matches
sha256sum doxx-linux-x86_64.tar.gz
grep linux-x86_64 doxx-checksums.txt
```

## 📋 Pre-release checklist

- [ ] All CI/CD tests passing on main branch
- [ ] `CHANGELOG.md` updated with release notes
- [ ] Version number follows semantic versioning
- [ ] All new features documented in README
- [ ] Breaking changes clearly documented
- [ ] Security issues addressed
- [ ] Dependencies updated and audited

## 🛠 Troubleshooting

### Release script issues
```bash
# If release script fails, check:
git status                    # Working directory clean?
cargo test --all-features    # All tests pass?
cargo clippy -- -D warnings  # No linting issues?
```

### GitHub Actions issues
- Check Actions tab for build failures
- Common issues: Missing secrets (`CARGO_REGISTRY_TOKEN`)
- Platform-specific build failures (usually dependency issues)

### crates.io publishing issues
- Ensure `CARGO_REGISTRY_TOKEN` secret is set
- Verify all required metadata in `Cargo.toml`
- Check for naming conflicts

## 🔐 Required secrets

Repository secrets needed for full automation:

- `CARGO_REGISTRY_TOKEN`: For publishing to crates.io
  - Get from https://crates.io/me
  - Scope: "Publish new crates and update existing crates"

## 📈 Success metrics

After release, verify:
- [ ] GitHub release created with all binary assets
- [ ] crates.io shows new version (may take a few minutes)
- [ ] `cargo install doxx` works with new version
- [ ] Download links in README work correctly
- [ ] Homebrew formula updated (if tap is public)

## 🎉 Post-release

1. **Announce release**
   - Update README badges if needed
   - Consider social media announcement
   - Update any documentation sites

2. **Monitor**
   - Watch for user issues or bug reports
   - Monitor download statistics
   - Track performance metrics

---

**Need help?** Check the GitHub Actions logs or open an issue for release pipeline problems.