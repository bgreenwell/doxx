# Known Issues & Development Constraints

This document tracks known limitations, bugs, and architectural constraints that affect development.

## Active Issues

### Equation Positioning (#58) 🔴 High Priority

**Problem:**
Display equations may not appear at exact positions in the document due to limitations in the docx-rs parsing library. The library doesn't parse equation-only paragraphs, causing index alignment issues between our document model and the actual file structure.

**Impact:**
- Display equations may appear out of order
- Equation context may be misaligned with surrounding text
- Affects documents with complex mathematical content

**Workaround:**
Currently using direct XML parsing for some cases, but incomplete solution.

**Fix Plan:**
- Full XML-based parsing for accurate placement
- Planned for v0.2.0 with complete OMML (Office Math Markup Language) parsing
- Requires custom XML parsing layer independent of docx-rs

**References:**
- GitHub Issue: #58
- Upstream feature request: https://github.com/bokuweb/docx-rs/issues

### Text Wrapping (#45) 🟡 Medium Priority

**Problem:**
Terminal width text wrapping not yet implemented. Text doesn't adapt to terminal width, causing poor UX on narrow terminals or when piping output.

**Impact:**
- Wide tables/text overflow terminal boundaries
- Poor experience on small terminals
- ANSI export doesn't respect terminal width properly

**Fix Plan:**
- Implement paragraph wrapping system
- Detect terminal width dynamically
- Smart word-breaking for tables and code blocks
- Planned for v0.2.0

**References:**
- GitHub Issue: #45

### Advanced Numbering (#24) 🟢 Low Priority

**Problem:**
Some legacy Word documents with style-based numbering may not display correctly. The issue affects documents that use implicit numbering derived from paragraph styles rather than explicit numbering properties.

**Impact:**
- Legacy document numbering may be incorrect
- Style-based lists might not render properly
- Modern documents with explicit numbering work fine

**Workaround:**
Most modern .docx files use explicit numbering and are unaffected. Recommend users save legacy documents in modern format.

**Fix Plan:**
- Style-based detection for legacy documents
- Better inference of numbering from paragraph styles
- Consider for v0.2.0+

**References:**
- GitHub Issue: #24

## Roadmap Context

### v0.2.0 Priorities
1. **Perfect equation positioning** - Full XML-based parsing (#58)
2. **Terminal width text wrapping** - Paragraph wrapping system (#45)
3. **Configurable keyboard shortcuts** - vim/less-style keybindings (#26)

### Future Considerations
- Image support in TUI (works in export, needs TUI integration) (#35)
- Text selection and copy enhancements (#13)
- Font-size based heading detection (#3)
- Hyperlink navigation
- Custom themes
- Bookmark system

## Resources

- **GitHub Issues**: https://github.com/bgreenwell/doxx/issues
- **Milestones**: https://github.com/bgreenwell/doxx/milestones
- **Project Board**: https://github.com/bgreenwell/doxx/projects
