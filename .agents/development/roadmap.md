# doxx Development Roadmap

Feature prioritization and timeline for doxx development.

## v0.2.0 - In Progress (Target: February 2026)

### Ready to Ship (Merged, Unreleased)
- ✅ TUI inline images via DocumentWidget
- ✅ List bullet formatting bleed fix
- ✅ ANSI formatting bleed fix
- ✅ TUI strikethrough rendering

### Blockers
- 🔴 **#65**: Fix failing test (`terminal_image::tests::test_renderer_creation`)

### Planned Features
- **#26**: Configurable keyboard shortcuts (vim/less-style keybindings)
- Config system implementation (complete stubs in `src/main.rs`)

**Timeline:** Ship after #65 is resolved

---

## v0.2.1 - Near Term (March-April 2026)

### Feature Enhancements
- **#67**: Table cell merge support (`<w:gridSpan>` parsing)
- **#66**: Remember last position (state persistence)
- **#3**: Font-size based heading detection

### UX Improvements
- Line-based scrolling (replace element-based navigation)
- Horizontal scrolling for wide tables

**Effort:** 3-4 weeks total
**Priority:** User experience and feature completeness

---

## v0.3.0 - Medium Term (Q2 2026)

### Major Features
- **#13**: Text selection and selective copy
  - Mouse-based or keyboard-based selection
  - Visual mode like vim

### Architecture Improvements
- Refactor `src/document.rs` into module directory:
  - `document/parser.rs`
  - `document/numbering.rs`
  - `document/search.rs`
  - `document/models.rs`

### Search Enhancements
- Regex support
- "Match whole word" toggle
- Case-sensitive option

**Effort:** 4-6 weeks
**Priority:** Architecture stability for future growth

---

## v0.4.0 - Long Term (Q3-Q4 2026)

### Advanced Features
- **#24**: Advanced numbering (style-based detection for legacy documents)
- Hyperlink navigation
- Custom themes/color schemes
- Bookmark system

### Integration
- **#37**: Yazi plugin (community contribution)
- lessopen integration improvements
- fzf-tab enhancements

**Effort:** 8-10 weeks
**Priority:** Polish and ecosystem integration

---

## Backlog (Future Consideration)

### Performance
- Parallel parsing with `rayon` (only if benchmarks show need)
- Incremental rendering for very large documents
- Memory optimization for embedded scenarios

### Document Support
- Merged table cells (complex layouts)
- Hyperlinks with protocol handlers
- Comments and track changes rendering

### Developer Experience
- Plugin system for custom renderers
- Programmatic API for library usage

**Timeline:** As needed based on user feedback

---

## Issue Closure Plan

### Can Close Now
- **#35**: Kitty Graphics Protocol (implemented via ratatui-image)
- **#45**: ANSI Export (implemented in v0.1.2, verify requirements)

### Will Close with v0.2.0
- **#26**: Configurable keyboard shortcuts
- **#65**: Test failure (once fixed)

### Will Close with v0.2.1
- **#3**: Font-size heading detection
- **#66**: Position memory
- **#67**: Table cell merge

### Long-Term
- **#13**: Text selection (v0.3.0)
- **#24**: Advanced numbering (v0.4.0)
- **#37**: Yazi plugin (community-driven)

---

## Milestone Targets

| Version | Target Date | Key Features | Status |
|---------|-------------|--------------|--------|
| v0.2.0 | Feb 2026 | TUI images, keyboard shortcuts, bug fixes | In Progress |
| v0.2.1 | Apr 2026 | Table merge, position memory, font-size headings | Planned |
| v0.3.0 | Jun 2026 | Text selection, search enhancements, refactoring | Planned |
| v0.4.0 | Q4 2026 | Advanced numbering, theming, bookmarks | Future |

---

## Community Engagement

**Current:** 3,554 stars, 85 forks, active issue discussions
**Goal:** Maintain momentum, encourage contributions

**Opportunities:**
- Good first issues: #3 (font-size headings), config implementation
- Community projects: #37 (Yazi plugin)
- Documentation: Integration guides for lessopen, fzf-tab, ranger

---

*Last updated: February 9, 2026*
