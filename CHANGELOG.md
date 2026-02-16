# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- **List Bullet Formatting Bleed**: Fixed list bullets and numbers incorrectly inheriting text formatting from list items
  - List bullets/numbers were inheriting bold, italic, color, and strikethrough from the first word
  - Affected all export modes (ANSI, markdown, text) and TUI rendering
  - Root cause: List prefix was prepended to first run's text, inheriting all formatting properties
  - Solution: Created separate FormattedRun with default formatting for list prefixes
- **ANSI Formatting Bleed**: Fixed ANSI formatting codes bleeding into adjacent unformatted runs
  - Formatting from one text run was persisting into subsequent runs that should be unformatted
  - Example: Formatted "text" followed by unformatted "!" would render both formatted
  - Root cause: ANSI reset codes were only applied after all runs, not after each individual run
  - Solution: Added reset code at end of each formatted run for proper isolation
- **TUI Strikethrough Rendering**: Fixed strikethrough text not displaying in TUI mode
  - Strikethrough worked in all export modes but was invisible in interactive viewer
  - Added missing `Modifier::CROSSED_OUT` to paragraph and table cell rendering
  - Also added missing underline support for table cells

### Added
- **TUI Inline Image Display**: Complete terminal image rendering within the interactive viewer
  - Custom `DocumentWidget` with unified text and image rendering pipeline
  - Unicode-aware text wrapping using `unicode-segmentation` and `unicode-width` crates
  - Support for Kitty, iTerm2, and half-block fallback protocols via `ratatui-image`
  - Images render inline at correct positions within document flow
  - Proper handling of image dimensions and terminal constraints
  - Works seamlessly with existing features (search, navigation, scrolling)
  - Foundation for future enhancements (text selection, hyperlinks)

### Changed
- Refactored document rendering architecture with custom `DocumentWidget`
  - Single-pass rendering for improved performance
  - Better separation of concerns (rendering logic in widget module)
  - Removed 430+ lines of duplicate manual rendering code from `ui.rs`
  - Custom `render()` method with `Frame` access enables `StatefulImage` rendering

### Technical
- Integrated equation support from main branch into custom widget architecture
- All rendering (text, tables, images, equations) now unified in `DocumentWidget`
- Tests verified with `equations.docx`, `images.docx`, and other fixtures
- **Document Module Refactoring**: Completely restructured document parsing into focused modules
  - Split monolithic 2,611-line `document.rs` into 12 specialized modules
  - Created `document/parsing/` submodule with focused parsers (equations, tables, headings, lists, formatting, numbering)
  - Improved code organization with clear separation of concerns
  - Enhanced maintainability and extensibility for future development
  - All 59 tests passing with no functional changes
  - Removed compiler warnings and updated documentation

## [0.1.2] - 2025-10-21

### Added
- **Inline Equation Support**: Complete inline equation rendering within paragraph text
  - Inline equations now appear at correct positions within text (e.g., "text $A=\pi r^{2}$ more text")
  - Display equations remain as separate elements for proper mathematical presentation
  - Automatic detection of inline vs display equations based on OMML structure
  - LaTeX formatting with `$...$` delimiters for inline equations
  - Preserves exact ordering of text and equations within paragraphs
- **ANSI Export Format**: Rich terminal output with colors and formatting ([#45](https://github.com/bgreenwell/doxx/issues/45))
  - `--export ansi` option for ANSI-colored terminal output
  - `--terminal-width`/`-w` option for setting terminal width (default: $COLUMNS or 80)
  - `--color-depth {auto,1,4,8,24}` option for controlling color rendering depth
  - Perfect integration with terminal tools like `less -R`, fzf-tab, yazi, and ranger
  - Support for all formatting: bold, italic, underline, strikethrough, colors
- **Strikethrough Support**: Complete strikethrough text formatting with `~~text~~` syntax in all export formats ([#47](https://github.com/bgreenwell/doxx/issues/47))
- Search state toggle functionality - press `S` to hide/show search results ([#50](https://github.com/bgreenwell/doxx/pull/50)) by [@Jianchi-Chen](https://github.com/Jianchi-Chen)

### Fixed
- **Integration Tests for Packaging**: Fixed integration tests to use `CARGO_BIN_EXE` instead of hardcoded `cargo run` ([#60](https://github.com/bgreenwell/doxx/issues/60))
  - Tests now work in Debian packaging environments
  - Tests work with system-wide installed binaries
  - Faster test execution without recompilation
  - Follows Rust integration testing best practices
- **Text Formatting Preservation**: Fixed critical bug where text formatting (bold, italic, colors) was lost during inline equation processing
- **Word Automatic List Formatting**: Fixed formatting being lost in Word automatic numbered lists (affects strikethrough, bold, italic, colors)
- Empty search queries no longer match entire document, preventing performance issues ([#50](https://github.com/bgreenwell/doxx/pull/50)) by [@Jianchi-Chen](https://github.com/Jianchi-Chen)
- **File Type Validation**: Added proper validation to reject non-.docx files with helpful error messages ([#40](https://github.com/bgreenwell/doxx/issues/40), [#56](https://github.com/bgreenwell/doxx/issues/56))
  - Checks file extension is `.docx` before attempting to parse
  - Validates ZIP structure contains `word/document.xml`
  - Detects Excel files (`.xlsx`) specifically with clear error message: "This appears to be an Excel file"
  - Prevents hangs and crashes from invalid file types (Excel, ZIP archives, old Word `.doc` format)
  - Improves user experience with actionable error messages
- **Equation Positioning (Partial Fix)**: Improved display equation positioning in document flow ([#58](https://github.com/bgreenwell/doxx/issues/58))
  - Display equations now appear inline at their correct paragraph positions instead of all at document end
  - Added paragraph index tracking to equation extraction pipeline
  - Implemented `merge_display_equations()` function for intelligent equation placement
  - Successfully tested with user-provided equation documents
  - **Known Limitation**: docx-rs library doesn't parse equation-only paragraphs, so positioning may not be pixel-perfect in all cases
  - Full fix with complete XML parsing planned for v0.2.0

### Changed
- **Dependency Upgrade**: Updated `ratatui-image` from v1.0 to v8.0 for improved Debian packaging compatibility ([#59](https://github.com/bgreenwell/doxx/issues/59))
  - Addresses Debian package compilation issues
  - Updated API calls to match v8.0 interface (Picker initialization methods)
  - All image display functionality remains unchanged
- Help text updated to document new search state toggle functionality
- Improved error messages for invalid file formats with specific guidance
- Enhanced equation extraction to track paragraph positions for better document structure

### Documentation
- Addressed VirusTotal false positive detections with comprehensive explanation ([#46](https://github.com/bgreenwell/doxx/issues/46))

### Notes
- This release focuses on stability and critical bug fixes
- Terminal width text wrapping deferred to v0.2.0 ([#45](https://github.com/bgreenwell/doxx/issues/45) - requires full text wrapping feature implementation)
- All 47 tests passing across unit, integration, and specialized test suites

## [0.1.1] - 2025-08-22

### Added
- **Comprehensive release pipeline** with automated package manager support
  - Cross-platform binary builds (Linux, macOS Intel/ARM, Windows)
  - Automated crates.io publishing on release
  - Homebrew formula with automatic updates
  - SHA256 checksums for security verification
- **Enhanced installation options** in README with package manager instructions
- **Release automation script** (`scripts/release.sh`) for easy version management
- **Comprehensive release documentation** (RELEASE.md)

### Changed
- **Updated README** to use sentence case consistently throughout
- **Improved TUI image placeholder messages** to be clearer about functionality
- **Enhanced Markdown export** to use actual image paths instead of placeholder text
- **Modernized GitHub Actions** workflows for better reliability

### Fixed
- **Platform-specific image picker initialization** on Windows (clippy compatibility)
- **CSV export documentation** now clearly explains table-only extraction purpose

### Documentation
- **Added detailed command line options reference** with examples and use cases
- **Enhanced installation section** with multiple package manager options
- **Clarified CSV export purpose** for structured data extraction workflows

## [0.1.0] - Initial Release

### Added
- Basic `.docx` document parsing and viewing
- Terminal UI with navigation, search, and outline views
- Export functionality (text, markdown, JSON, CSV)
- Table parsing and rendering with enhanced formatting
- Document metadata extraction
- Search functionality with highlighting
- Comprehensive test suite with sample documents