# doxx: AI Agent Development Guide

doxx is a terminal-based .docx viewer built with Rust. It provides rich text rendering, equation support (LaTeX), tables, search, navigation, and multiple export formats (Markdown, CSV, JSON, ANSI, text).

## Quick Start

```bash
cargo build --release
cargo test
cargo run -- tests/fixtures/minimal.docx
cargo run -- tests/fixtures/comprehensive.docx --export ansi
```

## Project Structure

```
src/
├── main.rs              # CLI entry point (clap)
├── lib.rs               # Library root
├── config.rs            # Config file loading (~/.config/doxx/config.toml)
├── state.rs             # App state
├── ui.rs                # TUI (ratatui)
├── export.rs            # Export formats: markdown, JSON, CSV, text
├── ansi.rs              # ANSI color export
├── equation.rs          # OMML to LaTeX conversion
├── image_extractor.rs   # Image extraction from DOCX
├── terminal_image.rs    # Terminal image rendering
├── keymap/              # Keymap presets (default, vim, less) and bindings
├── widgets/             # Custom ratatui widgets
└── document/            # Document parsing
    ├── models.rs        # Core types (Document, DocumentElement)
    ├── loader.rs        # Parsing orchestrator
    ├── query.rs         # Search and outline
    └── parsing/         # Specialized parsers (headings, lists, tables, equations, formatting, numbering)
```

## Key Dependencies

- docx-rs (0.4): .docx parsing — no built-in OMML support; we parse equation XML directly
- ratatui (0.29) + crossterm (0.27): TUI
- ratatui-image (8.0): Kitty/iTerm2/half-block image rendering
- tokio (1.0): Async runtime
- clap (4.4): CLI

## Known Issues

- Equation positioning (#58): Display equations may not be pixel-perfect in all documents
- Advanced numbering (#24): Complex Word numbering schemes not fully supported

## Testing

```bash
cargo test --all-features
cargo test --test integration_test
cargo test test_name -- --nocapture
```

Fixtures live in `tests/fixtures/`. Run `./scripts/regenerate-fixtures.sh` to rebuild them.

## Performance Targets

Startup < 100ms | Memory < 50MB | Rendering < 500ms | Binary ~3MB

## Development Workflow

Always run before pushing:

```bash
./scripts/quick-check.sh  # fmt, clippy, tests
./scripts/check.sh        # full: fmt --check, clippy, tests, release build
```

## Git Commit Guidelines

- Use conventional commit format: feat:, fix:, docs:, refactor:, etc.
- No signature blocks in commit messages

## Changelog Guidelines

CHANGELOG.md follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format:
- One line per entry, no sub-bullets
- Standard sections only: Added, Changed, Deprecated, Removed, Fixed, Security
- User-facing changes only — internal refactors belong in commit messages

## CI/CD

`.github/workflows/ci.yml` — Linux, macOS, Windows:
1. `cargo fmt --all -- --check` (Unix)
2. `cargo clippy --all-targets -- -D warnings` (zero warnings)
3. `cargo test --all-features`
4. `cargo build --release`
5. `nix build` (Unix)

## Resources

See `.agents/` for detailed docs: `known-issues.md`, `workflows.md`, `performance.md`, `release.md`, `ci-troubleshooting.md`, `development/`.
