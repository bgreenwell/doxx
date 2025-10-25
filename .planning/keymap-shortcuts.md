# Implementation Plan for Configurable Keyboard Shortcuts (GitHub Issue #26)

## Issue Summary
User request for configurable keyboard shortcuts similar to CLI tools like `less` and `vim`:
- **Navigation**: `u`/`d` or `Ctrl+u`/`Ctrl+d` for page up/down
- **Search**: `/` to initiate search
- **Search navigation**: `n`/`N` for next/previous search results
- **Positioning**: `H`/`L` for Home/End navigation

Community suggestion to expand into comprehensive keymap module supporting different editor styles.

## Current State Analysis

**Existing keyboard handling** (`src/ui.rs:415-510`):
- Hardcoded key matching using `match key.code` statements
- Different keybindings for each `ViewMode` (Document, Outline, Search, Help)
- Basic vim-like navigation exists: `k`/`j` for up/down, `h` for help
- Current search navigation uses `n`/`p` (not `n`/`N` as requested)
- Missing requested shortcuts: `u`/`d` for page navigation, `/` for search, `H`/`L` for home/end

**Architecture compatibility**:
- Well-structured UI module with clear separation of concerns
- `App` struct has all necessary state for navigation and search
- CLI already has comprehensive argument structure ready for extension
- Existing codebase follows Rust best practices

## Implementation Plan

### Phase 1: Core Keymap Infrastructure (~1 week)

**1.1 Create keymap module** (`src/keymap.rs`)
```rust
pub enum Action {
    ScrollUp, ScrollDown, PageUp, PageDown,
    GoHome, GoEnd, Search, SearchNext, SearchPrev,
    ToggleOutline, ToggleHelp, Copy, Quit
}

pub struct KeyMap {
    bindings: HashMap<KeyCode, Action>,
    name: String,
}

impl KeyMap {
    pub fn lookup(&self, key: KeyCode) -> Option<Action>
    pub fn validate(&self) -> Result<(), KeymapError>
}
```

**1.2 Refactor UI key handling** (`src/ui.rs`)
- Replace hardcoded key matching with action-based dispatch
- Implement `handle_action()` method for uniform action processing
- Maintain backward compatibility with existing shortcuts
- Update each ViewMode to use keymap lookup

### Phase 2: CLI Integration (~3 days)

**2.1 Add CLI keymap option**
- Add `--keymap <preset>` flag to `Cli` struct in `main.rs`
- Support presets: "default", "vim", "less"
- Update help text and documentation
- Pass keymap selection to `App::new()`

**2.2 Help system updates**
- Dynamically generate help overlay from active keymap
- Show current keybindings instead of hardcoded text
- Add keymap name to status line

### Phase 3: Popular Presets (~1 week)

**3.1 Vim/Less keymap preset** (addresses user request)
```rust
pub fn vim_keymap() -> KeyMap {
    // u/Ctrl+u for page up, d/Ctrl+d for page down
    // H for home, L for end
    // / for search initiation
    // n/N for search next/previous (capital N for reverse)
}
```

**3.2 Future extensible presets**
- Foundation for VSCode-style, Emacs-style keymaps
- Preset validation and error handling
- Documentation for creating custom presets

### Phase 4: Testing & Validation (~3 days)

**4.1 Comprehensive test suite**
- Unit tests for keymap module (validation, conflicts, lookups)
- Integration tests for each preset
- Regression tests ensuring existing functionality works
- Manual testing with real document navigation

**4.2 Documentation updates**
- README examples showing new keymap options
- Help text updates
- CHANGELOG entry

## Technical Details

**Files to create/modify:**
- `src/keymap.rs` (new) - Core keymap infrastructure
- `src/ui.rs` - Refactor key handling to use keymaps
- `src/main.rs` - Add CLI option
- `src/lib.rs` - Export keymap types
- `tests/keymap_test.rs` (new) - Comprehensive test suite

**CLI Usage Examples:**
```bash
# Use vim-style keybindings
doxx document.docx --keymap vim

# Use less-style keybindings
doxx document.docx --keymap less

# Default behavior (current keybindings)
doxx document.docx --keymap default
```

**Expected Vim/Less Keymap:**
- `u` / `Ctrl+u` - Page up (alternative to PageUp)
- `d` / `Ctrl+d` - Page down (alternative to PageDown)
- `H` - Go to document start (alternative to Home)
- `L` - Go to document end (alternative to End)
- `/` - Enter search mode
- `n` - Next search result
- `N` - Previous search result (capital N for reverse)
- All existing shortcuts remain available

**Backward Compatibility:**
- All existing shortcuts continue working through default keymap
- No breaking changes to current user experience
- Purely additive feature

**Architecture Benefits:**
- Extensible design supports future keymap additions
- Clean separation between key input and action handling
- Easy to add new shortcuts without modifying core UI logic
- Testable components with clear interfaces

**Risk Assessment:**
- **Risk level:** Low - purely additive feature
- **Complexity:** Medium - requires architectural refactor but builds on solid foundation
- **Testing coverage:** High - comprehensive unit and integration tests planned
- **Performance impact:** Minimal - HashMap lookup vs direct matching

**Success Criteria:**
- All existing keyboard shortcuts continue to work (backward compatibility)
- Users can select keymap presets with `--keymap vim`
- Vim/less users get familiar navigation shortcuts
- Help system shows current active keybindings
- Architecture supports easy addition of new presets
- Zero performance impact on key handling

## Timeline Estimate
- **Total development time:** ~2 weeks
- **Phase 1:** 1 week (core infrastructure)
- **Phase 2:** 3 days (CLI integration)
- **Phase 3:** 1 week (presets implementation)
- **Phase 4:** 3 days (testing and documentation)

## Future Expansion Opportunities
- Runtime keymap switching (press `K` to cycle through keymaps)
- Custom user-defined shortcuts via config files
- Context-aware keymaps (different shortcuts in search mode vs document mode)
- Mouse gesture support for touchpad navigation

## Implementation Status
- **Current status:** Planning phase complete
- **Next step:** Begin Phase 1 implementation
- **Dependencies:** None - can proceed immediately
- **Blockers:** None identified