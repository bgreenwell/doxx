# Plan: Refactor src/document.rs into Modular Components

## Goal
Refactor the monolithic 2,611-line `src/document.rs` file into a clean module hierarchy while maintaining 100% backward compatibility and passing all tests.

## Current State

**File Statistics:**
- Lines: 2,611
- Functions/Types: 67
- Public API: 3 functions, 14 types
- Tests: 9 unit tests, all passing

**Problem:** Monolithic file makes code hard to navigate, test, and maintain. Gemini audit identified this as a key code quality issue.

## Target Module Structure

```
src/
  document/
    mod.rs              # Public API re-exports (~150 lines)
    models.rs           # Core data structures (~165 lines)
    io.rs               # File I/O and validation (~93 lines)
    loader.rs           # Main document loading (~340 lines)
    parsing/
      mod.rs
      heading.rs        # Heading detection (~507 lines)
      numbering.rs      # Numbering management (~263 lines)
      formatting.rs     # Text extraction (~391 lines)
      list.rs           # List processing (~197 lines)
      table.rs          # Table extraction (~307 lines)
      equation.rs       # Equation processing (~597 lines)
    query.rs            # Search and outline (~103 lines)
    cleanup.rs          # Post-processing (~55 lines)
```

**Total:** 13 new files, average ~200-600 lines each

### Detailed Module Descriptions

**1. mod.rs** - Public API Facade
- Re-exports all public types and functions
- Main entry point for other crates
- Contains: `pub use` statements, module declarations
- Purpose: Maintains backward compatibility while organizing internals

**2. models.rs** - Core Data Structures
- All public structs/enums: Document, DocumentElement, TextFormatting, etc.
- Pure data containers with serde derives
- Contains: 14 public types, impl blocks for FormattedRun consolidation
- Purpose: Foundation layer with zero internal dependencies

**3. io.rs** - File I/O and Validation
- `validate_docx_file()` - Checks file is valid ZIP with required entries
- `merge_display_equations()` - Combines split equations from multiple paragraphs
- Contains: File handling, ZIP validation logic
- Purpose: Isolate file system operations and validation

**4. loader.rs** - Document Loading Orchestration
- `load_document()` - Main async entry point (340 lines)
- Orchestrates: docx parsing, heading detection, numbering, list grouping, table extraction
- Contains: Main parsing pipeline, calls all other modules
- Purpose: High-level coordination of document loading

**5. parsing/heading.rs** - Heading Detection
- Style-based detection (Heading1-6 from Word styles)
- Heuristic detection (bold + font size + length patterns)
- `HeadingInfo` struct, regex patterns for manual numbering
- Contains: `detect_heading_from_paragraph_style()`, `detect_heading_with_numbering()`, etc.
- Purpose: Identify headings from styles or text characteristics

**6. parsing/numbering.rs** - Numbering Management
- `DocumentNumberingManager` - Tracks list/heading numbering state
- `HeadingNumberTracker` - Auto-generates sequential heading numbers
- `NumberingFormat` enum - Decimal, roman, letter formats
- Contains: Stateful numbering logic, format conversion
- Purpose: Manage hierarchical numbering for lists and headings

**7. parsing/formatting.rs** - Text Extraction & Formatting
- `extract_run_formatting()` - Gets bold, italic, color from docx-rs
- `extract_paragraph_text()`, `extract_run_text()` - Text extraction
- `detect_heading_from_text()` - Heuristic heading detection
- Contains: Text processing utilities, formatting extraction
- Purpose: Bridge between docx-rs types and our models

**8. parsing/list.rs** - List Processing
- `group_list_items()` - Converts paragraphs to hierarchical lists
- `is_likely_list_item()` - Heuristic list detection
- `calculate_list_level()` - Determines nesting depth
- Contains: List detection and structuring logic
- Purpose: Transform flat paragraphs into nested list structures

**9. parsing/table.rs** - Table Extraction
- `extract_table_data()` - Main table parsing function
- `appears_to_be_header()` - Detects header rows
- `detect_cell_data_type()` - Number, date, text classification
- Contains: Table parsing, column width calculation, alignment detection
- Purpose: Extract and enhance table data with metadata

**10. parsing/equation.rs** - Equation Processing (Largest Module)
- `extract_equations_from_docx()` - Find display equations in XML
- `omml_to_latex()` - Convert Office MathML to LaTeX (400+ lines)
- `extract_inline_equation_positions()` - Inline equation detection
- Contains: Complex XML parsing, OMML→LaTeX conversion, regex patterns
- Purpose: Handle mathematical equations from Word

**11. query.rs** - Search & Navigation
- `search_document()` - Full-text search across all elements
- `generate_outline()` - Creates table of contents from headings
- Contains: Read-only document traversal functions
- Purpose: Public API for document querying

**12. cleanup.rs** - Post-Processing
- `clean_word_list_markers()` - Removes Word's internal list markers
- `is_likely_sentence()` - Sentence detection helper
- Contains: Post-load cleanup utilities
- Purpose: Final document cleanup after parsing

**13. parsing/mod.rs** - Parsing Module Organization
- Module exports and organization
- Re-exports parsing submodules
- Purpose: Clean namespace for parsing functionality

## Implementation Strategy: 13-Phase Incremental Approach

### Phase 1: Foundation (30 min, LOW RISK)
- Create `src/document/` directory structure
- Rename `document.rs` → `document_legacy.rs` (temporary)
- Create empty module files with placeholders
- Update `src/lib.rs` to use new module structure
- **Verify:** `cargo test --lib` passes

### Phase 2: Extract Models (1 hour, LOW RISK)
**Move:** Lines 13-177, 1937-1941
**Content:** All public structs/enums (ImageOptions, Document, DocumentElement, TextFormatting, etc.)
**Why first:** Pure data structures, no dependencies, lowest risk

### Phase 3: Extract I/O (45 min, LOW RISK)
**Move:** Lines 179-271
**Content:** `validate_docx_file()`, `merge_display_equations()`
**Dependencies:** Only uses models

### Phase 4: Extract Query (30 min, LOW RISK)
**Move:** Lines 1525-1627
**Content:** `search_document()`, `generate_outline()`
**Dependencies:** Only uses models
**Tests:** `search_functionality_test.rs`

### Phase 5: Extract Cleanup (15 min, LOW RISK)
**Move:** Lines 1943-1997
**Content:** `clean_word_list_markers()`
**Dependencies:** Only uses models

### Phase 6: Extract Table Processing (1.5 hours, MEDIUM RISK)
**Move:** Lines 1629-1935
**Content:** `extract_table_data()`, helper functions, impl blocks
**Dependencies:** Models + formatting (temporary cross-module call)
**Tests:** `ansi_export_test.rs` (table rendering)

### Phase 7: Extract List Processing (1 hour, MEDIUM RISK)
**Move:** Lines 1299-1495
**Content:** `is_likely_list_item()`, `group_list_items()`, helpers
**Dependencies:** Only uses models

### Phase 8: Extract Formatting (1.5 hours, MEDIUM-HIGH RISK)
**Move:** Lines 908-1298, 1520-1523
**Content:** Text extraction, `extract_run_formatting()`, heading detection helpers
**Dependencies:** Called from loader and table modules
**Tests:** `mixed_formatting_test.rs`, `strikethrough_test.rs`

### Phase 9: Extract Heading Detection (2 hours, HIGH RISK)
**Move:** Lines 613-1119, 792-796, 1081-1093
**Content:** Style-based + heuristic detection, regex patterns, HeadingInfo
**Special:** Move inline tests (1121-1173) to `tests/heading_detection_test.rs`
**Dependencies:** Uses numbering module

### Phase 10: Extract Numbering Management (1.5 hours, HIGH RISK)
**Move:** Lines 632-905, 997-1040
**Content:** DocumentNumberingManager, HeadingNumberTracker, NumberingFormat
**Dependencies:** Tightly coupled with heading detection

### Phase 11: Extract Equation Processing (2 hours, HIGHEST RISK)
**Move:** Lines 1999-2611
**Content:** OMML to LaTeX conversion, XML parsing, ~597 lines
**Dependencies:** Complex parsing logic, uses io module
**Tests:** Equation rendering

### Phase 12: Extract Main Loader (2 hours, CRITICAL RISK)
**Move:** Lines 272-611
**Content:** Main `load_document()` async function
**Dependencies:** ALL other modules (orchestration function)
**Tests:** All integration tests

### Phase 13: Final Cleanup (1 hour)
- Delete `document_legacy.rs`
- Clean up re-exports in `mod.rs`
- Add module documentation
- `cargo fmt`, `cargo clippy`
- Final verification

## Dependency Graph

```
loader → parsing/*, io, cleanup, models
parsing/equation → models, io
parsing/table → models, parsing/formatting
parsing/heading → models, parsing/numbering
parsing/numbering → models
parsing/formatting → models
parsing/list → models
io → models
cleanup → models
query → models
models → (no internal deps)
```

**Key Principle:** `models` has no internal dependencies; all modules can depend on models

## Critical Constraints

1. **Backward Compatibility:** Public API unchanged
2. **All Tests Pass:** 9 unit tests + integration tests must pass after each phase
3. **No Functional Changes:** Pure refactoring, no behavior changes
4. **No Circular Dependencies:** Clear dependency hierarchy

## Testing Strategy

**After Each Phase:**
```bash
cargo fmt
cargo build --lib
cargo test --lib        # All 9 tests must pass
git commit -m "Phase X: [description]"
```

**Focus Areas:**
- Search functionality (`search_functionality_test.rs`)
- Formatting (`mixed_formatting_test.rs`, `strikethrough_test.rs`)
- ANSI export (`ansi_export_test.rs`)
- Integration (`integration_test.rs`)

## Risk Management

| Phase | Risk Level | Primary Risk | Mitigation |
|-------|-----------|--------------|------------|
| 1-5 | LOW | Import errors | Clear dependency tracking |
| 6-8 | MEDIUM | Logic bugs | Extensive testing |
| 9-10 | HIGH | Regex/state bugs | Move tests with code |
| 11 | HIGHEST | XML parsing breaks | Keep equation tests |
| 12 | CRITICAL | Everything breaks | Full test suite |

**Rollback Strategy:**
- Git commit before each phase
- Keep `document_legacy.rs` until Phase 13
- Emergency rollback: `git reset --hard HEAD`

## Timeline Estimate

**Total:** 15-16 hours

**Recommended Schedule:**
- Day 1 (4h): Phases 1-5 (foundation, low-risk)
- Day 2 (5h): Phases 6-8 (medium-risk parsing)
- Day 3 (6h): Phases 9-13 (high-risk + critical)

## Success Criteria

**Must Have:**
- ✓ All 9 unit tests pass
- ✓ All integration tests pass
- ✓ Zero compilation warnings
- ✓ Public API unchanged
- ✓ No functional changes

**Should Have:**
- ✓ Code coverage maintained
- ✓ Clippy passes
- ✓ Module documentation added

## Benefits

**Code Organization:**
- 13 focused modules vs 1 monolithic file
- Average 200-600 lines per module (was 2,611)
- Clear separation of concerns

**Maintainability:**
- Easier to locate functionality
- Better git blame tracking
- Reduced cognitive load

**Future Development:**
- Easy to add new element types
- Can parallelize parsing
- Ready for alternative parsers (ODT, etc.)

## Critical Files

**To Modify:**
- `src/document.rs` → Refactor into modules
- `src/lib.rs` → Update module declarations

**External Dependencies (verify re-exports work):**
- `src/ansi.rs` - Uses `use crate::document::*;`
- `src/export.rs` - Uses `use crate::document::*;`
- `src/widgets/document.rs` - Uses document types

**Tests to Verify:**
- `tests/search_functionality_test.rs`
- `tests/integration_test.rs`
- All 9 unit tests in document.rs

## Line-by-Line Module Mapping

**Quick Reference:**
- Lines 13-177 → models.rs (core structs/enums)
- Lines 179-271 → io.rs (file I/O)
- Lines 272-611 → loader.rs (main loading)
- Lines 613-1119 → parsing/heading.rs (heading detection)
- Lines 632-905, 997-1040 → parsing/numbering.rs (numbering)
- Lines 908-1298 → parsing/formatting.rs (text extraction)
- Lines 1299-1495 → parsing/list.rs (list processing)
- Lines 1525-1627 → query.rs (search/outline)
- Lines 1629-1935 → parsing/table.rs (table extraction)
- Lines 1943-1997 → cleanup.rs (post-processing)
- Lines 1999-2611 → parsing/equation.rs (OMML conversion)

---

This refactoring transforms a 2,611-line monolithic file into 13 well-organized modules, significantly improving code maintainability while preserving all existing functionality.
