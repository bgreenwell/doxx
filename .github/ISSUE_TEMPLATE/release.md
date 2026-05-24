---
name: Release
about: Track a new version release
title: 'Release vX.Y.Z'
labels: release
assignees: ''

---

## Release Version

**Version:** vX.Y.Z

## Pre-Release Checklist

- [ ] All tests passing: `cargo test`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] CHANGELOG.md updated with new version changes
- [ ] Version bumped in `Cargo.toml`
- [ ] Test binary works: `cargo run --release -- tests/fixtures/comprehensive.docx`

## Create Release

- [ ] Commit version bump: `git commit -m "chore: release X.Y.Z"`
- [ ] Push to main: `git push`
- [ ] Create version tag: `git tag vX.Y.Z`
- [ ] Push tag: `git push origin vX.Y.Z`
- [ ] Wait for GitHub Actions workflows to complete (~10-15 minutes)

## Automated Release Verification

All publishing is automated. Verify these workflows complete successfully:

### Core Release (`.github/workflows/release.yml`)
- [ ] GitHub Release created at https://github.com/bgreenwell/doxx/releases/tag/vX.Y.Z
- [ ] All artifacts present (binaries, tarballs, installers, checksums)
- [ ] Homebrew formula published to [homebrew-doxx](https://github.com/bgreenwell/homebrew-doxx)
- [ ] Published to [crates.io](https://crates.io/crates/doxx)

### Distribution Workflows
- [ ] Scoop manifest published — `.github/workflows/publish-scoop.yml`
- [ ] AUR package updated — `.github/workflows/publish-aur.yml`
- [ ] WinGet PR created — `.github/workflows/publish-winget.yml` (may take 1-2 days to merge)
