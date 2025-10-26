#!/usr/bin/env bash
# Quick validation script for iterative development
# Runs essential checks only (faster than full check.sh)

set -e

echo "⚡ Running quick validation..."
echo ""

# 1. Format and fix automatically
echo "📝 Auto-formatting code..."
cargo fmt --all
echo "✅ Formatted"
echo ""

# 2. Clippy (most important for catching errors)
echo "🔎 Running clippy..."
if cargo clippy --all-targets -- -D warnings; then
    echo "✅ Clippy passed"
else
    echo "❌ Clippy failed"
    exit 1
fi
echo ""

# 3. Quick test (skip release build for speed)
echo "🧪 Running tests..."
if cargo test --all-features; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed"
    exit 1
fi
echo ""

echo "✅ Quick checks passed! Run ./scripts/check.sh before pushing."
