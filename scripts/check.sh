#!/usr/bin/env bash
# Pre-push validation script - mirrors CI/CD checks
# Run this before committing to catch issues early

set -e  # Exit on first error

echo "🔍 Running pre-push validation checks..."
echo ""

# Track overall success
FAILED=0

# 1. Format check
echo "📝 Checking code formatting..."
if cargo fmt --all -- --check; then
    echo "✅ Format check passed"
else
    echo "❌ Format check failed - run: cargo fmt --all"
    FAILED=1
fi
echo ""

# 2. Clippy lints
echo "🔎 Running clippy lints..."
if cargo clippy --all-targets -- -D warnings; then
    echo "✅ Clippy passed"
else
    echo "❌ Clippy failed"
    FAILED=1
fi
echo ""

# 3. Tests
echo "🧪 Running tests..."
if cargo test --all-features; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed"
    FAILED=1
fi
echo ""

# 4. Build check
echo "🔨 Checking release build..."
if cargo build --release; then
    echo "✅ Build passed"
else
    echo "❌ Build failed"
    FAILED=1
fi
echo ""

# Summary
if [ $FAILED -eq 0 ]; then
    echo "✅ All checks passed! Safe to push."
    exit 0
else
    echo "❌ Some checks failed. Fix issues before pushing."
    exit 1
fi
