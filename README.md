# doxx 📄

> Beautiful `.docx` viewing in your terminal — no Microsoft Word required

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

**doxx** is a lightning-fast, terminal-native document viewer for Microsoft Word files. Built with Rust for performance and reliability, it brings Word documents to your command line with beautiful rendering, smart table support, and powerful export capabilities.

## ✨ Features

### Document viewing
- 🎨 **Beautiful terminal rendering** with syntax highlighting and formatting
- 📊 **Professional table display** with smart alignment and Unicode borders
- 🔍 **Full-text search** with highlighting and context
- 📑 **Document outline** for quick navigation
- 🎯 **Multiple view modes** — document, outline, search, and help

### Smart table support
- 📋 **Advanced table parsing** with automatic header detection
- 🎯 **Intelligent alignment** — numbers right-aligned, text left-aligned, booleans centered  
- 💱 **Data type detection** for currency, percentages, dates, and more
- 🎨 **Professional ASCII rendering** with scalable Unicode borders
- 🔄 **Search within tables** across headers and cell content

### Export & integration
- 📝 **Markdown export** with proper table alignment indicators
- 📊 **CSV extraction** for data analysis workflows
- 📄 **Plain text** output for piping to other tools
- 🗂️ **JSON export** with full document structure
- ⚡ **CLI-friendly** for scripts and automation

## 🚀 Quick start

### Installation

```bash
# Install from source (requires Rust)
git clone https://github.com/bgreenwell/doxx.git
cd doxx
cargo install --path .
```

### Basic usage

```bash
# View a document
doxx quarterly-report.docx

# Start with outline view
doxx document.docx --outline

# Search for specific content
doxx contract.docx --search "payment terms"

# Export to different formats
doxx spreadsheet.docx --export csv > data.csv
doxx report.docx --export markdown > report.md
doxx document.docx --export json > structure.json

# Get help
doxx --help
```

## 🎮 Terminal UI

Navigate documents with intuitive keyboard shortcuts:

| Key | Action |
|-----|--------|
| `↑`/`k` | Scroll up |
| `↓`/`j` | Scroll down |
| `Page Up`/`Page Down` | Page navigation |
| `Home`/`End` | Jump to start/end |
| `o` | Toggle outline view |
| `s` | Open search |
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

### vs. Microsoft Word
- ⚡ **Instant startup** (50ms vs 8+ seconds)
- 💾 **Minimal memory** (15MB vs 500MB+ RAM)
- 💰 **Zero licensing costs** ($0 vs $149+ per license)
- 🔒 **SSH-friendly** for remote server access
- 🔧 **Scriptable** for automation workflows

### vs. Other viewers
- 🖥️ **Terminal-native** design for CLI workflows
- 📊 **Advanced table support** with smart formatting
- 🔍 **Powerful search** with highlighting and context
- 📤 **Multiple export formats** for integration
- ⚡ **Built with Rust** for speed and reliability

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
└── ai.rs        # AI integration framework (coming soon)
```

## 🚧 Roadmap

### Coming soon
- 🤖 **AI integration** for document summarization and Q&A
- 🔗 **Hyperlink support** for navigation within documents  
- 📷 **Image descriptions** with AI-generated alt text
- 🎨 **Themes and customization** for personalized viewing
- 🌐 **Web interface** for browser-based viewing

### AI-powered features (planned)
- 📝 **Document summarization** with key points extraction
- ❓ **Interactive Q&A** about document content
- 📊 **Smart data extraction** for action items and deadlines
- 🔍 **Semantic search** beyond keyword matching
- 🛡️ **Privacy-first** with local AI model support

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