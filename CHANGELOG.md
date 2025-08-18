# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Color Support for Text Rendering**: Added comprehensive color support with optional `--color` flag
  - **Color Detection**: Extracts hex color codes from Word documents (e.g., `#FF0000`, `#0066CC`)
  - **Terminal Rendering**: Converts hex colors to RGB terminal colors using ratatui
  - **Optional Flag**: `--color` enables color rendering (disabled by default for compatibility)
  - **Format Integration**: Works alongside existing bold, italic, and underline formatting
  - **Export Support**: Color information preserved in JSON exports regardless of flag
  - **Multiple Colors**: Supports any hex color from Word documents (red, blue, green, purple, etc.)
  - **Test Coverage**: Added comprehensive `color-showcase.docx` test document
  - **Current Limitation**: Single color per paragraph (mixed-color paragraphs show first color only)
- **Version Flag**: Added `--version` and `-V` command-line flags ([#19](https://github.com/bgreenwell/doxx/issues/19))
  - Display current version of doxx for bug reports and version verification
  - Automatically uses version from Cargo.toml (currently `0.1.0`)
  - Supports both long (`--version`) and short (`-V`) forms
  - Integrated with clap's built-in version handling
- **Copy to Clipboard Functionality**: Added comprehensive copy-to-clipboard support across all view modes
  - Copy full document content with `c` key in Document and Outline views
  - Copy search results with `F2` key in Search view to avoid input conflicts
  - Cross-platform clipboard support using `arboard` crate (Windows, macOS, Linux with X11/Wayland)
  - Smart content formatting based on current view:
    - Document view: Copies full document as formatted text with headings, lists, and tables
    - Outline view: Copies document structure with indented headings
    - Search view: Copies search results with context
  - Visual feedback with green status messages and error handling
  - Status messages auto-clear on next keypress
- **Enhanced Non-Interactive Mode**: Added content preview when running without interactive terminal
  - Shows document structure with proper heading hierarchy
  - Displays nested lists with correct indentation  
  - Previews first 20 elements with fallback message for longer documents
- **Force UI Mode**: Added `--force-ui` flag to bypass TTY detection for testing and development

### Fixed
- **CRITICAL: Unicode Safety Bug**: Fixed runtime panic when searching documents with emojis and special characters ([#22](https://github.com/bgreenwell/doxx/issues/22))
  - **Issue**: Unsafe string slicing at byte position 77 in search results could slice through Unicode character boundaries
  - **Root Cause**: Code used `&result.text[..77]` which panics when position 77 falls inside a multi-byte Unicode character (like emojis)
  - **Solution**: Implemented Unicode-safe truncation that finds valid UTF-8 boundaries before slicing
  - **Additional Fixes**: Audited and fixed all unsafe string slicing operations throughout codebase
    - `src/ui.rs`: Fixed search result truncation with safe boundary detection
    - `src/document.rs`: Fixed list prefix removal using `strip_prefix()` instead of direct slicing
    - Added comprehensive Unicode test suite with emojis, CJK text, and accented characters
  - **Test Coverage**: Created targeted reproduction tests and Unicode safety validation
  - **Impact**: Application now handles all Unicode content safely without runtime panics
- **Missing LICENSE File**: Added MIT license file to repository root ([#14](https://github.com/bgreenwell/doxx/issues/14))
  - **Issue**: Repository missing required LICENSE file for proper open source compliance
  - **Solution**: Created MIT license file and updated Cargo.toml to use single MIT license
  - **Compliance**: Now properly licensed for distribution and contribution
- **Numbered Headings Not Displaying**: Fixed Microsoft Word multilevel list headings not showing numbers ([#16](https://github.com/bgreenwell/doxx/issues/16))
  - **Issue**: Word documents using "Multilevel List" feature for headings showed content without numbering
  - **Root Cause**: Word's multilevel list numbering is stored differently than manual text numbering
  - **Solution**: Implemented automatic heading numbering generation with `HeadingNumberTracker`
    - Generates hierarchical numbering (1.0, 1.1, 1.2, 2.0, etc.) based on heading levels
    - Properly resets counters when returning to higher heading levels
    - Integrates with all export formats (markdown, text, JSON)
  - **Test Coverage**: Added comprehensive heading numbering tests
  - **Result**: Professional document structure display matching Word's original numbering
- **Word Automatic List Formatting**: Fixed automatic lists from Microsoft Word not rendering correctly ([#17](https://github.com/bgreenwell/doxx/issues/17))
  - **Issue**: Word's automatic lists (using numbering buttons) were parsed as headings instead of list items
  - **Root Cause**: Word stores list formatting in paragraph numbering properties (`w:numPr`), not as visible text
  - **Solution**: Added comprehensive Word numbering detection system
    - New `detect_list_from_paragraph_numbering()` function to extract list info from Word's numbering properties
    - Level-based list type detection for mixed list styles:
      - Level 0: Bullets (`*`) for unordered lists
      - Level 1: Letters (`a)`, `b)`, `c)`) for ordered sublists  
      - Level 2: Roman numerals (`i.`, `ii.`, `iii.`) for nested ordered lists
    - Smart formatting priority: Word numbering > heading styles > text heuristics
    - Marker system to prevent interference with existing text-based list processing
  - **Result**: Perfect rendering of Word's automatic mixed list formatting with proper nesting and indentation
- **Nested List Display in Interactive UI**: Fixed nested lists not showing proper indentation in terminal interface
  - Root cause: `trim: true` in ratatui Paragraph widget was removing leading whitespace
  - Solution: Changed to `trim: false` to preserve list indentation
  - Now correctly displays multi-level nested lists with proper visual hierarchy
- **Unicode Character Handling**: Fixed crash when cleaning list item text with Unicode bullets (•)
  - Replaced unsafe string slicing with `strip_prefix()` for proper Unicode handling

### Enhanced
- **Document Structure Display**: Significantly improved heading and numbering support
  - **Heading Numbering**: Added automatic hierarchical numbering for Word multilevel list headings
  - **Professional Output**: Documents now display with proper section numbering (1.0, 1.1, 2.0, etc.)
  - **Export Integration**: Heading numbers included in all export formats (markdown, text, JSON)
  - **Unicode Safety**: All text processing now handles Unicode characters safely
- **Word Document Compatibility**: Significantly improved handling of Microsoft Word documents
  - Enhanced paragraph numbering property parsing for automatic lists
  - Better integration between Word's native formatting and terminal rendering
  - Improved support for complex nested list structures from Word documents
- **List Processing Pipeline**: Improved list detection and grouping logic
  - Enhanced `group_list_items()` function to properly combine consecutive list items
  - Better level calculation based on leading whitespace (2 spaces = 1 level)
  - Improved text cleaning for various bullet styles (•, -, *, numbered lists)
  - Smart processing to avoid conflicts between Word automatic lists and text-based lists
- **Help Documentation**: Updated help system with copy functionality instructions
  - Added copy shortcuts to navigation help bar
  - Enhanced help overlay with detailed copy instructions for each view mode
  - Context-aware help showing different shortcuts for different views

### Dependencies
- **Tokio Optimization**: Reduced tokio feature set from `"full"` to specific features (`"rt-multi-thread"`, `"macros"`, `"fs"`)
  - Significantly reduces binary size and compilation time
  - Only includes necessary async runtime features for current functionality
  - Prepares foundation for future AI integration features
- Added `arboard ^3.3` for cross-platform clipboard functionality
- Updated `ratatui` from `0.26` to `0.29` for latest terminal UI features and API compatibility
- Fixed deprecated GitHub Actions: updated upload/download-artifact@v3→v4, codecov-action@v3→v4, action-gh-release@v1→v2
- Fixed overly specific test assertion in `test_formatting_markdown_export` that was causing CI failures

### Changed
- **Build Optimizations**: Enabled Link-Time Optimization (LTO) for release builds ([#1](https://github.com/bgreenwell/doxx/issues/1))
  - Added `codegen-units = 1` and `lto = true` to `[profile.release]`
  - Reduced binary size from ~5MB to ~3MB (up to 40% smaller)
  - Improved runtime performance through better optimization
  - Only affects release builds, keeping development builds fast
  - Thanks to @zamazan4ik for the suggestion!

### Infrastructure
- **GitHub Pages Deployment**: Fixed documentation deployment workflow
  - **Issue**: GitHub Actions deployment failing with "Pages site failed" error
  - **Solution**: Added `enablement: true` and proper error handling in workflow
  - **Enhancement**: Improved deployment robustness with continue-on-error and helpful status messages
  - **Result**: Automated Rust documentation deployment now working reliably
- **Code Quality**: Enhanced linting and formatting configuration
  - Added `rustfmt.toml` for consistent code formatting
  - Added `clippy.toml` for enhanced linting rules with MSRV support
  - Fixed all clippy warnings including type complexity, format string issues, and dead code warnings
  - Cleaned up unused struct fields and improved code organization
- **CI/CD Pipeline**: Implemented robust GitHub Actions workflows
  - Multi-platform testing (Linux, Windows, macOS) with different Rust versions
  - Automated security auditing with cargo-audit and cargo-deny
  - Code coverage reporting with cargo-llvm-cov and Codecov integration
  - Dependency update framework prepared (Dependabot config ready when needed)
  - Release automation with cross-platform binary builds
- **Developer Experience**: Enhanced development workflow
  - Added pre-commit hooks configuration for quality gates
  - Updated .gitignore with comprehensive Rust project exclusions
  - Added CI status badges to README
  - Configured cargo-deny for supply chain security
  - Prepared Dependabot configuration (disabled during early development to avoid PR spam)

## [0.1.0] - Initial Release

### Added
- Basic `.docx` document parsing and viewing
- Terminal UI with navigation, search, and outline views
- Export functionality (text, markdown, JSON, CSV)
- Table parsing and rendering with enhanced formatting
- Document metadata extraction
- Search functionality with highlighting
- Comprehensive test suite with sample documents