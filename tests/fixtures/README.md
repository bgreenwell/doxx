# Test Fixtures

Five fixture files covering all doxx features. Three are generated (reproducible), two are kept as-is.

## Fixture inventory

| File | Source | Purpose |
|---|---|---|
| `comprehensive.docx` | `comprehensive.md` via pandoc | All structural/text features: headings H1–H6, bold/italic/strikethrough, tables, lists, unicode, financial content |
| `comprehensive.md` | Hand-authored | Canonical markdown source for `comprehensive.docx`; also the expected content baseline for the round-trip test |
| `colors.docx` | `generate_test_docs.rs` | Color run formatting (red, green, blue, orange, purple); used by ANSI color-depth tests |
| `equations.docx` | Pre-existing | OMML equation parsing (inline and display equations) |
| `images.docx` | Pre-existing | Embedded images; used by image extraction tests |
| `minimal.docx` | `generate_test_docs.rs` | Minimal document (title + 2 paragraphs); fast smoke-test baseline |

## Regenerating fixtures

```bash
# Run both regeneration steps at once
./scripts/regenerate-fixtures.sh

# Or run individually:
pandoc tests/fixtures/comprehensive.md -o tests/fixtures/comprehensive.docx
cargo run --bin generate_test_docs
```

## Adding a new fixture

1. **Text/structure focused** — add content to `comprehensive.md` and re-run pandoc.
2. **Color/equation specific** — add a `generate_*_doc()` function to `src/bin/generate_test_docs.rs`.
3. **Pre-built binary content** (e.g. real images) — commit the `.docx` directly and document it here.

Keep files under 2MB. Update this README and add tests.
