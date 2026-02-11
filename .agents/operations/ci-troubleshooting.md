# CI/CD Troubleshooting Guide

Common failures and fixes for the CI/CD pipeline.

## CI/CD Pipeline Overview

**Location**: `.github/workflows/ci.yml`

**Platforms Tested**: Linux (Ubuntu), macOS, Windows

**Checks Performed**:
1. Format check (Unix only): `cargo fmt --all -- --check`
2. Clippy lints: `cargo clippy --all-targets -- -D warnings` (ZERO warnings required)
3. Test suite: `cargo test --all-features`
4. Release build: `cargo build --release`
5. Nix build (Unix only): `nix build`

## Common Failures & Fixes

### 1. Format Check Fails ❌

**Error Message:**
```
Diff in /path/to/file.rs
```

**Cause:**
Code formatting doesn't match rustfmt conventions.

**Fix:**
```bash
cargo fmt --all
git add .
git commit --amend --no-edit
```

**Prevention:**
- Run `./scripts/quick-check.sh` before committing
- Install pre-push hook: `cp scripts/pre-push.hook .git/hooks/pre-push`

**Platform Note:**
Format check only runs on Unix (Linux/macOS), not Windows.

### 2. Clippy Warnings ⚠️

**Error Message:**
```
error: ... (clippy::...)
```

**Cause:**
Code quality issues detected by Clippy. CI requires `-D warnings` flag, so ANY warning = failure.

**Common Patterns:**

#### Lifetime Elision
**Error:**
```
error: elided lifetime has a name
```

**Fix:**
Add explicit lifetimes to match compiler's inference:
```rust
// Before
fn render(&self) -> Vec<Line> { ... }

// After
fn render(&self) -> Vec<Line<'_>> { ... }
```

#### Unused Variables
**Error:**
```
error: unused variable: `foo`
```

**Fix:**
Either use the variable or prefix with `_`:
```rust
// Option 1: Remove if truly unused
// let foo = bar();

// Option 2: Prefix with _ if intentionally unused
let _foo = bar();
```

#### Type Mismatches
**Error:**
```
error: mismatched types: expected `X`, found `Y`
```

**Cause:**
Often from dependency updates changing API signatures.

**Fix:**
Check dependency changelogs and update code to match new API:
```bash
cargo update --dry-run  # Check what changed
# Read dependency CHANGELOG
# Update code accordingly
```

### 3. Test Failures 🧪

**Error Message:**
```
test result: FAILED. X passed; Y failed; 0 ignored
```

**Debugging:**
```bash
# Run tests with output
cargo test --all-features -- --nocapture

# Run specific test
cargo test --test integration_test -- --nocapture

# Run single test case
cargo test test_name -- --nocapture --exact
```

**Common Causes:**
- Fixture files missing or corrupted
- Platform-specific behavior differences
- Race conditions in async tests
- Environment-dependent tests (terminal size, locale)

### 4. Build Failures 🔨

**Error Message:**
```
error[E0XXX]: compilation error
```

**Debugging:**
```bash
# Clean build
cargo clean
cargo build --release

# Check specific features
cargo build --all-features

# Update dependencies
cargo update
cargo build
```

**Common Causes:**
- Dependency version conflicts
- Missing feature flags
- Platform-specific compilation issues
- Syntax errors from merge conflicts

### 5. Nix Build Failures ❄️

**Error Message:**
```
error: builder for '...' failed
```

**Cause:**
Nix flake definition out of sync with Cargo.toml or dependencies.

**Fix:**
```bash
# Update flake.lock
nix flake update

# Test locally
nix build

# Check flake.nix matches Cargo.toml version/dependencies
```

## Best Practices

### Before Every Commit
```bash
./scripts/quick-check.sh
```

### Before Every Push
```bash
./scripts/check.sh
```

### Pre-Push Hook (Recommended)
```bash
cp scripts/pre-push.hook .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

This automatically runs validation before every push. Bypass only if necessary:
```bash
git push --no-verify  # Emergency only!
```

## Platform-Specific Notes

### Unix (Linux/macOS)
- Format check enforced
- Nix build enforced
- All checks run

### Windows
- Format check skipped
- Nix build skipped
- Clippy, tests, build still required

## Resources

- **CI Workflow**: `.github/workflows/ci.yml`
- **Validation Scripts**: `scripts/check.sh`, `scripts/quick-check.sh`
- **Clippy Docs**: https://doc.rust-lang.org/clippy/
- **rustfmt Docs**: https://rust-lang.github.io/rustfmt/
