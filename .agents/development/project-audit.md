# doxx Project Audit - February 2026

Combined analysis from Claude Code and Gemini CLI agents.

## Executive Summary

**Status:** Healthy and active project with strong community engagement
**Version:** v0.1.2 (Oct 21, 2025)
**Community:** 3,554 stars | 85 forks | 10 open issues | 0 open PRs
**Unreleased Work:** Major features ready for v0.2.0 release

## Recent Activity

### Merged Since v0.1.2 (Unreleased)
1. **TUI Inline Images** (#63, Oct 26) - Complete terminal image rendering in TUI
2. **List Bullet Formatting Bleed Fix** (#64, Oct 26) - Fixed formatting inheritance
3. **ANSI Formatting Bleed Fix** (Oct 26) - Proper ANSI reset codes
4. **TUI Strikethrough** (Oct 26) - Added missing strikethrough rendering
5. **Debian Documentation** (#68, Dec 12) - Package information

**Impact:** ~430+ lines of code removed via refactoring (DocumentWidget architecture)

## Open Issues Prioritization

### 🔴 Critical

**#65 - Test Failing** (reopened Oct 28)
- `terminal_image::tests::test_renderer_creation` fails
- Error: `assertion failed: renderer.max_width > 0`
- Passes locally, may be CI environment-specific
- **Action:** Investigate or mark as flaky

### 🟡 High Priority

**#26 - Configurable Keyboard Shortcuts** (Aug 20)
- vim/less-style keybindings requested
- User wants: u/d, ctrl-u/ctrl-d, /, n/N, H/L
- Planned for v0.2.0
- **Impact:** High user demand, improves UX for all users

**#67 - Table Cell Merge** (Nov 10)
- Cells with `<w:gridSpan>` not rendering correctly
- User provided XML example
- **Technical:** Requires docx-rs parsing enhancement

### 🟢 Medium Priority

**#66 - Remember Last Position** (Nov 3)
- Bookmark/state persistence
- **Implementation:** Save to `~/.config/doxx/positions.toml`
- Straightforward UX enhancement

**#45 - ANSI Export** (reopened Aug 28)
- **Status:** IMPLEMENTED in v0.1.2 but issue reopened
- **Action:** Verify requirements met, possibly terminal width improvements

**#35 - Kitty Graphics Protocol** (Aug 24)
- **Status:** IMPLEMENTED via ratatui-image
- **Action:** Verify and close

### 🔵 Low Priority (Backlog)

**#3 - Font-Size Based Heading Detection** (Aug 17)
- Infrastructure exists (`font_size` field in `TextFormatting`)
- TODO comment at src/document.rs:223-224
- **Advantage:** Competitive edge over pandoc

**#24 - Advanced Numbering** (Aug 19)
- Legacy documents with style-based numbering
- User-provided test document
- Modern documents work fine

**#13 - Text Selection & Copy** (Aug 18)
- Comprehensive feature request
- Multiple implementation options (mouse vs keyboard)
- Significant UX work, v0.3.0+ candidate

**#37 - Yazi Plugin** (Aug 25)
- Community contribution opportunity
- Not core doxx development

## Architecture Analysis (Gemini)

### Code Quality Issues

**Monolithic Parser** (2,600 lines in `src/document.rs`)
- Handles data structures, DOCX parsing, numbering, search, outline
- **Recommendation:** Refactor into `src/document/` module with:
  - `parser.rs` - DOCX parsing logic
  - `numbering.rs` - Hierarchical numbering
  - `search.rs` - Search functionality
  - `models.rs` - Data structures

**Config Stubs** (`src/main.rs`)
- `ConfigCommands` stubs without implementation
- **Recommendation:** Complete TODOs using `dirs` and `toml` crates (already in dependencies)

**XML Workarounds**
- Uses `quick-xml` for OMML equation parsing
- Supplements `docx-rs` limitations
- Necessary until upstream adds OMML support

### UI/UX Issues

**Element-Based Scrolling**
- Scrolls by element index, not lines
- Causes "jumps" when navigating large tables/paragraphs
- **Recommendation:** Implement virtual line mapping in `DocumentWidget`

**Custom Widget Pattern**
- `DocumentWidget` bypasses standard `ratatui` traits for `Frame` access
- Enables image rendering but non-idiomatic
- **Action:** Investigate if ratatui-image 8.0+ allows proper `Widget` trait implementation

### Feature Gaps

**Search Enhancements**
- Current: Basic case-insensitive literal matching
- **Recommendation:** Add regex support, "match whole word" toggle

**Table Navigation**
- Wide tables overflow without horizontal scrolling
- **Recommendation:** Add table view mode or better horizontal navigation

### Performance & Testing

**Strengths:**
- Strong integration test suite
- Good use of LTO and codegen-units for release builds
- <100ms startup time ✅

**Concerns:**
- Parallel parsing with `rayon` suggested for large documents
  - **Counter:** Current performance excellent, profile before adding complexity
- More unit tests needed in `document.rs` modules
  - **Counter:** Integration tests provide good coverage

**Security:**
- No hardcoded secrets ✅
- Modern, maintained dependencies ✅
- `ImageExtractor` temp files persist during session (cleanup needed for long-running processes)

## Prioritization Strategy

### Immediate (This Week)
1. **Fix test #65** - Unblock CI/CD (1-2 days)
2. **Issue triage** - Close #35, #45 if complete
3. **v0.2.0 release** - Ship unreleased features (1 week)

### Short Term (2-4 Weeks)
4. **Keyboard shortcuts #26** - High user demand, v0.2.0 planned
   OR
5. **Table cell merge #67** - Recent request, user-provided example

### Medium Term (v0.2.1+)
6. **Position memory #66** - Straightforward UX win
7. **Font-size heading detection #3** - Competitive advantage
8. **Config system implementation** - Complete stubs
9. **Line-based scrolling** - Smoother UX

### Future (v0.3.0+)
10. **Refactor document.rs** - Split into modules
11. **Enhanced search** - Regex, whole-word matching
12. **Text selection #13** - Complex UX feature
13. **Advanced numbering #24** - Low ROI, legacy docs only

## Technical Debt Assessment

### Critical
- Test #65 failure (blocks CI)

### Important
- Config stubs incomplete
- Element-based scrolling UX issue

### Nice-to-Have
- `document.rs` refactoring
- Enhanced search capabilities
- ImageExtractor cleanup mechanism

### Not Urgent
- Parallel parsing (performance already excellent)
- Advanced numbering (affects legacy docs only)

## Recommendation

**Ship user-facing features first, then refactor.**

**Rationale:**
1. Code works well (3.5k stars, active community)
2. Premature refactoring before understanding long-term needs
3. User-facing features have clearer ROI
4. Refactoring can happen incrementally

**Prioritize:** #26 (keyboard shortcuts) → #67 (table merge) → #66 (position memory) → #3 (font-size headings)

**Then consider:** Config implementation, line-based scrolling, document.rs refactoring

## Success Metrics

- Reduce open issues from 10 to <5
- Release v0.2.0 with unreleased features
- Maintain test coverage and CI health
- Sustain community engagement (3.5k+ stars)

## Next Actions

1. Investigate #65 test failure
2. Verify #35 and #45 can be closed
3. Update CHANGELOG.md for v0.2.0
4. Create release tag
5. Begin work on #26 (keyboard shortcuts)

---

*Audit performed by Claude Code and Gemini CLI agents, February 9, 2026*
