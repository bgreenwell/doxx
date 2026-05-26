# Scripts

| Script | Purpose |
|--------|---------|
| `quick-check.sh` | Auto-format, clippy, tests — use during development |
| `check.sh` | Full CI mirror (fmt check, clippy, tests, release build) — use before pushing |
| `pre-push.hook` | Git hook wrapper around `check.sh` |
| `release.sh` | Bump version, validate, commit, tag, and push |
| `regenerate-fixtures.sh` | Rebuild test fixtures via pandoc and `generate_test_docs` |

## Setup

```bash
# Optional: auto-run full checks before every push
cp scripts/pre-push.hook .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

## Release workflow

1. Update `CHANGELOG.md` (move `[Unreleased]` entries to the new version section)
2. Run `./scripts/release.sh [major|minor|patch]`

The script bumps `Cargo.toml`, runs fmt/clippy/tests, commits, tags, and pushes.
GitHub Actions handles binaries, crates.io, Homebrew, Scoop, AUR, and WinGet from there.
