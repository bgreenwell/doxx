# Widget Integration Notes

## Status: Phase 6 - Architectural Decision Required

### Completed (Phases 1-5)
- ✅ DocumentWidget structure with builder pattern
- ✅ Unicode-aware text wrapping (wrap_formatted_runs)
- ✅ Element rendering: headings, paragraphs, lists
- ✅ Table rendering with scaling and alignment
- ✅ Image placeholder rendering
- ✅ Main render loop with element dispatch

### Challenge: Widget Trait Limitations

The ratatui `Widget` trait has this signature:
```rust
fn render(self, area: Rect, buf: &mut Buffer)
```

**Problem**: `StatefulImage` rendering requires `Frame` access, not just `Buffer`:
```rust
f.render_stateful_widget(image_widget, img_rect, protocol);
```

This is a fundamental architectural mismatch.

### Attempted Solutions

#### Option A: Hybrid Rendering (Current Working Approach)
- Render text with Paragraph widget
- Overlay images using Frame after text rendering
- ✅ Works correctly for images
- ❌ Disables text wrapping when images present (positioning breaks)

#### Option B: Pure Widget Approach (This Branch)
- Custom DocumentWidget handles all rendering
- ✅ Perfect text wrapping
- ❌ Can't render images (Widget trait limitation)

#### Option C: Custom Render Function (Not Yet Tried)
Instead of Widget trait, create:
```rust
fn render_document_content(
    elements: &[DocumentElement],
    area: Rect,
    buf: &mut Buffer,
    frame: &mut Frame,  // <-- Key difference
    image_protocols: &mut [Box<dyn StatefulProtocol>],
    scroll_offset: usize,
    color_enabled: bool,
) {
    // Can render both text AND images in single pass
}
```

###Option D: Two-Pass with Position Tracking
1. DocumentWidget renders text and returns image Y positions
2. Caller overlays images at returned positions
3. Challenge: Widget trait consumes `self` - can't return values

### Recommendation

For v0.2.0, I recommend **Option C**: Custom render function that takes both Buffer and Frame.

**Benefits**:
- Single-pass rendering
- Perfect text wrapping
- Correct image positioning
- Full control over layout

**Implementation**:
1. Keep DocumentWidget struct and all helper methods
2. Don't implement Widget trait
3. Add custom `render()` method that takes Frame
4. Update ui.rs to call custom render instead of Widget::render

This gives us the best of both worlds: clean architecture + full feature support.

### Current State

The DocumentWidget implementation is **complete and working** for text rendering.
It just can't be integrated yet due to the Widget trait limitation.

**Options for moving forward**:
1. Implement Option C (custom render function) - ~2 hours
2. Keep current hybrid approach, improve positioning logic - ~1 hour
3. Defer image support to v0.3.0, use DocumentWidget for text-only - ~30 min

## File Changes Summary

### Created
- `src/widgets/mod.rs` - Module structure
- `src/widgets/document.rs` - DocumentWidget implementation (540 lines)

### Modified
- `Cargo.toml` - Added unicode-width dependency

### To Modify (for Option C)
- `src/widgets/document.rs` - Change render signature
- `src/ui.rs` - Replace render_document() with custom DocumentWidget::render()

## Testing Needed

Once integrated:
- [ ] Text wrapping works correctly at all terminal widths
- [ ] Images appear at correct positions after wrapping
- [ ] Scrolling works smoothly
- [ ] Search highlighting preserved
- [ ] Tables render correctly with wrapped text
- [ ] Lists with long items wrap properly
- [ ] All test fixtures render correctly

## Git Commits

Phase 1: `feat(widgets): add DocumentWidget skeleton structure`
Phase 2: `feat(widgets): implement text wrapping with unicode support`
Phase 3: `feat(widgets): implement basic element rendering`
Phase 4: `feat(widgets): implement image & table rendering`
Phase 5: `feat(widgets): implement main render loop`
Phase 6: **Blocked on architectural decision**
