# Test Fixtures

This directory contains comprehensive test documents for validating doxx functionality across various document types and edge cases.

## Core Functionality Tests

### `minimal.docx`
- **Purpose**: Smallest possible test document
- **Content**: Title + single paragraph
- **Tests**: Basic document parsing, minimal content handling
- **Usage**: `cargo run tests/fixtures/minimal.docx`

### `tables-heavy.docx`
- **Purpose**: Complex table parsing and CSV export validation
- **Content**: Multiple tables with headers, financial data, nested information
- **Tests**: Table detection, header identification, CSV export, ASCII rendering
- **Usage**: `cargo run tests/fixtures/tables-heavy.docx --export csv`

### `headings-hierarchy.docx`
- **Purpose**: Outline generation and heading detection
- **Content**: 6 levels of headings in complex hierarchy
- **Tests**: Heading level detection, outline view, document structure parsing
- **Usage**: `cargo run tests/fixtures/headings-hierarchy.docx --outline`

### `formatting-showcase.docx`
- **Purpose**: Text formatting detection and preservation
- **Content**: Bold, italic, combined formatting, special characters
- **Tests**: Formatting extraction, markdown export, style preservation
- **Usage**: `cargo run tests/fixtures/formatting-showcase.docx --export markdown`

### `lists-comprehensive.docx`
- **Purpose**: List parsing and nesting detection
- **Content**: Ordered/unordered lists, nested structures, formatted list items
- **Tests**: List identification, nesting levels, bullet point rendering
- **Usage**: `cargo run tests/fixtures/lists-comprehensive.docx`

## Edge Cases & Stress Tests

### `unicode-special.docx`
- **Purpose**: International character and special symbol handling
- **Content**: Multi-language text, emojis, mathematical symbols, currency
- **Tests**: Unicode parsing, character encoding, special symbol rendering
- **Usage**: `cargo run tests/fixtures/unicode-special.docx --search "π"`

## AI Integration Prep Documents

### `business-report.docx`
- **Purpose**: Professional document for AI summarization testing
- **Content**: Executive summary, KPIs, financial analysis, action items
- **Tests**: Document summarization, key point extraction, business intelligence
- **Usage**: `cargo run tests/fixtures/business-report.docx --summarize` (when AI is enabled)

### `sample.docx`
- **Purpose**: Mixed content for general AI testing
- **Content**: Claude Code conversation with technical discussions
- **Tests**: Q&A functionality, technical content analysis
- **Usage**: `cargo run tests/fixtures/sample.docx --ask "What are agents?"`

## Export Validation

### `export-test.docx`
- **Purpose**: Comprehensive export format validation
- **Content**: Mixed content designed to test all export formats
- **Tests**: Markdown, JSON, CSV, text export accuracy
- **Usage**: 
  ```bash
  cargo run tests/fixtures/export-test.docx --export markdown
  cargo run tests/fixtures/export-test.docx --export json
  cargo run tests/fixtures/export-test.docx --export csv
  ```

## Test Commands Reference

### Basic Testing
```bash
# Test basic functionality
cargo run tests/fixtures/minimal.docx

# Test with all documents
for doc in tests/fixtures/*.docx; do
  echo "Testing: $doc"
  cargo run "$doc" --outline
done
```

### Export Testing
```bash
# Test all export formats
cargo run tests/fixtures/export-test.docx --export markdown > test_output.md
cargo run tests/fixtures/tables-heavy.docx --export csv > test_tables.csv
cargo run tests/fixtures/business-report.docx --export json > test_report.json
```

### Search Testing
```bash
# Test search across different document types
cargo run tests/fixtures/business-report.docx --search "revenue"
cargo run tests/fixtures/unicode-special.docx --search "emoji"
cargo run tests/fixtures/sample.docx --search "agent"
```

### Performance Testing
```bash
# Measure parsing performance
time cargo run tests/fixtures/business-report.docx
time cargo run tests/fixtures/tables-heavy.docx --export csv
```

## Validation Checklist

When adding new test documents:

1. **File naming**: Use descriptive hyphenated names (`new-feature-test.docx`)
2. **Size limits**: Keep under 1MB for fast CI/CD
3. **Content variety**: Include diverse formatting, structures, edge cases
4. **Privacy**: No sensitive, proprietary, or personal information
5. **Documentation**: Update this README with purpose and usage examples
6. **Testing**: Verify all doxx features work with the new document

## Expected Behaviors

### Table Parsing
- Headers automatically detected using heuristics
- Empty cells handled gracefully
- Nested tables supported (content flattened)
- CSV export preserves table structure

### Heading Detection
- Styled headings (Heading1-6) detected accurately
- Text-based heading detection as fallback
- Outline generation respects hierarchy
- Navigation jumps work correctly

### Text Formatting
- Bold, italic, underline preserved in exports
- Combined formatting handled correctly
- Special characters render properly
- Unicode support comprehensive

### Search Functionality
- Case-insensitive by default
- Context provided around matches
- Searches across all content types (text, tables, headings)
- Match highlighting in terminal UI

## Automated Testing Integration

These fixtures are designed for both manual testing and automated integration tests. Each document validates specific functionality and can be used in regression testing to ensure doxx maintains compatibility across updates.