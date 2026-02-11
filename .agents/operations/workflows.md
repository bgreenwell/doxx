# Development Workflows & Manual Commands

Detailed cargo commands and workflow examples for manual testing and development.

## Quick Reference

**Prefer automation:**
- Development: `./scripts/quick-check.sh`
- Pre-push: `./scripts/check.sh`

**Use manual commands when:**
- Scripts unavailable
- Debugging specific issues
- Running selective tests
- Iterating on specific features

## Code Formatting

### Auto-format Code
```bash
cargo fmt --all
```

Use during development to maintain consistent formatting.

### Check Formatting (CI-style)
```bash
cargo fmt --all -- --check
```

Returns non-zero exit code if formatting needed. Used in CI pipeline.

## Linting

### Run Clippy
```bash
cargo clippy --all-targets -- -D warnings
```

**Important:** CI requires `-D warnings` flag (zero warnings policy). Any warning = CI failure.

### Common Flags
```bash
# All targets (bins, tests, benches, examples)
cargo clippy --all-targets

# Specific target
cargo clippy --bin doxx

# Fix auto-fixable issues
cargo clippy --fix --all-targets
```

## Testing

### Run All Tests
```bash
cargo test --all-features
```

### Run Specific Test Suite
```bash
# Integration tests
cargo test --test integration_test

# Unicode safety tests
cargo test --test unicode_safety

# Search functionality tests
cargo test --test search_functionality_test
```

### Run Single Test
```bash
# By name pattern
cargo test test_equation

# Exact match
cargo test test_equation_parsing -- --exact

# With output
cargo test test_equation_parsing -- --nocapture
```

### Test with Debug Output
```bash
# See println! and dbg! output
cargo test -- --nocapture

# Show test names as they run
cargo test -- --show-output
```

## Building

### Development Build
```bash
cargo build
```

Fast compilation, includes debug symbols.

### Release Build
```bash
cargo build --release
```

Optimized binary at `target/release/doxx`. Use for performance testing.

### Check Without Building
```bash
cargo check
```

Fast compilation check without generating binary.

## Running with Fixtures

### Test Fixtures Location
```
tests/fixtures/
├── minimal.docx              # Minimal test case
├── business-report.docx      # Complex formatting
├── equations.docx            # Math equations
├── equation-issue.docx       # Known equation positioning issue
├── images.docx               # Image handling
└── ...
```

### Running Examples

```bash
# Basic viewing
cargo run -- tests/fixtures/minimal.docx

# With search
cargo run -- tests/fixtures/business-report.docx --search "revenue"

# Export formats
cargo run -- tests/fixtures/equations.docx --export text
cargo run -- tests/fixtures/business-report.docx --export markdown
cargo run -- tests/fixtures/business-report.docx --export ansi

# Images
cargo run -- tests/fixtures/images.docx --images --export text

# Outline mode
cargo run -- tests/fixtures/business-report.docx --outline

# With colors
cargo run -- tests/fixtures/business-report.docx --color
```

## Debugging

### Debug Logging
```bash
# Set RUST_LOG environment variable
RUST_LOG=debug cargo run -- tests/fixtures/minimal.docx

# Specific module
RUST_LOG=doxx::document=debug cargo run -- tests/fixtures/minimal.docx

# Trace level (very verbose)
RUST_LOG=trace cargo run -- tests/fixtures/minimal.docx
```

### Backtrace on Panic
```bash
RUST_BACKTRACE=1 cargo run -- tests/fixtures/problem.docx
RUST_BACKTRACE=full cargo run -- tests/fixtures/problem.docx
```

## Dependencies

### Update Dependencies
```bash
# Check what would update
cargo update --dry-run

# Update all dependencies
cargo update

# Update specific dependency
cargo update docx-rs
```

### Check Outdated Dependencies
```bash
cargo install cargo-outdated
cargo outdated
```

### Audit for Security Issues
```bash
cargo install cargo-audit
cargo audit
```

## Benchmarking & Profiling

### Run Benchmarks
```bash
cargo bench
```

### Profile with perf (Linux)
```bash
cargo build --release
perf record --call-graph=dwarf ./target/release/doxx tests/fixtures/large.docx
perf report
```

### Profile with Instruments (macOS)
```bash
cargo build --release
instruments -t "Time Profiler" ./target/release/doxx tests/fixtures/large.docx
```

## Cleanup

### Clean Build Artifacts
```bash
# Remove target directory
cargo clean

# Remove specific profile
cargo clean --release
```

### Remove Dependencies Cache
```bash
rm -rf ~/.cargo/registry
rm -rf ~/.cargo/git
```

## Tips & Tricks

### Watch Mode (with cargo-watch)
```bash
cargo install cargo-watch

# Auto-run tests on file change
cargo watch -x test

# Auto-run specific test
cargo watch -x 'test test_equation'

# Auto-format and test
cargo watch -x fmt -x test
```

### Fast Iteration Workflow
```bash
# 1. Make changes
# 2. Quick check
cargo check

# 3. Run specific test
cargo test test_name -- --nocapture

# 4. Full validation
./scripts/quick-check.sh
```

### Selective Testing
```bash
# Skip slow tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run only unit tests
cargo test --bins
```

## Resources

- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **Test Fixtures**: `tests/fixtures/`
- **Scripts**: `scripts/quick-check.sh`, `scripts/check.sh`
