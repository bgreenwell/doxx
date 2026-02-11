# Test Fixture Consolidation Plan

## Goal

Reduce the number of test fixtures from 19 files (~4.7MB) to ~7-8 essential files (~2-2.5MB) by consolidating related test cases into comprehensive fixtures.

## Current New Fixtures (Untracked)

### `feature-showcase.docx` (1.5MB)
**Status:** ✅ Working, comprehensive coverage

**Content:**
- Text formatting (bold, italic, underline, strikethrough, combinations)
- Colors (red, blue, green, purple, orange)
- Headings (5 levels: H1 → H5)
- Lists (numbered, bulleted, nested)
- Tables (with borders and alignment)
- Equations (inline: E=mc², display equations)
- Images (Sonic + 2 more images)

**Purpose:** Single comprehensive fixture demonstrating all major doxx features

### `edge-cases.docx` (12KB)
**Status:** ⚠️ Currently panicking with Unicode boundary error

**Error:**
```
thread 'main' panicked at src/document.rs:2300:16:
byte index 31 is not a char boundary; it is inside '\u{302}' (bytes 30..32)
of `<m:acc><m:accPr><m:chr m:val="̂"/></m:accPr><m:e><m:r><m:t>y</m:t></m:r></m:e></m:acc>`
```

**Purpose:** Tests Unicode combining characters in equation XML (ŷ = y with combining circumflex)

**Action needed:** Fix Unicode boundary handling in `document.rs:2300` before using

## Current Fixtures Analysis

### Test File Dependencies

**integration_test.rs:**
- `minimal.docx` - Basic parsing test
- `tables-heavy.docx` - CSV export validation
- `headings-hierarchy.docx` - Outline generation
- `formatting-showcase.docx` - Markdown export
- `lists-comprehensive.docx` - Batch test
- `unicode-special.docx` - Unicode handling
- `business-report.docx` - Search, export
- `export-test.docx` - JSON export

**mixed_formatting_test.rs:**
- `color-showcase.docx` - Color rendering
- `formatting-showcase.docx` - Mixed formatting

**test_image_extraction.rs:**
- `images.docx` - Image extraction

**search_functionality_test.rs:**
- `business-report.docx` - Search functionality

### Size Distribution
```
12K   edge-cases.docx (NEW)
16K   color-showcase.docx
16K   equation-issue.docx
16K   equations.docx
16K   numbered-headings.docx
20K   advanced-numbering.docx
20K   minimal.docx
24K   export-test.docx
24K   formatting-showcase.docx
24K   headings-hierarchy.docx
24K   lists-comprehensive.docx
24K   unicode-special.docx
28K   business-report.docx
28K   tables-heavy.docx
100K  retro-gaming-guide.docx
296K  unicode_panic_test.docx
816K  example.docx
816K  simple-numbering.docx
1.5M  feature-showcase.docx (NEW)
1.5M  images.docx
```

## Consolidation Strategy

### Keep (Essential - ~2.5MB)

1. **`minimal.docx`** (20KB)
   - **Reason:** Simplest test case, fast parsing validation
   - **Coverage:** Basic document structure
   - **Tests:** integration_test.rs

2. **`images.docx`** (1.5MB)
   - **Reason:** Dedicated image extraction test
   - **Coverage:** Image handling, terminal rendering
   - **Tests:** test_image_extraction.rs

3. **`feature-showcase.docx`** (1.5MB) ✨ NEW
   - **Reason:** Comprehensive single-file test
   - **Coverage:** All formatting, colors, headings, lists, tables, equations, images
   - **Replaces:** formatting-showcase, color-showcase, lists-comprehensive, headings-hierarchy, export-test, example

4. **`edge-cases.docx`** (12KB) ✨ NEW (once fixed)
   - **Reason:** Unicode edge cases, combining characters
   - **Coverage:** Complex Unicode in equations
   - **Tests:** Equation parser robustness

5. **`business-report.docx`** (28KB)
   - **Reason:** Professional document structure, AI integration prep
   - **Coverage:** Real-world document, search functionality
   - **Tests:** search_functionality_test.rs, integration_test.rs

6. **`tables-heavy.docx`** (28KB)
   - **Reason:** Complex table structures, CSV export validation
   - **Coverage:** Table parsing, header detection, nested tables
   - **Tests:** integration_test.rs (CSV export)

7. **`unicode-special.docx`** (24KB)
   - **Reason:** Comprehensive Unicode/emoji testing
   - **Coverage:** Multi-language, emojis, mathematical symbols, currency
   - **Tests:** integration_test.rs

8. **`equations.docx`** OR **`equation-issue.docx`** (14-16KB)
   - **Reason:** Equation-specific test cases
   - **Coverage:** OMML to LaTeX conversion, inline/display equations
   - **Decision:** Keep one, determine which has better coverage

**Total: 7-8 files (~2-2.5MB)**

### Remove (Covered by consolidation - ~2.2MB)

#### Covered by `feature-showcase.docx`:
- `formatting-showcase.docx` (24KB) - Text formatting now in feature-showcase
- `color-showcase.docx` (16KB) - Colors now in feature-showcase
- `lists-comprehensive.docx` (24KB) - Lists now in feature-showcase
- `headings-hierarchy.docx` (24KB) - Headings now in feature-showcase
- `export-test.docx` (24KB) - Export testing covered by feature-showcase
- `example.docx` (816KB) - Generic example, replaced by feature-showcase

#### Niche/Redundant:
- `retro-gaming-guide.docx` (100KB) - Not used in tests
- `numbered-headings.docx` (16KB) - Covered by feature-showcase headings
- `advanced-numbering.docx` (20KB) - Known issue (#24), specific edge case
- `simple-numbering.docx` (816KB) - Large file, specific numbering test
- `unicode_panic_test.docx` (296KB) - Large panic test, replaced by edge-cases.docx

**Total removed: 10 files (~2.2MB)**

## Implementation Steps

### Phase 1: Fix Blockers
- [ ] **Fix `edge-cases.docx` panic** - Unicode boundary handling in `document.rs:2300`
  - Issue: String slicing on Unicode combining character (U+0302)
  - Location: Equation XML parsing
  - Fix: Use character-aware slicing or `.char_indices()`

### Phase 2: Update Tests
- [ ] Update `integration_test.rs` to use `feature-showcase.docx`:
  - Replace `formatting-showcase.docx` → `feature-showcase.docx`
  - Replace `export-test.docx` → `feature-showcase.docx`
  - Add `feature-showcase.docx` to batch test array
- [ ] Update `mixed_formatting_test.rs`:
  - Replace `color-showcase.docx` → `feature-showcase.docx`
  - Replace `formatting-showcase.docx` → `feature-showcase.docx`
- [ ] Add tests for `edge-cases.docx` once panic is fixed

### Phase 3: Verify
- [ ] Run full test suite: `cargo test --all-features`
- [ ] Run integration tests: `cargo test --test integration_test`
- [ ] Run formatting tests: `cargo test --test mixed_formatting_test`
- [ ] Verify all export formats work with `feature-showcase.docx`
- [ ] Test search functionality across retained fixtures

### Phase 4: Remove Redundant Files
- [ ] Remove files listed in "Remove" section
- [ ] Update `tests/fixtures/README.md` with new structure
- [ ] Update documentation references to removed fixtures
- [ ] Commit changes

### Phase 5: Update Documentation
- [ ] Update `tests/fixtures/README.md`:
  - Add `feature-showcase.docx` documentation
  - Add `edge-cases.docx` documentation
  - Remove entries for deleted fixtures
  - Update usage examples
- [ ] Update `CLAUDE.md` if it references specific fixtures
- [ ] Update any scripts in `scripts/` that reference fixtures

## Expected Benefits

### Repository Size
- **Before:** 19 fixtures, ~4.7MB
- **After:** 7-8 fixtures, ~2-2.5MB
- **Savings:** ~2.2MB (47% reduction)

### Maintenance
- Fewer files to maintain and update
- Single comprehensive test document easier to understand
- Reduced CI/CD time (fewer files to test)
- Clearer test intentions

### Test Coverage
- ✅ All existing functionality still tested
- ✅ Better organized test cases
- ✅ New edge case coverage (Unicode combining chars)
- ✅ More realistic comprehensive document

## Risk Mitigation

### Backup Strategy
Before removing files:
```bash
# Create backup branch
git checkout -b backup/fixture-consolidation
git add tests/fixtures/*.docx
git commit -m "Backup: All fixtures before consolidation"
git checkout main

# Or create archive
tar -czf fixtures-backup-$(date +%Y%m%d).tar.gz tests/fixtures/*.docx
```

### Rollback Plan
If tests fail after consolidation:
1. Check git log for removed files
2. Restore specific fixtures: `git checkout HEAD^ -- tests/fixtures/FILENAME.docx`
3. Re-run tests to identify specific failures
4. Adjust consolidation strategy based on failures

## Open Questions

1. **Equation fixtures:** Keep `equations.docx` or `equation-issue.docx`?
   - Compare coverage between the two
   - Determine which has more comprehensive equation examples

2. **Numbering fixtures:** Are advanced-numbering and simple-numbering needed?
   - Known issue #24 mentions "legacy Word documents with style-based numbering"
   - May want to keep one for regression testing

3. **Unicode panic test:** Is `unicode_panic_test.docx` (296KB) needed?
   - If `edge-cases.docx` covers the same Unicode issues, can remove
   - May have different Unicode edge cases worth preserving

## Success Criteria

- [ ] All tests pass with new fixture structure
- [ ] Repository size reduced by ~40%+
- [ ] Test coverage maintained or improved
- [ ] Documentation updated
- [ ] No test dependencies on removed fixtures
- [ ] CI/CD passes on all platforms
