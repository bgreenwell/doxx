# Enhancement: Font-Size Based Heading Detection

## Issue Reference
GitHub Issue: #3 (Aug 17, 2025)

## Problem Statement

Many Word documents contain "visual headings" created by increasing font size and bold formatting rather than using proper heading styles. Pandoc and other converters miss these entirely, causing information loss in Markdown/outline exports.

**User Pain Point:** "Only issue with using pandoc to convert word to markdown" - Reddit user

## Current Behavior

doxx detects headings via:
1. ✅ Proper Word heading styles (Heading1, Heading2, etc.)
2. ✅ Text heuristics (bold text, short length, capitalization)
3. ❌ **Font size analysis** - Not implemented

## Proposed Solution

Implement intelligent font-size based heading detection with relative sizing.

**Algorithm:**
1. Calculate document baseline font size (most common size)
2. Detect text significantly larger than baseline
3. Combine with existing heuristics (bold, length, capitalization)
4. Assign heading levels based on relative size

**Example Logic:**
```
Baseline: 11pt (most common in document)
16pt+ bold = Heading 1
14pt+ bold = Heading 2
12pt+ bold = Heading 3
```

## Implementation Plan

### 1. Infrastructure (Already Exists!)

```rust
// src/document.rs:38
pub struct TextFormatting {
    pub font_size: Option<f32>,  // Already available!
    // ... other fields
}
```

### 2. Add Baseline Calculation (src/document.rs)

```rust
fn calculate_baseline_font_size(doc: &Document) -> f32 {
    // Collect all font sizes from document
    // Return mode (most common) or median
    // Default to 11.0 if no data
}
```

### 3. Enhance Heading Detection (src/document.rs:223-224)

```rust
// TODO comment already marks the location!
fn detect_heading_from_text(
    text: &str,
    formatting: &TextFormatting,
    baseline_size: f32,
) -> Option<HeadingLevel> {
    // Existing heuristics (bold, short, caps)
    // NEW: Check if font_size significantly > baseline
    if let Some(size) = formatting.font_size {
        if formatting.bold {
            let ratio = size / baseline_size;
            if ratio >= 1.5 { return Some(HeadingLevel::H1); }
            if ratio >= 1.3 { return Some(HeadingLevel::H2); }
            if ratio >= 1.1 { return Some(HeadingLevel::H3); }
        }
    }
    // Fall back to existing logic
}
```

### 4. Integration

- Calculate baseline once during document parsing
- Pass baseline to heading detection function
- Keep existing heuristics as fallback

## Technical Considerations

**Dependencies:** None (uses existing `font_size` field)

**Files Affected:**
- `src/document.rs:detect_heading_from_text` (~line 223)
- `src/document.rs:parse_document` (add baseline calculation)

**Breaking Changes:** None (improves existing heuristics)

**Edge Cases:**
- Documents with no font size info → use existing heuristics
- Mixed font sizes throughout → median/mode calculation
- Mathematical documents with large equations → check bold + short text

## Testing Strategy

### Test Documents
1. Document with font-size-only headings (no styles)
2. Corporate template with 14pt headers
3. Academic paper with size-based sections
4. Mixed: Some styled headings, some font-size-only

### Acceptance Tests
```rust
#[test]
fn test_font_size_heading_detection() {
    // Parse document with 16pt bold text
    // Verify detected as Heading 1
    // Verify outline includes font-size headings
}
```

## Success Criteria

- [ ] Baseline font size calculated from document
- [ ] Font-size-based headings detected
- [ ] Outline view includes font-size headings
- [ ] Markdown export captures visual headings
- [ ] No regression in style-based heading detection
- [ ] Competitive advantage over pandoc documented

## Competitive Advantage

**Why This Matters:**
- Pandoc doesn't handle this at all
- Most terminal viewers miss visual headings
- Common in real-world corporate/academic documents
- Direct user pain point from Reddit feedback

## Priority

**Medium** - Clear competitive advantage, infrastructure exists, solves real user problem

## Estimated Effort

**4-6 hours** (1 day) - Straightforward, clear TODO already in code

## References

- Issue: https://github.com/bgreenwell/doxx/issues/3
- TODO comment: src/document.rs:223-224
- User feedback: Reddit thread (linked in issue)
- Pandoc limitation: No font-size-based heading detection
