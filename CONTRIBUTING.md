# Contributing to doxx

Thanks for your interest in improving doxx! This document covers the practical steps for submitting a change. For deeper technical context (project structure, key dependencies, known issues), see [AGENTS.md](AGENTS.md) — it's written for AI coding agents but is equally useful for human contributors.

## Getting started

```bash
git clone https://github.com/bgreenwell/doxx.git
cd doxx
cargo build --release
cargo test
cargo run -- tests/fixtures/minimal.docx
```

**Requirements:** Rust 1.70+, and `libxcb` on Linux.

## Before you open a PR

1. **Check for an existing issue.** If you're fixing a bug or adding a feature, search [open issues](https://github.com/bgreenwell/doxx/issues) first — either to link your PR to it, or to confirm the change is wanted before you invest time.
2. **Keep changes focused.** One logical change per PR. Unrelated formatting/cleanup changes make review harder — split them out.
3. **Add tests.** New parsing logic, especially in `src/document/parsing/`, should have unit tests. Bug fixes should include a regression test where practical.
4. **Update `CHANGELOG.md`** for any user-facing change, under `## [Unreleased]`. Follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format — one line per entry, no sub-bullets, standard sections only (Added/Changed/Deprecated/Removed/Fixed/Security). Internal refactors with no user-visible effect don't need an entry; the commit message covers those.

## Before every commit

```bash
./scripts/quick-check.sh   # fmt, clippy, tests - fast, run this often
```

## Before pushing / opening a PR

```bash
./scripts/check.sh   # fmt --check, clippy -D warnings, tests, release build
```

CI runs the same checks (`cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-features`, `cargo build --release`, plus `nix build` on Unix) across Linux, macOS, and Windows. Clippy warnings fail CI — there's no "just a warning" here.

## Commit messages

Conventional commit format: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, etc. No signature blocks.

## Code style

- `rustfmt` and `clippy` are the source of truth — if they're happy, formatting/style is fine.
- Errors: `anyhow::Result<T>` with `.context()`/`.with_context()` at the app layer, consistent with the rest of the codebase.
- Prefer extending an existing parsing module (`src/document/parsing/*.rs`) over adding new top-level modules for document-format logic.

## Testing

```bash
cargo test --all-features
cargo test --test integration_test
cargo test test_name -- --nocapture
```

Fixtures live in `tests/fixtures/`; see `tests/fixtures/README.md` for what each one covers. Run `./scripts/regenerate-fixtures.sh` if you need to rebuild the generated ones.

## Reporting bugs / requesting features

Use the [issue templates](https://github.com/bgreenwell/doxx/issues/new/choose) — they ask for the details (doxx version, OS, sample document if applicable) that make bugs actually reproducible. For security vulnerabilities, see [SECURITY.md](SECURITY.md) instead of filing a public issue.

## License

By contributing, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
