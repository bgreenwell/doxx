# Release v0.1.2 Planning

**Target Date**: TBD
**Branch**: `prep-release-v0.1.2`

## Release Goals

Fix critical bugs and polish existing features for a stable release suitable for wider adoption.

## Issues Analysis

### MUST FIX (Blocking Release):

#### ✅ #40 - File type validation (xlsx/zip crashes)
- **Problem**: App hangs when given .xlsx or .zip files
- **Impact**: HIGH - Causes confusion and system resource consumption
- **Effort**: LOW
- **Solution**: Validate file extension and ZIP structure before parsing
- **Status**: ✅ COMPLETED

#### ❌ #45 - Fix `-w` terminal width flag
- **Problem**: `-w` flag doesn't change output width in ANSI export
- **Impact**: MEDIUM - Feature is incomplete
- **Effort**: HIGH - Requires text wrapping implementation
- **Findings**: Flag only affects separators, not paragraph text. Needs full text wrapping feature.
- **Status**: 🔄 DEFER TO v0.2.0

#### ✅ #46 - VirusTotal false positives
- **Problem**: 3/69 vendors flag Windows binary
- **Impact**: LOW - Just needs explanation
- **Solution**: Post detailed explanation, close issue
- **Status**: 📝 TODO (comment only)

#### ✅ #56 - Better error for .doc files
- **Problem**: Confusing error when opening old .doc format
- **Impact**: LOW
- **Solution**: Duplicate of #40, will be fixed automatically
- **Status**: 🔄 Close as duplicate

### NICE TO HAVE (If Time Permits):

#### ✅ #58 - Equations appearing at bottom (PARTIAL FIX)
- **Problem**: Display equations render at end of document instead of inline
- **Impact**: MEDIUM-HIGH
- **Effort**: MEDIUM (architectural limitation discovered)
- **Status**: ✅ PARTIAL FIX IMPLEMENTED
- **What was fixed**:
  - Added paragraph index tracking to equation extraction
  - Created merge function to insert equations at approximate positions
  - Equations now appear inline rather than all at end
- **Known limitation**:
  - docx-rs library doesn't parse equation-only paragraphs
  - Positioning not pixel-perfect for all documents
  - Works well for most cases, some edge cases remain
- **Next steps**: Full XML-based parsing for perfect positioning (v0.2.0)

#### ⚠️ #26 - Configurable keyboard shortcuts
- **Problem**: Users want vim/less-style keybindings
- **Impact**: MEDIUM - Quality of life
- **Effort**: MEDIUM - Requires keymap refactor
- **Status**: 🎯 Planned for this release per prior commitment
- **Decision**: Include if time allows

### DEFER TO v0.2.0:

❌ **#45** - Terminal width text wrapping (HIGH effort, requires new wrapping system)
❌ **#58** - Perfect equation positioning (MEDIUM effort, needs full XML parsing)
❌ **#24** - Advanced numbering improvements (HIGH effort, complex)
❌ **#35** - Kitty graphics in TUI (works in export, TUI is complex)
❌ **#13** - Text selection/copy (HIGH effort)
❌ **#3** - Font-size based heading detection (MEDIUM effort, low priority)
❌ **#37** - Yazi plugin (external, community can build)
❌ **#31** - WebAssembly support (unclear use case, not CLI focus)

## Implementation Checklist

### Phase 1: Critical Fixes ✅ COMPLETED
- [x] Implement file type validation (#40)
  - [x] Add `validate_docx_file()` function
  - [x] Check extension is `.docx`
  - [x] Check ZIP contains `word/document.xml`
  - [x] Add helpful error messages
  - [x] Test with .xlsx, .zip, .doc files
- [x] Investigate `-w` terminal width flag (#45)
  - [x] Debug why width parameter isn't being used
  - [x] Determined requires full text wrapping implementation
  - [x] Decision: Defer to v0.2.0

### Phase 2: Equation Positioning ✅ COMPLETED (PARTIAL)
- [x] Investigate #58 equation positioning
  - [x] Add paragraph index tracking
  - [x] Implement merge function
  - [x] Test with user-provided document
  - [x] Document known limitations
  - [x] Plan full fix for v0.2.0

### Phase 3: Issue Management
- [ ] Post explanation on #46 (false positive)
- [ ] Close #56 as duplicate of #40

### Phase 4: Optional (Time Permitting)
- [ ] Implement #26 keyboard shortcuts (if feasible)

### Phase 5: Release Prep
- [ ] Update CHANGELOG.md
- [ ] Run full test suite
- [ ] Test with problematic documents from issues
- [ ] Update version to 0.1.2 in Cargo.toml
- [ ] Create release notes
- [ ] Merge to main
- [ ] Tag release

## Testing Plan

### Regression Testing
- [ ] All existing tests pass
- [ ] Formatting still works (bold, italic, colors)
- [ ] ANSI export works
- [ ] Image support works
- [ ] Equation support works

### New Feature Testing
- [x] File validation rejects .xlsx files with helpful error
- [x] File validation rejects .zip files with helpful error
- [x] File validation rejects .doc files with helpful error (via extension check)
- [x] File validation accepts valid .docx files
- [x] Equation positioning improved (inline vs all at end)

### Document Testing
- [ ] business-report.docx
- [ ] example.docx
- [ ] equations document (#58 if investigating)
- [ ] User-provided documents from issues

## Release Notes Draft

### v0.1.2

**Release Date**: TBD

#### Fixed
- **File Type Validation**: Added proper validation to reject non-.docx files (Excel, ZIP, old Word .doc) with helpful error messages (#40, #56)
  - Checks file extension is `.docx`
  - Validates ZIP structure contains `word/document.xml`
  - Detects Excel files specifically with clear error messages
  - Prevents hangs and crashes from invalid file types
- **Equation Positioning (Partial)**: Improved display equation positioning (#58)
  - Equations now appear inline in document flow instead of all at end
  - Added paragraph index tracking for better positioning
  - Known limitation: Some edge cases may not have pixel-perfect positioning
  - Full fix planned for v0.2.0 with complete XML parsing
- **Security**: Addressed VirusTotal false positive detections with documentation (#46)

#### Changed
- Improved error messages for invalid file formats
- Enhanced equation extraction to track paragraph positions

#### Notes
- This release focuses on stability and bug fixes
- All integration tests now work with Debian packaging (#60)
- Text formatting preservation from v0.1.1 continues to work
- Terminal width text wrapping deferred to v0.2.0 (#45 - requires larger feature implementation)

## Post-Release

### Immediate Actions
- [ ] Close fixed issues
- [ ] Announce release
- [ ] Monitor for new issue reports

### v0.2.0 Planning
High priority for next release:
- **Perfect equation positioning** (#58) - Full XML-based parsing for accurate placement
- **Terminal width text wrapping** (#45) - Implement paragraph wrapping system
- **Configurable keyboard shortcuts** (#26) - vim/less-style keybindings

Consider for v0.2.0 or later:
- Advanced numbering improvements (#24)
- Kitty graphics in TUI (#35)
- Text selection and copy (#13)
- Font-size based heading detection (#3)
