#!/bin/bash

# Test script for validating all doxx test documents
# Run from the doxx project root directory

set -e

echo "Testing doxx with generated test documents..."
echo "=============================================="

# Test minimal document
echo "1. Testing minimal.docx..."
cargo run --bin doxx tests/fixtures/minimal.docx

echo -e "\n2. Testing tables-heavy.docx with CSV export..."
cargo run --bin doxx tests/fixtures/tables-heavy.docx --export csv | head -20

echo -e "\n3. Testing headings-hierarchy.docx with outline..."
cargo run --bin doxx tests/fixtures/headings-hierarchy.docx --outline

echo -e "\n4. Testing formatting-showcase.docx with markdown export..."
cargo run --bin doxx tests/fixtures/formatting-showcase.docx --export markdown | head -20

echo -e "\n5. Testing business-report.docx..."
cargo run --bin doxx tests/fixtures/business-report.docx

echo -e "\n6. Testing unicode-special.docx..."
cargo run --bin doxx tests/fixtures/unicode-special.docx --search "emoji"

echo -e "\n7. Testing export-test.docx with JSON export..."
cargo run --bin doxx tests/fixtures/export-test.docx --export json | head -10

echo -e "\n8. Testing lists-comprehensive.docx..."
cargo run --bin doxx tests/fixtures/lists-comprehensive.docx

echo -e "\nAll tests completed! ✅"
echo "Run individual tests with:"
echo "  cargo run --bin doxx tests/fixtures/[document-name].docx [options]"