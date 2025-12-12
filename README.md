# doxx 📄 <a href="https://terminaltrove.com/"><img src="https://cdn.terminaltrove.com/media/badges/tool_of_the_week/svg/terminal_trove_tool_of_the_week_green_on_dark_grey_bg.svg" align="right" height="40" /></a>

> `.docx` files in your terminal — no Microsoft Word required

[![CI](https://github.com/bgreenwell/doxx/workflows/CI/badge.svg)](https://github.com/bgreenwell/doxx/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

A fast, terminal-native document viewer for Word files. View, search, and export `.docx` documents without leaving your command line.

## Screenshots

<div align="center">
  <table>
    <tr>
      <td align="center">
        <img src="assets/screenshot1-images.png" alt="Terminal image display" width="400">
        <br><em>Terminal image display</em>
      </td>
      <td align="center">
        <img src="assets/screenshot2-colors.png" alt="Color support" width="400">
        <br><em>Color support</em>
      </td>
    </tr>
    <tr>
      <td align="center">
        <img src="assets/screenshot3-tables.png" alt="Smart tables" width="400">
        <br><em>Smart tables with alignment</em>
      </td>
      <td align="center">
        <img src="assets/screenshot4-lists.png" alt="Lists and formatting" width="400">
        <br><em>Lists and formatting</em>
      </td>
    </tr>
    <tr>
      <td align="center" colspan="2">
        <img src="assets/screenshot5-equations.png" alt="Equation support" width="400">
        <br><em>Inline and display equations</em>
      </td>
    </tr>
  </table>
</div>

## 🎬 Demo

<div align="center">
  <img src="assets/demo.gif" alt="doxx mixed formatting demo" width="600">
  <br><em>Mixed formatting with colors, bold, italic, underline, strikethrough and interactive navigation</em>
</div>

## ✨ Features

- **Beautiful terminal rendering** with formatting, tables, and lists
- **Equation support** — LaTeX rendering for inline and display equations 📐
- **Fast search** with highlighting 🔍
- **Smart tables** with proper alignment and Unicode borders
- **Copy to clipboard** — grab content directly from the terminal
- **Export formats** — Markdown, CSV, JSON, plain text, ANSI-colored output
- **Terminal images** for Kitty, iTerm2, WezTerm 🖼️
- **Color support** — see Word document colors in your terminal

## 🚀 Installation

### Package managers

#### Homebrew (macOS/Linux)

```bash
brew install doxx
```

#### Cargo (cross-platform)
```bash
cargo install doxx
```

#### Debian/Ubuntu

`doxx` is officially packaged for Debian as
[`rust-doxx`](https://tracker.debian.org/pkg/rust-doxx) and is available in testing
(forky).

```bash
sudo apt install doxx
```

#### Arch Linux

```bash
pacman -S doxx
```

The AUR package is also available for the development version:

```bash
yay -S doxx-git
```
*Thanks to [@mhegreberg](https://github.com/mhegreberg) for creating and maintaining the AUR package!*

#### Nix (cross-platform)
```bash
nix profile install github:bgreenwell/doxx
```
*Thanks to [@bobberb](https://github.com/bobberb) for creating the Nix flake!*

#### Conda-Forge (cross-platform)
```bash
conda install doxx
```
or globally using [Pixi](pixi.sh):
```bash
pixi global install doxx
```

#### Scoop (Windows)
```bash
# Coming soon
scoop bucket add doxx https://github.com/bgreenwell/doxx-scoop
scoop install doxx
```

### Pre-built binaries

Download from [GitHub releases](https://github.com/bgreenwell/doxx/releases):

```bash
# macOS/Linux - automatic platform detection
curl -L https://github.com/bgreenwell/doxx/releases/latest/download/doxx-$(uname -s)-$(uname -m).tar.gz | tar xz
sudo mv doxx /usr/local/bin/

# Verify installation
doxx --version
```

**Available platforms:**
- **Linux**: `x86_64-unknown-linux-musl` (statically linked)
- **macOS**: `x86_64-apple-darwin` (Intel) and `aarch64-apple-darwin` (Apple Silicon)
- **Windows**: `x86_64-pc-windows-msvc`

### Build from source

```bash
git clone https://github.com/bgreenwell/doxx.git
cd doxx
cargo install --path .

# Or for development
cargo build --release
```

**Requirements:**
- Rust 1.70+ 
- System dependencies: `libxcb` (Linux only)

## 🎯 Usage

```bash
# View a document
doxx report.docx

# Search for content
doxx contract.docx --search "payment"

# Start with outline view
doxx document.docx --outline

# Export to different formats
doxx data.docx --export csv > data.csv
doxx report.docx --export markdown > report.md

# View with images (supported terminals)
doxx presentation.docx --images --export text

# Enable color rendering
doxx slides.docx --color
```

## 📋 Command Line Options

### Basic options
```bash
doxx [OPTIONS] <FILE>
```

| Option | Description |
|--------|-------------|
| `<FILE>` | Input document file (.docx) |
| `-h, --help` | Show help information |
| `-V, --version` | Show version information |

### Viewing options
| Option | Description |
|--------|-------------|
| `-o, --outline` | Start with outline view for quick navigation |
| `-p, --page <PAGE>` | Jump to specific page number on startup |
| `-s, --search <TERM>` | Search and highlight term immediately |
| `--force-ui` | Force interactive UI mode (bypass TTY detection) |
| `--color` | Enable color support for text rendering |

### Export options
| Option | Values | Description |
|--------|--------|-------------|
| `--export <FORMAT>` | `markdown`, `text`, `csv`, `json`, `ansi` | Export document instead of viewing |

**Export examples:**
```bash
doxx report.docx --export markdown  # Convert to Markdown
doxx data.docx --export csv         # Extract tables as CSV (tables only!)
doxx document.docx --export text    # Plain text output
doxx structure.docx --export json   # Document metadata as JSON
doxx document.docx --export ansi    # ANSI-colored terminal output
```

**📊 CSV export note:**
The CSV export extracts **only tables** from the document, ignoring all text content. Perfect for pulling structured data from business reports, research papers, or surveys for analysis in Excel, Python, or databases.

### ANSI export options
| Option | Values | Description |
|--------|--------|-------------|
| `-w, --terminal-width <COLS>` | Number | Set terminal width for formatting (default: $COLUMNS or 80) |
| `--color-depth <DEPTH>` | `auto`, `1`, `4`, `8`, `24` | Control color rendering depth |

**ANSI export examples:**
```bash
doxx document.docx --export ansi                     # Full color ANSI output
doxx document.docx --export ansi --color-depth 1     # Monochrome (no colors)
doxx document.docx --export ansi --color-depth 4     # 16 colors
doxx document.docx --export ansi --terminal-width 80 # Set terminal width
doxx report.docx --export ansi | less -R             # Pipe to less with color support
```

**🌈 Color depth options:**
- `auto` - Auto-detect terminal capabilities
- `1` - Monochrome (no colors, formatting only)
- `4` - 16 colors (standard ANSI colors)
- `8` - 256 colors (extended ANSI palette)
- `24` - True color (16.7 million colors)

### Image options
| Option | Description |
|--------|-------------|
| `--images` | Display images inline in terminal (auto-detect capabilities) |
| `--extract-images <DIR>` | Extract images to specified directory |
| `--image-width <COLS>` | Maximum image width in terminal columns (default: auto-detect) |
| `--image-height <ROWS>` | Maximum image height in terminal rows (default: auto-detect) |
| `--image-scale <SCALE>` | Image scaling factor (0.1 to 2.0, default: 1.0) |

**Image examples:**
```bash
doxx presentation.docx --images                    # Show images inline
doxx document.docx --images --image-width 80       # Limit image width
doxx slides.docx --extract-images ./images/        # Save images to folder
```

**⚠️ Image display notes:**
- `--images` currently works with `--export text` mode and shows placeholders in TUI
- Supports iTerm2, Kitty, and WezTerm terminals


## ⌨️ Navigation

| Key | Action |
|-----|--------|
| `↑`/`k` | Scroll up |
| `↓`/`j` | Scroll down |
| `o` | Toggle outline |
| `s` | Search |
| `c` | Copy to clipboard |
| `h` | Help |
| `q` | Quit |

## 🔧 Why doxx?

Current terminal tools for Word documents:
- **docx2txt** → Loses all formatting, mangled tables
- **pandoc** → Complex chain, formatting lost
- **antiword** → Only handles old `.doc` files

**doxx** gives you:
- ✅ Rich formatting preserved (bold, italic, headers)
- ✅ Professional table rendering with alignment
- ✅ Equation support (inline and display LaTeX)
- ✅ Interactive navigation and search
- ✅ Multiple export formats for workflows
- ✅ Terminal image display for modern terminals
- ✅ Fast startup (50ms vs Word's 8+ seconds)

Perfect for developers, sysadmins, and anyone who prefers the terminal.

## 📊 Examples

### Quick document analysis
```bash
# Get overview and search
doxx quarterly-report.docx
doxx --search "revenue"

# Extract tables for analysis
doxx financial-data.docx --export csv | python analyze.py
```

### Copy workflows
```bash
# Review and copy sections
doxx meeting-notes.docx
# Press 'c' to copy current view to clipboard

# Copy search results
doxx specs.docx --search "requirements"
# Press F2 to copy results with context
```

### Pipeline integration
```bash
# Extract text for processing
doxx notes.docx --export text | grep "action items"

# Get document structure
doxx report.docx --export json | jq '.metadata'
```

## 🏗️ Architecture

Built with Rust for performance:
- **[docx-rs](https://crates.io/crates/docx-rs)** — Document parsing
- **[ratatui](https://crates.io/crates/ratatui)** — Terminal UI
- **[viuer](https://crates.io/crates/viuer)** — Image rendering
- **[unicode-segmentation](https://crates.io/crates/unicode-segmentation)** — Proper Unicode handling

## 🛠️ Development

```bash
# Build and test
cargo build --release
cargo test

# Run with sample document
cargo run -- tests/fixtures/minimal.docx
```

## Known limitations

**Equation positioning:** Display equations may not appear at exact positions due to limitations in the underlying docx-rs parsing library. We've filed an [upstream issue](https://github.com/bokuweb/docx-rs/issues) and are planning a complete fix for v0.2.0 using direct XML parsing.

## Roadmap

- Perfect equation positioning (v0.2.0)
- Image support in TUI via ratatui-image crate
- Enhanced table support (merged cells, complex layouts)
- Performance improvements for large documents
- Hyperlink navigation
- Custom themes

## 💡 Inspiration

This project was inspired by [Charm](https://github.com/charmbracelet)'s [Glow](https://github.com/charmbracelet/glow) package — the beautiful terminal Markdown renderer that shows how terminal document viewing can be both powerful and elegant. Just as Glow brings rich Markdown rendering to your command line, doxx aims to do the same for Microsoft Word documents.

Thanks to the Charm team for the inspiration! ✨

## 📝 License

MIT License — see [LICENSE](LICENSE) file for details.

---

**Made for developers who live in the terminal** 🚀
