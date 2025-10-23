# Integration Guide: Adding Image Support to doxx TUI

## Current State Analysis

### What Already Exists in doxx
1. **Image loading infrastructure** (`ui.rs:99-130`)
   - `init_image_support()` already loads images
   - Creates `StatefulProtocol` for each image
   - Stores protocols in `app.image_protocols: Vec<StatefulProtocol>`

2. **Image placeholders in TUI** (`ui.rs:771-799`)
   - Shows "🖼️ [description]" with status message
   - Detects if protocols exist but doesn't render them

3. **Working export mode** (`terminal_image.rs`)
   - Images display correctly in non-interactive mode
   - Uses `viuer` for Kitty/iTerm2 protocols

## Integration Steps

### Step 1: Track Image Index (5 min)
**Problem**: Multiple images in document, need to map DocumentElement to protocol index.

**Solution**: Add image counter during render loop.

```rust
// In render_document(), before the loop:
let mut image_index = 0;

// In the DocumentElement::Image match arm:
DocumentElement::Image { description, width, height, image_path, .. } => {
    if let Some(protocol) = app.image_protocols.get_mut(image_index) {
        // Render the image
        let image_widget = StatefulImage::new();

        // Calculate available space (constrain to reasonable size)
        let img_area = Rect {
            x: inner.x,
            y: /* calculate based on current text position */,
            width: inner.width.min(80),
            height: 20, // or calculate from image aspect ratio
        };

        f.render_stateful_widget(image_widget, img_area, protocol);

        // Add description below image
        text.lines.push(Line::from(vec![
            Span::styled("🖼️ ", Style::default().fg(Color::Magenta)),
            Span::styled(description, Style::default().fg(Color::Gray)),
        ]));
    } else {
        // Fallback to placeholder if protocol not available
        text.lines.push(Line::from(vec![
            Span::styled("🖼️ ", Style::default().fg(Color::Magenta)),
            Span::styled(description, Style::default().fg(Color::Gray)),
        ]));
    }

    image_index += 1;
    text.lines.push(Line::from(""));
}
```

### Step 2: Handle Layout Challenges (10 min)
**Problem**: `render_document()` uses `Paragraph` widget which builds `Text` first, then renders.
Images need direct frame rendering with specific `Rect`.

**Option A: Mixed Rendering (Simpler)**
1. Render paragraph normally up to image position
2. Calculate current Y position
3. Render image directly at that position
4. Continue with remaining text

**Option B: Custom Widget (Better but more work)**
1. Create custom widget that handles both text and images
2. Properly tracks layout and scrolling
3. More control but significant refactor

**Recommendation**: Start with Option A for MVP.

### Step 3: Scrolling Considerations (5 min)
**Problem**: Images take vertical space, affects scroll calculations.

**Solution**:
- Track cumulative line count including image heights
- Adjust `scroll_offset` to account for image lines
- May need to refactor from element-based scrolling to line-based

### Step 4: Error Handling (5 min)
**Additions needed**:
```rust
// Check if image support initialized
if app.image_picker.is_none() {
    // Show placeholder only
    return;
}

// Handle missing protocols gracefully
if image_index >= app.image_protocols.len() {
    eprintln!("Warning: Image index out of bounds");
    // Show placeholder
}
```

## Testing Plan

### Test Cases
1. **Single image document** - `tests/fixtures/images.docx`
2. **Multiple images** - Create test doc with 3+ images
3. **No images** - Ensure no regression
4. **Large images** - Test sizing constraints
5. **Terminal resize** - Verify layout adapts

### Terminals to Test
- ✅ WezTerm (Kitty protocol)
- iTerm2 (iTerm protocol)
- Kitty (Kitty protocol)
- Standard terminal (halfblock fallback)

## Known Limitations

1. **Paragraph widget limitation**: Can't easily interleave images in text flow
2. **Scroll complexity**: Element-based scrolling doesn't account for multi-line images
3. **Performance**: Each image render might be expensive
4. **Terminal compatibility**: Not all terminals support graphics protocols

## Alternative Approach: Image Panel

Instead of inline images, consider:
- Separate panel for current image
- Press `i` to toggle image view
- Shows image at cursor position
- Simpler to implement, better control

## Files to Modify

1. **src/ui.rs**:
   - `render_document()` - Add image rendering logic
   - Import `StatefulImage` widget (already imported line 25)
   - Handle layout and positioning

2. **src/document.rs** (maybe):
   - If we need to track image dimensions for layout

3. **tests/** (new):
   - Integration test for TUI image rendering
   - Requires terminal emulator testing strategy

## Estimated Time
- MVP (inline images, basic layout): **2-3 hours**
- Polished (proper scrolling, sizing): **4-6 hours**
- Alternative panel approach: **1-2 hours**

## Decision Point

**Question for user**:
- Inline images (more complex, better UX)
- Image panel (simpler, separate view)
- Try experiment first, then decide?
