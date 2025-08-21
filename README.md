# doxx 📄

> Expose `.docx` files in your terminal — no Microsoft Word required

[![CI](https://github.com/bgreenwell/doxx/workflows/CI/badge.svg)](https://github.com/bgreenwell/doxx/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

> **🚧 Under Active Development**: New features and improvements are being added regularly. Check back often for updates!

**doxx** is a lightning-fast, terminal-native document viewer for Microsoft Word files. Built with Rust for performance and reliability, it brings Word documents to your command line with beautiful rendering, smart table support, and powerful export capabilities.

![doxx screenshot](assets/doxx-screenshot.png)

## ✨ Features

### Document viewing
- 🎨 **Beautiful terminal rendering** with syntax highlighting and formatting
- 🌈 **Color support** with `--color` flag for Word document text colors (red, blue, green, etc.)
- 📊 **Professional table display** with smart alignment and Unicode borders
- 📋 **Nested list support** with proper indentation and multi-level hierarchy
- 🔍 **Full-text search** with highlighting and context
- 📑 **Document outline** for quick navigation
- 🎯 **Multiple view modes** — document, outline, search, and help

### Smart table support
- 📋 **Advanced table parsing** with automatic header detection
- 🎯 **Intelligent alignment** — numbers right-aligned, text left-aligned, booleans centered  
- 💱 **Data type detection** for currency, percentages, dates, and more
- 🎨 **Professional ASCII rendering** with scalable Unicode borders
- 🔄 **Search within tables** across headers and cell content

### Copy & clipboard
- 📋 **Copy to clipboard** - Copy rendered content directly from the terminal UI
- 🎯 **Context-aware copying** - Different content based on current view:
  - **Document view**: Copy full formatted document with headings, lists, and tables
  - **Outline view**: Copy document structure with indented headings
  - **Search view**: Copy search results with context
- 🖥️ **Cross-platform**: Works on Windows, macOS, and Linux (X11/Wayland)
- ✅ **Visual feedback** with status messages and error handling

### Export & integration
- 📝 **Markdown export** with proper table alignment indicators
- 📊 **CSV extraction** for data analysis workflows
- 📄 **Plain text** output for piping to other tools
- 🗂️ **JSON export** with full document structure
- ⚡ **CLI-friendly** for scripts and automation

## 🚀 Quick start

### Installation

#### 📦 Pre-built binaries (recommended)

**doxx** provides pre-built binaries for all major platforms via GitHub releases:

| Platform | Architecture | Download |
|----------|-------------|----------|
| 🪟 **Windows** | x86_64 | `doxx-windows-x86_64.zip` |
| 🐧 **Linux** | x86_64 (musl) | `doxx-linux-x86_64.tar.gz` |
| 🍎 **macOS** | Intel (x86_64) | `doxx-macos-x86_64.tar.gz` |
| 🍎 **macOS** | Apple Silicon (ARM64) | `doxx-macos-arm64.tar.gz` |

```bash
# Download the latest release for your platform
curl -L https://github.com/bgreenwell/doxx/releases/latest/download/doxx-[platform].tar.gz | tar xz

# Move to your PATH
sudo mv doxx /usr/local/bin/

# Verify installation
doxx --version
```

**✅ Supported operating systems:**
- Windows 10/11 (x86_64)
- Linux distributions (x86_64, musl-based for maximum compatibility)
- macOS 10.12+ (Intel and Apple Silicon)

#### 🍺 Package managers (coming soon!)

We're working on official package manager support:

- 🍺 **Homebrew** (macOS/Linux): `brew install doxx` - *coming soon*
- 📦 **Chocolatey** (Windows): `choco install doxx` - *coming soon*  
- 🦀 **Cargo** (all platforms): `cargo install doxx` - *coming soon*
- 🐧 **APT/YUM** (Linux): Official repo packages - *coming soon*

#### 🔧 Build from source

For the latest development features or unsupported platforms:

```bash
# Install from source (requires Rust 1.70+)
git clone https://github.com/bgreenwell/doxx.git
cd doxx
cargo install --path .

# Or install directly from Git
cargo install --git https://github.com/bgreenwell/doxx.git
```

**Requirements:**
- Rust 1.70 or later
- Git for cloning the repository

### Basic usage

```bash
# View a document
doxx quarterly-report.docx

# Start with outline view
doxx document.docx --outline

# Search for specific content
doxx contract.docx --search "payment terms"

# Enable color support for Word text colors
doxx presentation.docx --color

# Export to different formats
doxx spreadsheet.docx --export csv > data.csv
doxx report.docx --export markdown > report.md
doxx document.docx --export json > structure.json

# Force interactive UI (useful for development/testing)
doxx document.docx --force-ui

# Check version
doxx --version

# Get help
doxx --help
```

## 🎮 Terminal UI

Navigate documents with intuitive keyboard shortcuts and mouse support:

| Input | Action |
|-------|--------|
| `↑`/`k` or mouse wheel up | Scroll up |
| `↓`/`j` or mouse wheel down | Scroll down |
| `Page Up`/`Page Down` | Page navigation |
| `Home`/`End` | Jump to start/end |
| `o` | Toggle outline view |
| `s` | Open search |
| `c` | **Copy content to clipboard** |
| `F2` | Copy content (in search view) |
| `n`/`p` | Next/previous search result |
| `h`/`F1` | Toggle help |
| `q` | Quit |

## 💻 Examples

### Document analysis
```bash
# Quick document overview
doxx annual-report.docx

# Find all tables and export as CSV
doxx financial-data.docx --export csv

# Search for specific terms with context
doxx legal-contract.docx --search "liability"

# Navigate large documents with outline
doxx technical-manual.docx --outline
```

### Copy & clipboard workflow
```bash
# Interactive document review with copy
doxx quarterly-report.docx
# Navigate to important section, press 'c' to copy to clipboard
# Paste into email, Slack, or other applications

# Copy search results for sharing
doxx meeting-notes.docx --search "action items"
# Press F2 to copy all search results with context

# Copy document structure for planning
doxx technical-spec.docx --outline
# Press 'c' to copy hierarchical outline
```

### Color rendering workflow
```bash
# View presentations with original colors preserved
doxx marketing-deck.docx --color
# Red titles, blue highlights, green callouts render in terminal

# Compare colored vs plain text output
doxx design-review.docx          # Plain text (default)
doxx design-review.docx --color  # With Word document colors

# Color information always available in exports
doxx colorful-report.docx --export json | jq '.elements[] | select(.Paragraph.formatting.color != null)'
```

### Pipeline integration
```bash
# Extract text for further processing
doxx meeting-notes.docx --export text | grep "action items"

# Convert Word tables to CSV for analysis
doxx survey-results.docx --export csv | python analyze.py

# Get document metadata as JSON
doxx report.docx --export json | jq '.metadata'
```

## 🏗️ Architecture

Built with modern Rust for maximum performance:

- **Document parsing**: [`docx-rs`](https://crates.io/crates/docx-rs) for robust `.docx` file handling
- **Terminal UI**: [`ratatui`](https://crates.io/crates/ratatui) for beautiful cross-platform interfaces  
- **Text processing**: [`unicode-segmentation`](https://crates.io/crates/unicode-segmentation) for proper Unicode support
- **Search**: [`regex`](https://crates.io/crates/regex) for powerful pattern matching

## 🎯 Why doxx?

**doxx fills a critical gap**: there's no good way to view Word documents in the terminal. Current solutions force you to choose between losing all formatting or switching to GUI applications.

### The Problem with Existing Tools

| Tool | Type | DOCX Support | Formatting | Tables | Interactive |
|------|------|-------------|------------|---------|------------|
| **docx2txt** | Text extractor | ✅ Basic | ❌ Lost | ❌ Mangled | ❌ No |
| **antiword** | Legacy converter | ❌ .doc only | ❌ Lost | ❌ Basic | ❌ No |
| **pandoc** | Universal converter | ✅ Via chain | ❌ Lost | ❌ Basic | ❌ No |
| **glow** | Markdown viewer | ❌ Wrong format | ✅ Rich | ✅ Good | ✅ Yes |
| **Microsoft Word** | GUI application | ✅ Native | ✅ Rich | ✅ Perfect | ✅ Yes |

### What doxx Brings

**doxx** is the **first terminal-native DOCX viewer** that preserves formatting and provides an interactive experience:

| Feature | doxx | Best Alternative |
|---------|------|------------------|
| **Rich DOCX viewing** | ✅ Native with formatting | ❌ Plain text only |
| **Smart table rendering** | ✅ Aligned with borders | ❌ Unformatted text |
| **Interactive navigation** | ✅ Full TUI interface | ❌ Pipe to `less` |
| **Terminal integration** | ✅ SSH-friendly, scriptable | ❌ GUI required |
| **Multiple exports** | ✅ Markdown, CSV, JSON | ❌ Text only |

### vs. Microsoft Word
- ⚡ **Instant startup** (50ms vs 8+ seconds)
- 💾 **Minimal memory** (15MB vs 500MB+ RAM)
- 💰 **Zero licensing costs** ($0 vs $149+ per license)
- 🔒 **SSH-friendly** for remote server access
- 🔧 **Scriptable** for automation workflows

### vs. Text Extractors (docx2txt, antiword)
- 🎨 **Preserves formatting** (bold, italic, structure)
- 📊 **Intelligent table rendering** with proper alignment
- 🖥️ **Interactive interface** vs. static text output
- 🔍 **Built-in search** with highlighting and navigation
- 📤 **Smart exports** with format-aware output

### vs. Terminal Document Viewers (glow, bat, mdcat)
- 📄 **Native DOCX support** vs. markdown/code only
- 🏢 **Business document focused** vs. developer files
- 📊 **Advanced table intelligence** for spreadsheet-like data
- 🔄 **Multiple export formats** for downstream processing

## 🛠️ Development

### Building from source

```bash
# Clone the repository
git clone https://github.com/bgreenwell/doxx.git
cd doxx

# Build and run
cargo build --release
cargo run -- --help

# Run tests
cargo test
```

### Project structure

```
src/
├── main.rs      # CLI argument parsing and entry point
├── document.rs  # Document parsing and table structures  
├── ui.rs        # Terminal interface and rendering
├── export.rs    # Export functionality for different formats
```

## 🚧 Roadmap

### Coming soon
- 🔗 **Hyperlink support** for navigation within documents  
- 🖼️ **Image display** in supported terminals
- 🎨 **Themes and customization** for personalized viewing
- 🌐 **Web interface** for browser-based viewing

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a pull request**

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with the amazing [Rust](https://www.rust-lang.org/) programming language
- Terminal UI powered by [ratatui](https://ratatui.rs/)
- Document parsing with [docx-rs](https://crates.io/crates/docx-rs)
- Inspired by [Charm's Glow](https://github.com/charmbracelet/glow) for beautiful CLI rendering
- Influenced by the terminal-first development philosophy

---

**Made with ❤️ for developers who live in the terminal**