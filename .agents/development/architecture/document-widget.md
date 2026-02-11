# Custom Document Widget Implementation Plan

## Overview
Replace the current `Paragraph` widget + image overlay approach with a custom `DocumentWidget` that natively handles both text and images with proper wrapping and positioning.

## Current Architecture (Problems)

```
render_document() {
    1. Build Text from DocumentElements
    2. Render Paragraph widget (handles wrapping)
    3. Overlay images on top (doesn't know about wrapping!)
    └─> Result: Images misaligned when text wraps
}
```

**Issues**:
- Text wrapping breaks image positioning
- Two-pass rendering (text, then images)
- Fragile coordination between Paragraph and image rendering
- Can't disable wrapping without cutting off text

## New Architecture (Solution)

```
DocumentWidget {
    - Takes Vec<DocumentElement> directly
    - Single-pass rendering with full layout control
    - Handles text wrapping AND image positioning
    - Renders everything in correct order
}

render_document() {
    1. Create DocumentWidget(document.elements)
    2. Render widget to frame
    └─> Result: Perfect layout with wrapping + images
}
```

## Widget Design

### File: `src/widgets/document.rs`

```rust
pub struct DocumentWidget<'a> {
    elements: &'a [DocumentElement],
    scroll_offset: usize,
    color_enabled: bool,
    search_results: &'a [SearchResult],
    image_protocols: &'a mut [Box<dyn StatefulProtocol>],
}

impl<'a> DocumentWidget<'a> {
    pub fn new(elements: &'a [DocumentElement]) -> Self { ... }

    pub fn scroll_offset(mut self, offset: usize) -> Self { ... }
    pub fn color_enabled(mut self, enabled: bool) -> Self { ... }
    pub fn search_results(mut self, results: &'a [SearchResult]) -> Self { ... }
    pub fn image_protocols(mut self, protocols: &'a mut [Box<dyn StatefulProtocol>]) -> Self { ... }
}

impl<'a> Widget for DocumentWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Core rendering logic here
    }
}
```

### Rendering Algorithm

```
1. Start at (area.x, area.y)
2. Track current_y position
3. For each visible element:

   match element {
       Heading => {
           - Render heading text with style
           - Advance current_y by 2 (heading + blank line)
       }

       Paragraph => {
           - Split text into runs
           - For each run:
               * Calculate wrapped lines (account for terminal width)
               * Render each line with formatting
               * Advance current_y
           - Add blank line
       }

       Image => {
           - Render image at (area.x, current_y)
           - Use StatefulImage widget with protocol
           - Advance current_y by image height (15) + 2
       }

       List => {
           - Render bullet/number
           - Render list item text with wrapping
           - Advance current_y
       }

       Table => {
           - Render table rows
           - Advance current_y by table height
       }
   }

4. Stop when current_y >= area.y + area.height
```

### Text Wrapping Implementation

```rust
fn wrap_text(text: &str, width: usize, style: Style) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_width = 0;

    for grapheme in text.graphemes(true) {
        let g_width = grapheme.width();

        if current_width + g_width > width {
            // Line is full, start new line
            lines.push(Line::from(current_line));
            current_line = Vec::new();
            current_width = 0;
        }

        current_line.push(Span::styled(grapheme, style));
        current_width += g_width;
    }

    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    lines
}
```

### Element-Specific Rendering

```rust
fn render_heading(
    heading: &str,
    level: u8,
    number: Option<&str>,
    area: Rect,
    buf: &mut Buffer,
    current_y: &mut u16,
) -> () {
    let style = match level {
        1 => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        2 => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    };

    let prefix = match level {
        1 => "■ ",
        2 => "  ▶ ",
        _ => "    ◦ ",
    };

    let text = if let Some(num) = number {
        format!("{}{} {}", prefix, num, heading)
    } else {
        format!("{}{}", prefix, heading)
    };

    buf.set_string(area.x, *current_y, &text, style);
    *current_y += 2; // Heading + blank line
}

fn render_paragraph(
    runs: &[FormattedRun],
    area: Rect,
    buf: &mut Buffer,
    current_y: &mut u16,
    color_enabled: bool,
) -> () {
    let full_text: String = runs.iter().map(|r| r.text.as_str()).collect();
    let wrapped_lines = wrap_text_with_formatting(runs, area.width as usize, color_enabled);

    for line in wrapped_lines {
        if *current_y >= area.y + area.height {
            break; // Off screen
        }

        buf.set_line(area.x, *current_y, &line, area.width);
        *current_y += 1;
    }

    *current_y += 1; // Blank line after paragraph
}

fn render_image(
    protocol_idx: usize,
    description: &str,
    area: Rect,
    frame: &mut Frame,
    current_y: &mut u16,
    protocols: &mut [Box<dyn StatefulProtocol>],
) -> () {
    if let Some(protocol) = protocols.get_mut(protocol_idx) {
        let img_rect = Rect {
            x: area.x,
            y: *current_y,
            width: area.width.min(80),
            height: 15.min(area.y + area.height - *current_y),
        };

        let image_widget = StatefulImage::new(None);
        frame.render_stateful_widget(image_widget, img_rect, protocol);

        *current_y += 15; // Image height
    }

    // Render description
    let desc_line = Line::from(vec![
        Span::styled("🖼️ ", Style::default().fg(Color::Magenta)),
        Span::styled(description, Style::default().fg(Color::Gray)),
    ]);
    buf.set_line(area.x, *current_y, &desc_line, area.width);
    *current_y += 2; // Description + blank
}
```

## Implementation Steps

### Phase 1: Create Widget Structure (20 min)
1. Create `src/widgets/` directory
2. Create `src/widgets/mod.rs` with module exports
3. Create `src/widgets/document.rs` with basic struct
4. Implement `Widget` trait with placeholder render

### Phase 2: Implement Text Rendering (30 min)
1. Add text wrapping function with unicode support
2. Implement `render_heading()`
3. Implement `render_paragraph()` with formatting
4. Implement `render_list()`
5. Test with text-only document

### Phase 3: Implement Image Rendering (20 min)
1. Add `render_image()` function
2. Handle StatefulProtocol integration
3. Test with images.docx

### Phase 4: Implement Tables & Other Elements (20 min)
1. Add `render_table()`
2. Add `render_page_break()`
3. Handle search highlighting

### Phase 5: Integration (30 min)
1. Update `render_document()` in ui.rs to use new widget
2. Remove old Text-building code
3. Remove image overlay code
4. Test with all fixtures

### Phase 6: Polish & Edge Cases (20 min)
1. Handle scrolling edge cases
2. Test terminal resize
3. Test with no images
4. Test with multiple images
5. Performance testing

**Total Estimated Time**: 2 hours 20 minutes

## File Changes

### New Files
- `src/widgets/mod.rs` - Module exports
- `src/widgets/document.rs` - Custom widget implementation

### Modified Files
- `src/ui.rs` - Update `render_document()` to use custom widget
- `src/lib.rs` - Add `mod widgets;` if needed

### Deleted Code (from ui.rs)
- `render_table_enhanced()` - Move to widget
- Image overlay loop - Replaced by widget
- Text building code - Replaced by widget

## Benefits of This Approach

1. **Correct Wrapping + Images**: Single layout pass, perfect positioning
2. **Cleaner Code**: All rendering logic in one place
3. **Better Performance**: No double-rendering
4. **Future-Proof**: Easy to add features:
   - Image captions
   - Text selection
   - Hyperlinks
   - Inline equations
   - Mixed content layouts
5. **Maintainable**: Clear separation of concerns

## Testing Strategy

### Test Documents
1. `tests/fixtures/retro-gaming-guide.docx` - Images + long text
2. `tests/fixtures/images.docx` - Multiple images
3. `tests/fixtures/business-report.docx` - Complex formatting
4. `tests/fixtures/minimal.docx` - Simple, no images
5. `tests/fixtures/formatting-showcase.docx` - All text styles

### Test Cases
- [ ] Text wraps correctly at terminal width
- [ ] Images appear at correct positions
- [ ] Scrolling works smoothly
- [ ] Search highlighting works
- [ ] All text formatting preserved (bold, italic, colors)
- [ ] Lists render with correct indentation
- [ ] Tables render correctly
- [ ] Terminal resize adjusts layout
- [ ] No images mode still works
- [ ] Large documents perform well

## Rollback Plan

If issues arise:
1. Keep current implementation in git
2. Custom widget on feature branch
3. Can revert to current MVP if needed
4. Or fix issues and continue

## Future Enhancements (Post-MVP)

Once widget is working:
- Smart image sizing based on aspect ratio
- Image caching for performance
- Horizontal scrolling for wide content
- Line numbers
- Copy/paste support
- Text selection
- Hyperlink rendering

## Notes

- Use `unicode-width` for proper grapheme width calculation
- Handle terminal width changes via area.width
- StatefulImage widget already handles Kitty protocol
- Can reuse existing text formatting logic
- Keep search highlighting integration
- Maintain color toggle functionality
