# Enhancement: Remember Last Position

## Issue Reference
GitHub Issue: #66 (Nov 3, 2025)

## Problem Statement

When users quit doxx and reopen the same document, they must navigate back to their previous reading position. This is tedious for long documents or when frequently switching between files.

## Proposed Solution

Persist document reading position to a config file and restore on document open.

**Storage:** `~/.config/doxx/positions.toml`
```toml
["/path/to/document.docx"]
scroll_offset = 42
last_accessed = "2026-02-09T21:30:00Z"

["/another/doc.docx"]
scroll_offset = 156
last_accessed = "2026-02-08T14:22:00Z"
```

## Implementation Plan

### 1. Add Position Tracking to App State (src/ui.rs)

```rust
struct App {
    // ... existing fields
    document_path: PathBuf,  // Store for position saving
}

impl App {
    fn save_position(&self) -> Result<()> {
        // Write current scroll_offset to config
    }

    fn load_position(&mut self, path: &Path) -> Result<()> {
        // Read saved position, update scroll_offset
    }
}
```

### 2. Create Config Module (src/config.rs)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct PositionConfig {
    positions: HashMap<PathBuf, DocumentPosition>,
}

#[derive(Serialize, Deserialize)]
struct DocumentPosition {
    scroll_offset: usize,
    last_accessed: chrono::DateTime<Utc>,
}
```

### 3. Integration Points

- **On document open:** Load saved position, update scroll_offset
- **On quit (q key):** Save current position
- **Periodic save:** Auto-save every 30 seconds or on navigation
- **Config file location:** Use `dirs::config_dir()` + "doxx/positions.toml"

### 4. Cleanup Strategy

- Limit to last 100 documents (LRU eviction)
- Remove entries for deleted files on access
- Add `--clear-positions` CLI flag to reset

## Technical Considerations

**Dependencies:**
- `dirs` (already in Cargo.toml) - Config directory location
- `toml` (already in Cargo.toml) - Serialization
- `chrono` - Timestamps (add to Cargo.toml)

**Files Affected:**
- `src/ui.rs:App` structure and quit handler
- `src/main.rs:main()` - Load position on startup
- New file: `src/config.rs` - Config management

**Breaking Changes:** None (additive feature)

## Testing Strategy

- Integration test: Open document, scroll, quit, reopen → verify position restored
- Unit test: Position serialization/deserialization
- Edge case: Missing config file, corrupted TOML, deleted document

## Success Criteria

- [ ] Position saved on quit
- [ ] Position restored on document open
- [ ] Config file created in `~/.config/doxx/`
- [ ] Handles missing/corrupted config gracefully
- [ ] Documentation in README

## Priority

**Medium** - Nice UX enhancement, straightforward implementation

## Estimated Effort

**6-8 hours** (1-2 days) - Simple feature with clear scope

## References

- Issue: https://github.com/bgreenwell/doxx/issues/66
- Similar feature in `less`, `vim` (viminfo), PDF viewers
