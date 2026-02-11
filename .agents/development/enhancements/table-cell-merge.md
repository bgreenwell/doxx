# Enhancement: Table Cell Merge Support

## Issue Reference
GitHub Issue: #67 (Nov 10, 2025)

## Problem Statement

Word documents can contain tables with merged cells that span multiple columns using the `<w:gridSpan>` element. Currently, doxx does not parse or render these correctly, causing table layout issues.

User-provided example shows a single cell spanning all 32 columns:
```xml
<w:tc>
  <w:tcPr>
    <w:tcW w:w="8640" w:type="dxa"/>
    <w:gridSpan w:val="32"/>  <!-- This specifies column span -->
  </w:tcPr>
  <w:p>
    <w:r><w:t>ProtocolId</w:t></w:r>
  </w:p>
</w:tc>
```

## Impact

- Technical documents with complex table layouts render incorrectly
- Protocol specifications, data tables with headers affected
- Competitive disadvantage vs. other .docx viewers

## Proposed Solution

Extend table parsing in `src/document.rs` to recognize and handle `gridSpan` properties:

1. Parse `<w:gridSpan>` element from table cell properties
2. Store span information in `TableCell` structure
3. Update table rendering logic to respect column spans
4. Adjust ASCII table layout calculations

## Implementation Plan

### 1. Extend Data Structures (src/document.rs)

```rust
pub struct TableCell {
    pub content: Vec<Paragraph>,
    pub col_span: Option<usize>,  // Add this field
    // ... existing fields
}
```

### 2. Parse gridSpan in docx-rs Callback

Modify table parsing to extract `gridSpan` value:
```rust
// In parse_table_cell or equivalent
if let Some(tc_pr) = cell.property {
    if let Some(grid_span) = tc_pr.grid_span {
        table_cell.col_span = Some(grid_span as usize);
    }
}
```

### 3. Update Table Rendering (src/ui.rs, src/export.rs)

ASCII table rendering must account for merged cells:
- Skip rendering cells that are spanned over
- Adjust column width calculations
- Extend cell content across multiple columns

### 4. Handle Edge Cases

- Cells with `gridSpan=1` (no merge) - treat as normal
- Cells spanning beyond table width - clamp to table bounds
- Row height calculations with merged cells
- Alignment within merged cells

## Technical Considerations

**Dependencies:**
- Requires `docx-rs` 0.4+ (current version supports `GridSpan`)
- No new crates needed

**Files Affected:**
- `src/document.rs:TableCell` structure (~line 150)
- `src/document.rs:parse_table` function (~line 800)
- `src/ui.rs:render_table` function (~line 600)
- `src/export.rs:format_table` (all export formats)

**Breaking Changes:**
- `TableCell` structure changes (minor version bump acceptable)
- Table rendering may change layout for existing documents (intentional fix)

**Complexity:**
- ASCII table layout with spans is non-trivial
- Must handle variable-width columns + merged cells
- Export formats (Markdown, CSV) have different span support

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_gridspan_parsing() {
    // Parse table with gridSpan=2
    // Verify TableCell.col_span == Some(2)
}

#[test]
fn test_merged_cell_rendering() {
    // Create table with merged cells
    // Verify ASCII output spans correct width
}
```

### Integration Tests
- Use user-provided document from #67
- Create test fixture with various gridSpan values
- Test all export formats (text, markdown, ANSI)

### Test Cases
1. Single cell spanning 2 columns
2. Header row with full-width merged cell (user's example)
3. Table with mix of normal and merged cells
4. Edge case: gridSpan exceeding table width

## Success Criteria

- [ ] `gridSpan` attribute parsed from DOCX XML
- [ ] `TableCell` structure stores col_span information
- [ ] ASCII table rendering respects merged cells
- [ ] All export formats handle merged cells appropriately
- [ ] User-provided test case (#67) renders correctly
- [ ] No regression in tables without merged cells
- [ ] Documentation updated in README

## Alternative Approaches

1. **Ignore gridSpan** - Simple but poor UX
2. **Merge during parsing** - Combine cells into single wide cell (chosen approach)
3. **Virtual columns** - Expand grid to handle spans (complex, better for future)

## Priority

**Medium-High** - Recent user request with clear example, improves rendering accuracy

## Estimated Effort

- Parsing: 2 hours
- Rendering logic: 4-6 hours
- Export format updates: 3-4 hours
- Testing: 2-3 hours
**Total: ~12-15 hours** (2-3 days)

## References

- Issue: https://github.com/bgreenwell/doxx/issues/67
- Word OOXML spec: gridSpan element
- docx-rs GridSpan support: https://docs.rs/docx-rs/latest/docx_rs/
