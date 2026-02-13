# doxx: AI Agent Development Guide

## Project Context

doxx is a terminal-based document viewer for .docx files built with Rust. It provides rich text rendering, equation support (LaTeX), tables, search, navigation, and multiple export formats (Markdown, CSV, JSON, ANSI, text). The codebase emphasizes performance, cross-platform support, and terminal-native workflows.

## Quick Start

```bash
# Build and test
cargo build --release
cargo test

# Run with sample documents
cargo run -- tests/fixtures/minimal.docx
cargo run -- tests/fixtures/business-report.docx --export ansi
cargo run -- tests/fixtures/equations.docx --search "formula"
```

## Project Structure

```
src/
├── main.rs              # CLI interface
├── document/            # Document parsing and structures (modular)
│   ├── mod.rs           # Module root with re-exports
│   ├── models.rs        # Core data structures (Document, DocumentElement, etc.)
│   ├── loader.rs        # Main document loading orchestrator
│   ├── io.rs            # File I/O operations
│   ├── query.rs         # Search and outline generation
│   ├── cleanup.rs       # Post-processing utilities
│   └── parsing/         # Specialized parsing modules
│       ├── numbering.rs # Numbering management (headings, lists)
│       ├── formatting.rs# Text and formatting extraction
│       ├── heading.rs   # Heading detection (style-based, heuristic)
│       ├── list.rs      # List processing and grouping
│       ├── table.rs     # Table extraction with alignment detection
│       └── equation.rs  # OMML to LaTeX conversion
├── ui.rs                # Terminal UI (ratatui)
├── export.rs            # Export formats (markdown, JSON, CSV, ANSI)
├── image_extractor.rs   # Image extraction from DOCX
└── terminal_image.rs    # Terminal image rendering
```

## Core Architecture

Runtime & Error Handling:
- Async Runtime: Tokio for file operations and extensibility
- Error Handling: anyhow for ergonomic error management

Key Dependencies:
- docx-rs (0.4): .docx file parsing - LIMITATION: No OMML parsing (we work around with direct XML parsing)
- ratatui (0.29): Terminal UI framework
- ratatui-image (8.0): Terminal image rendering
- crossterm (0.27): Cross-platform terminal control
- unicode-segmentation (1.10): Unicode handling
- clap (4.4): CLI argument parsing

Design Principles:
- Modular design: Separate concerns (parsing, UI, export, equations, images)
- Document model: Structured representation supporting rich content
- Terminal UI: Cross-platform terminal interfaces with ratatui

## Development Constraints

Known Issues: Equation positioning (#58), text wrapping (#45), advanced numbering (#24)

See .agents/known-issues.md for detailed context, workarounds, and fix plans.

## Testing

```bash
cargo test --all-features                    # Run all tests
cargo test --test integration_test           # Specific suite
cargo test test_name -- --nocapture          # Single test with output
```

See .agents/workflows.md for detailed commands and fixture usage.

## Performance Targets

Startup: < 100ms | Memory: < 50MB | Rendering: < 500ms | Binary: ~3MB

See .agents/performance.md for benchmarks and profiling tips.

## Development Workflow - CRITICAL

IMPORTANT: Always run validation scripts before committing/pushing to ensure CI/CD passes.

Quick Iteration (during development):
```bash
./scripts/quick-check.sh  # Runs: fmt, clippy, tests
```

Full Validation (before pushing):
```bash
./scripts/check.sh  # Runs: fmt --check, clippy, tests, build --release
```

Optional: Git Pre-Push Hook
```bash
cp scripts/pre-push.hook .git/hooks/pre-push
chmod +x .git/hooks/pre-push
# Now validation runs automatically on every push
```

## Git Commit Guidelines

- DO NOT include signature blocks in commit messages
- Use conventional commit format: feat:, fix:, docs:, etc.
- Write clear, concise commit messages describing the change
- Test thoroughly before committing

## CI/CD Pipeline

Pipeline location: .github/workflows/ci.yml

Platforms tested: Linux (Ubuntu), macOS, Windows

Checks performed:
1. Format check (Unix only): cargo fmt --all -- --check
2. Clippy lints: cargo clippy --all-targets -- -D warnings (ZERO warnings required)
3. Test suite: cargo test --all-features
4. Release build: cargo build --release
5. Nix build (Unix only): nix build

Best practice: Run ./scripts/check.sh before every push to catch issues locally.

Common failures and fixes: See .agents/ci-troubleshooting.md

## Release Process

See .agents/release.md for detailed release steps, version bumping, and CI/CD automation.

## Additional Resources

See .agents/ directory for detailed documentation organized into three categories:

**development/** - Enhancement planning and architecture decisions
- project-audit.md - Current state analysis and priorities
- roadmap.md - Feature timeline and prioritization
- enhancements/ - Detailed proposals for planned features
- architecture/ - Design patterns and structural decisions

**operations/** - Daily workflows and procedures
- known-issues.md - Active bugs and constraints
- ci-troubleshooting.md - Common CI/CD failures and fixes
- workflows.md - Detailed cargo commands
- performance.md - Benchmarks and profiling
- release.md - Release process steps

Start with .agents/README.md for navigation and index of all resources.
