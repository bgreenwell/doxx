#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "Regenerating test fixtures..."

# comprehensive.docx: generated from the markdown source via pandoc
if ! command -v pandoc >/dev/null 2>&1; then
    echo "Warning: pandoc not found — skipping comprehensive.docx regeneration"
    echo "Install pandoc to regenerate: https://pandoc.org/installing.html"
else
    pandoc tests/fixtures/comprehensive.md -o tests/fixtures/comprehensive.docx
    echo "Generated: tests/fixtures/comprehensive.docx"
fi

# minimal.docx, colors.docx: generated via docx-rs
cargo run --bin generate_test_docs

echo "Done."
