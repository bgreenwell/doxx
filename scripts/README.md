# Development Scripts

This directory contains scripts to help ensure code quality and CI/CD compatibility.

## Available Scripts

### `check.sh` - Full Pre-Push Validation
Runs all checks that CI/CD runs. Use before pushing commits.

```bash
./scripts/check.sh
```

**Checks performed:**
- ✅ Code formatting (`cargo fmt --check`)
- ✅ Clippy lints (`cargo clippy --all-targets -- -D warnings`)
- ✅ All tests (`cargo test --all-features`)
- ✅ Release build (`cargo build --release`)

**When to use:** Before pushing commits to ensure CI/CD will pass.

---

### `quick-check.sh` - Fast Iterative Development
Runs essential checks only. Faster for iterative development.

```bash
./scripts/quick-check.sh
```

**Checks performed:**
- ✅ Auto-format code (`cargo fmt`)
- ✅ Clippy lints
- ✅ All tests

**When to use:** During development, before each commit.

---

### `pre-push.hook` - Git Hook (Optional)
Automatically runs validation before every push.

**Installation:**
```bash
cp scripts/pre-push.hook .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

**Bypass (use sparingly):**
```bash
git push --no-verify
```

**When to use:** Set up once to prevent accidentally pushing broken code.

---

## Recommended Workflow

### During Development
```bash
# Make changes...
./scripts/quick-check.sh  # Fast validation
git add .
git commit -m "feat: add feature"
```

### Before Pushing
```bash
./scripts/check.sh        # Full validation
git push origin feature-branch
```

### One-Time Setup (Optional)
```bash
# Install pre-push hook to automate validation
cp scripts/pre-push.hook .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

---

## CI/CD Compatibility

These scripts mirror the checks in `.github/workflows/ci.yml`:
- Format check (Unix only in CI, but recommended for all platforms)
- Clippy with warnings as errors
- Full test suite
- Release build verification

Running `./scripts/check.sh` locally ensures CI/CD will pass.

---

## Troubleshooting

**Format check fails:**
```bash
cargo fmt --all  # Auto-fix formatting
```

**Clippy fails:**
- Read the error messages carefully
- Fix the code issues
- Common issues: unused variables, lifetime annotations, type mismatches

**Tests fail:**
```bash
cargo test --all-features -- --nocapture  # Show test output
```

**Build fails:**
- Check for compilation errors
- Ensure all dependencies are up to date: `cargo update`
