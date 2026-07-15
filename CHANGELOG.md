# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- README now documents the config file, `doxx config` subcommand, keymap presets, and custom key bindings ([#78](https://github.com/bgreenwell/doxx/issues/78))
- The spacebar can now be bound in `config.toml` via a literal `" "` key or the `"space"` alias
- Declared minimum supported Rust version (MSRV) of 1.88 in `Cargo.toml` ([#87](https://github.com/bgreenwell/doxx/issues/87))

### Changed
- README Rust version badge now reflects the actual 1.88+ MSRV instead of a stale 1.70+ ([#87](https://github.com/bgreenwell/doxx/issues/87))

### Fixed
- Custom key bindings for the spacebar were silently dropped due to whitespace trimming during config parsing

## [0.1.4] - 2026-05-26

### Added
- `SourceCode` paragraph style renders as a distinct code block in all export formats and the TUI ([#76](https://github.com/bgreenwell/doxx/issues/76))
- Text inside `wps:txbx` shape text boxes is now extracted and rendered with box-drawing borders instead of being silently dropped ([#76](https://github.com/bgreenwell/doxx/issues/76))

### Fixed
- Numbered list items sharing the same abstract numbering definition now count sequentially instead of both restarting at 1 ([#76](https://github.com/bgreenwell/doxx/issues/76))
- Bullet-format list levels (`numFmt="bullet"`) no longer render as lettered sequences ([#76](https://github.com/bgreenwell/doxx/issues/76))
- `w:br` line break elements inside a run are now preserved, fixing multi-line code blocks concatenated into a single line ([#76](https://github.com/bgreenwell/doxx/issues/76))

## [0.1.3] - 2026-05-24

### Added
- Configurable keymap presets (`default`, `vim`, `less`) with per-user TOML overrides in `~/.config/doxx/config.toml` ([#26](https://github.com/bgreenwell/doxx/issues/26))
- Search result highlighting: current match in yellow, other matches in gray
- `--restore-position` / `-r` flag to save and restore scroll position across sessions ([#66](https://github.com/bgreenwell/doxx/issues/66))
- Inline image display in TUI via Kitty, iTerm2, or half-block fallback ([#35](https://github.com/bgreenwell/doxx/issues/35))
- NetBSD package via `pkgin install doxx`

### Fixed
- `--terminal-width` / `-w` wraps headings and text correctly in ANSI export ([#45](https://github.com/bgreenwell/doxx/issues/45))
- List bullets and numbers no longer inherit formatting from the first run of their item
- ANSI formatting no longer bleeds into adjacent unformatted runs
- Strikethrough and underline render correctly in the TUI viewer and table cells

### Changed
- Refactored document rendering into a custom `DocumentWidget`
- Split `document.rs` into focused submodules under `document/parsing/`

## [0.1.2] - 2025-10-21

### Added
- Inline equation rendering within paragraph text using `$...$` LaTeX delimiters
- ANSI export (`--export ansi`) with `--terminal-width`/`-w` and `--color-depth` options ([#45](https://github.com/bgreenwell/doxx/issues/45))
- Strikethrough text rendering in TUI and all export formats ([#47](https://github.com/bgreenwell/doxx/issues/47))
- Search state toggle: press `S` to show/hide results ([#50](https://github.com/bgreenwell/doxx/pull/50)) by [@Jianchi-Chen](https://github.com/Jianchi-Chen)

### Fixed
- Integration tests use `CARGO_BIN_EXE` for compatibility with packaging environments ([#60](https://github.com/bgreenwell/doxx/issues/60))
- Text formatting (bold, italic, colors) no longer lost during inline equation processing
- Formatting preserved in Word automatic numbered lists
- Empty search queries no longer match the entire document ([#50](https://github.com/bgreenwell/doxx/pull/50))
- Non-`.docx` files now rejected with clear error messages ([#40](https://github.com/bgreenwell/doxx/issues/40), [#56](https://github.com/bgreenwell/doxx/issues/56))
- Display equations now appear at their correct paragraph position ([#58](https://github.com/bgreenwell/doxx/issues/58))

### Changed
- Updated `ratatui-image` from v1.0 to v8.0 for Debian packaging compatibility ([#59](https://github.com/bgreenwell/doxx/issues/59))

## [0.1.1] - 2025-08-22

### Added
- Release pipeline: cross-platform binaries, crates.io publishing, Homebrew formula, SHA256 checksums
- Release automation script (`scripts/release.sh`)

### Fixed
- Image picker initialization on Windows

### Changed
- README updated to use sentence case throughout

## [0.1.0] - Initial Release

### Added
- `.docx` document parsing and terminal viewer
- Navigation, search, and outline views
- Export formats: text, markdown, JSON, CSV
- Table parsing and rendering
- Document metadata extraction
- Equation support (OMML to LaTeX)