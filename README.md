# doxx <a href="https://terminaltrove.com/"><img src="https://cdn.terminaltrove.com/media/badges/tool_of_the_week/svg/terminal_trove_tool_of_the_week_green_on_dark_grey_bg.svg" align="right" height="40" /></a>

[![CI](https://img.shields.io/github/actions/workflow/status/bgreenwell/doxx/ci.yml?style=for-the-badge)](https://github.com/bgreenwell/doxx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/doxx.svg?style=for-the-badge&color=%232B579A)](https://crates.io/crates/doxx)
[![Downloads](https://img.shields.io/crates/d/doxx?style=for-the-badge&color=%232B579A)](https://crates.io/crates/doxx)

[![License: MIT](https://img.shields.io/badge/License-MIT-%232196F3.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-%23D34516.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Easy Install](https://img.shields.io/badge/Easy%20Install-Homebrew%20%7C%20Scoop%20%7C%20WinGet-%23FBB040?style=for-the-badge)](#installation)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=for-the-badge)](https://github.com/bgreenwell/doxx/releases/latest)

> `.docx` files in your terminal — no Microsoft Word required

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

## Demo

<div align="center">
  <img src="assets/demo.gif" alt="doxx mixed formatting demo" width="600">
  <br><em>Mixed formatting with colors, bold, italic, underline, strikethrough and interactive navigation</em>
</div>

## Features

- **Beautiful terminal rendering** with formatting, tables, and lists
- **Equation support** — LaTeX rendering for inline and display equations
- **Fast search** with highlighting
- **Smart tables** with proper alignment and Unicode borders
- **Copy to clipboard** — grab content directly from the terminal
- **Export formats** — Markdown, CSV, JSON, plain text, ANSI-colored output
- **Terminal images** for Kitty, iTerm2, WezTerm
- **Color support** — see Word document colors in your terminal

## Installation

### Package managers

#### Homebrew (macOS/Linux)

```bash
brew install bgreenwell/tap/doxx
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

#### NetBSD
```bash
pkgin install doxx
```

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
# Linux (x86_64, statically linked)
curl -L https://github.com/bgreenwell/doxx/releases/latest/download/doxx-x86_64-unknown-linux-musl.tar.xz | tar xJ
sudo mv doxx /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/bgreenwell/doxx/releases/latest/download/doxx-x86_64-apple-darwin.tar.xz | tar xJ
sudo mv doxx /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/bgreenwell/doxx/releases/latest/download/doxx-aarch64-apple-darwin.tar.xz | tar xJ
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

## Usage

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

## Command Line Options

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
| `-r, --restore-position` | Restore the last saved scroll position for this document |
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

**CSV export note:**
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

**Color depth options:**
- `auto` - Auto-detect terminal capabilities
- `1` - Monochrome (no colors, formatting only)
- `4` - 16 colors (standard ANSI colors)
- `8` - 256 colors (extended ANSI palette)
- `24` - True color (16.7 million colors)

### Image options
| Option | Description |
|--------|-------------|
| `--images` | Display images inline in terminal (auto-detect capabilities) |
| `--no-images` | Force text-only mode, even if the terminal supports inline images |
| `--extract-images <DIR>` | Extract images to specified directory |
| `--image-width <COLS>` | Maximum image width in terminal columns (default: auto-detect) |
| `--image-height <ROWS>` | Maximum image height in terminal rows (default: auto-detect) |
| `--image-scale <SCALE>` | Image scaling factor (0.1 to 2.0, default: 1.0) |
| `--debug-terminal` | Print detected terminal image capabilities and exit |

**Image examples:**
```bash
doxx presentation.docx --images                    # Show images inline
doxx document.docx --images --image-width 80       # Limit image width
doxx slides.docx --extract-images ./images/        # Save images to folder
doxx --debug-terminal                               # Check what your terminal supports
```

**Image display notes:**
- `--images` renders inline in both the TUI and `--export text` output
- Supports iTerm2, Kitty, WezTerm, Sixel-capable terminals, and a half-block fallback elsewhere

## Configuration

doxx reads an optional config file to set a default keymap preset and override individual key bindings. Nothing here is required — doxx works out of the box with the `default` preset — but it's how you switch to vim/less-style navigation or remap keys to your own layout.

### Config file location

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/doxx/config.toml` |
| Linux | `~/.config/doxx/config.toml` (or `$XDG_CONFIG_HOME/doxx/config.toml`) |
| Windows | `%APPDATA%\doxx\config.toml` |

Create it with:
```bash
doxx config init
```
This writes a default config (`preset = "default"`, no overrides) and prints the exact path it used.

### Managing the preset

```bash
doxx config set keymap.preset vim    # switch to vim-style bindings
doxx config get keymap.preset        # see the active preset
```

Three presets are built in:

| Preset | Style |
|--------|-------|
| `default` | Arrow keys + `j`/`k`, single-letter shortcuts (this is what the [Navigation](#navigation) table below documents) |
| `vim` | Adds `u`/`d`/`ctrl-u`/`ctrl-d` half-page scroll, `g`/`G`/`H`/`L` for start/end, `/` to search, `N` for previous match |
| `less` | Adds the same half-page scroll and `g`/`G`, plus `b` for page up and `Space` for page down, `/` to search, `N` for previous match |

`doxx config set`/`get` currently only manage `keymap.preset` — custom key bindings (below) are set by hand-editing the TOML file.

### Custom key bindings

Add a `[keymap.custom]` table to override or add individual bindings on top of whichever preset is active:

```toml
[keymap]
preset = "vim"

[keymap.custom]
n = "scroll_up"
m = "scroll_down"
```

Each entry maps a **key string** to an **action name**. A custom binding always wins over whatever the active preset assigned to that key.

**Key strings:**
- A single character: `"n"`, `"/"`, `"q"`
- A modifier combo: `"ctrl-d"`, `"shift-h"`
- A named key: `"enter"`, `"esc"`, `"backspace"`, `"tab"`, `"up"`, `"down"`, `"left"`, `"right"`, `"pageup"`/`"pgup"`, `"pagedown"`/`"pgdn"`, `"home"`, `"end"`, `"f1"`, `"f2"`, `"space"`

**Action names:**

| Action | Effect |
|--------|--------|
| `scroll_up` / `scroll_down` | Line-by-line scroll |
| `page_up` / `page_down` | Full-page scroll |
| `half_page_up` / `half_page_down` | Half-page scroll |
| `goto_start` / `goto_end` | Jump to top/bottom of document |
| `toggle_outline` | Switch to/from outline view |
| `search` | Enter search mode |
| `search_next` / `search_previous` | Jump between search matches |
| `toggle_search_state` | Cycle search highlighting on/off |
| `toggle_help` | Show/hide the help screen |
| `copy` | Copy current view to clipboard |
| `outline_select` | Select the highlighted outline entry |
| `escape` | Cancel the current mode / close overlays |
| `search_delete_char` / `search_submit` | Edit/submit the search query (search mode only) |

If a key or action string can't be parsed, doxx prints a warning to stderr and skips that entry — the rest of your config still loads. Since the TUI takes over the screen, that warning won't be visible while doxx is running; if a binding doesn't seem to take effect, run `doxx config get keymap.preset` to sanity-check the file is being read, or temporarily redirect stderr (e.g. `doxx file.docx 2>/tmp/doxx-config-errors.log`) to check for typos.

## Navigation

The tables below show the **default** preset. Run `doxx config set keymap.preset vim` (or `less`) to switch — see [Configuration](#configuration) above for what each preset changes, or press `h` / `F1` in doxx to see the active bindings for your current preset.

| Key | Action |
|-----|--------|
| `↑`/`k` | Scroll up |
| `↓`/`j` | Scroll down |
| `PageUp` / `PageDown` | Page up / down |
| `Home` / `End` | Go to start / end of document |
| `o` | Toggle outline |
| `s` | Search |
| `S` | Toggle search highlighting |
| `n` / `p` | Next / previous search match |
| `c` | Copy to clipboard |
| `F2` | Copy search results with surrounding context |
| `h` / `F1` | Toggle help screen |
| `Esc` | Cancel current mode / close overlays |
| `Enter` | Select highlighted item (outline view) |
| `q` | Quit |

**What `vim` and `less` presets change:**

| Key | `vim` | `less` |
|-----|-------|--------|
| `u` / `ctrl-u` | Half-page up | Half-page up |
| `d` / `ctrl-d` | Half-page down | Half-page down |
| `g` / `G` | Go to start / end | Go to start / end |
| `H` / `L` | Go to start / end | *(unbound)* |
| `b` | *(unbound)* | Page up |
| `Space` | *(unbound)* | Page down |
| `/` | Search (replaces `s`) | Search (replaces `s`) |
| `N` | Previous search match (replaces `p`) | Previous search match (replaces `p`) |

## Why doxx?

Current terminal tools for Word documents:
- **docx2txt** → Loses all formatting, mangled tables
- **pandoc** → Complex chain, formatting lost
- **antiword** → Only handles old `.doc` files

**doxx** gives you:
- Rich formatting preserved (bold, italic, headers)
- Professional table rendering with alignment
- Equation support (inline and display LaTeX)
- Interactive navigation and search
- Multiple export formats for workflows
- Terminal image display for modern terminals
- Fast startup (50ms vs Word's 8+ seconds)

Perfect for developers, sysadmins, and anyone who prefers the terminal.

## Examples

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

## Architecture

Built with Rust for performance:
- **[docx-rs](https://crates.io/crates/docx-rs)** — Document parsing
- **[ratatui](https://crates.io/crates/ratatui)** — Terminal UI
- **[viuer](https://crates.io/crates/viuer)** — Image rendering
- **[unicode-segmentation](https://crates.io/crates/unicode-segmentation)** — Proper Unicode handling

## Development

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

## Inspiration

This project was inspired by [Charm](https://github.com/charmbracelet)'s [Glow](https://github.com/charmbracelet/glow) package — the beautiful terminal Markdown renderer that shows how terminal document viewing can be both powerful and elegant. Just as Glow brings rich Markdown rendering to your command line, doxx aims to do the same for Microsoft Word documents.

Thanks to the Charm team for the inspiration!

## License

MIT License — see [LICENSE](LICENSE) file for details.

---

**Made for developers who live in the terminal**
