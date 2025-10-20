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
- **Status**: ⏳ IN PROGRESS

#### ✅ #45 - Fix `-w` terminal width flag
- **Problem**: `-w` flag doesn't change output width in ANSI export
- **Impact**: MEDIUM - Feature is incomplete
- **Effort**: LOW
- **Status**: 🔍 TODO

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

#### ⚠️ #58 - Equations appearing at bottom
- **Problem**: Display equations render at end of document instead of inline
- **Impact**: MEDIUM-HIGH
- **Effort**: MEDIUM (needs debugging)
- **Status**: 🔍 INVESTIGATE (user provided test doc)
- **Decision**: Include if quick fix, otherwise defer

#### ⚠️ #26 - Configurable keyboard shortcuts
- **Problem**: Users want vim/less-style keybindings
- **Impact**: MEDIUM - Quality of life
- **Effort**: MEDIUM - Requires keymap refactor
- **Status**: 🎯 Planned for this release per prior commitment
- **Decision**: Include if time allows

### DEFER TO v0.2.0:

❌ **#24** - Advanced numbering improvements (HIGH effort, complex)
❌ **#35** - Kitty graphics in TUI (works in export, TUI is complex)
❌ **#13** - Text selection/copy (HIGH effort)
❌ **#3** - Font-size based heading detection (MEDIUM effort, low priority)
❌ **#37** - Yazi plugin (external, community can build)
❌ **#31** - WebAssembly support (unclear use case, not CLI focus)

## Implementation Checklist

### Phase 1: Critical Fixes
- [ ] Implement file type validation (#40)
  - [ ] Add `validate_docx_file()` function
  - [ ] Check extension is `.docx`
  - [ ] Check ZIP contains `word/document.xml`
  - [ ] Add helpful error messages
  - [ ] Test with .xlsx, .zip, .doc files
- [ ] Fix `-w` terminal width flag (#45)
  - [ ] Debug why width parameter isn't being used
  - [ ] Test with various width values
  - [ ] Verify markdown wrapping works

### Phase 2: Issue Management
- [ ] Post explanation on #46 (false positive)
- [ ] Close #56 as duplicate of #40

### Phase 3: Optional (Time Permitting)
- [ ] Investigate #58 equation positioning
- [ ] Implement #26 keyboard shortcuts (if feasible)

### Phase 4: Release Prep
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
- [ ] File validation rejects .xlsx files with helpful error
- [ ] File validation rejects .zip files with helpful error
- [ ] File validation rejects .doc files with helpful error
- [ ] File validation accepts valid .docx files
- [ ] Terminal width flag works correctly

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
- **Terminal Width Flag**: Fixed `-w`/`--terminal-width` flag in ANSI export mode (#45)
- **Security**: Addressed VirusTotal false positive detections with documentation (#46)

#### Changed
- Improved error messages for invalid file formats

#### Notes
- This release focuses on stability and bug fixes
- All integration tests now work with Debian packaging (#60)
- Text formatting preservation from v0.1.1 continues to work

## Post-Release

### Immediate Actions
- [ ] Close fixed issues
- [ ] Announce release
- [ ] Monitor for new issue reports

### v0.2.0 Planning
Consider for next major release:
- Advanced numbering improvements (#24)
- Kitty graphics in TUI (#35)
- Text selection and copy (#13)
- Keyboard shortcuts if not in v0.1.2 (#26)
- Font-size based heading detection (#3)
