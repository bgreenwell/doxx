# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
- **Nested List Display in Interactive UI**: Fixed nested lists not showing proper indentation in terminal interface
  - Root cause: `trim: true` in ratatui Paragraph widget was removing leading whitespace
  - Solution: Changed to `trim: false` to preserve list indentation
  - Now correctly displays multi-level nested lists with proper visual hierarchy
- **Unicode Character Handling**: Fixed crash when cleaning list item text with Unicode bullets (•)
  - Replaced unsafe string slicing with `strip_prefix()` for proper Unicode handling

### Enhanced
- **List Processing Pipeline**: Improved list detection and grouping logic
  - Enhanced `group_list_items()` function to properly combine consecutive list items
  - Better level calculation based on leading whitespace (2 spaces = 1 level)
  - Improved text cleaning for various bullet styles (•, -, *, numbered lists)
- **Help Documentation**: Updated help system with copy functionality instructions
  - Added copy shortcuts to navigation help bar
  - Enhanced help overlay with detailed copy instructions for each view mode
  - Context-aware help showing different shortcuts for different views

### Dependencies
- Added `arboard ^3.3` for cross-platform clipboard functionality
- Updated `ratatui` from `0.26` to `0.29` for latest terminal UI features and API compatibility

### Changed
- **Build Optimizations**: Enabled Link-Time Optimization (LTO) for release builds ([#1](https://github.com/bgreenwell/doxx/issues/1))
  - Added `codegen-units = 1` and `lto = true` to `[profile.release]`
  - Reduced binary size from ~5MB to ~3MB (up to 40% smaller)
  - Improved runtime performance through better optimization
  - Only affects release builds, keeping development builds fast
  - Thanks to @zamazan4ik for the suggestion!

### Infrastructure
- **Code Quality**: Added comprehensive linting and formatting configuration
  - Added `rustfmt.toml` for consistent code formatting
  - Added `clippy.toml` for enhanced linting rules with MSRV support
  - Fixed all clippy warnings including type complexity and format string issues
- **CI/CD Pipeline**: Implemented robust GitHub Actions workflows
  - Multi-platform testing (Linux, Windows, macOS) with different Rust versions
  - Automated security auditing with cargo-audit and cargo-deny
  - Code coverage reporting with cargo-llvm-cov and Codecov integration
  - Automated dependency updates via Dependabot
  - Release automation with cross-platform binary builds
- **Developer Experience**: Enhanced development workflow
  - Added pre-commit hooks configuration for quality gates
  - Updated .gitignore with comprehensive Rust project exclusions
  - Added CI status badges to README
  - Configured cargo-deny for supply chain security

## [0.1.0] - Initial Release

### Added
- Basic `.docx` document parsing and viewing
- Terminal UI with navigation, search, and outline views
- Export functionality (text, markdown, JSON, CSV)
- Table parsing and rendering with enhanced formatting
- Document metadata extraction
- Search functionality with highlighting
- Comprehensive test suite with sample documents