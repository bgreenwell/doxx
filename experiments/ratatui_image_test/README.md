# ratatui-image Test Experiment

## Purpose
Test ratatui-image rendering capabilities in WezTerm (Kitty graphics protocol).

## What This Tests
- Basic ratatui-image integration with ratatui 0.29
- StatefulProtocol usage for rendering images
- Picker initialization and protocol selection
- Terminal graphics protocol detection

## How to Run
```bash
cd experiments/ratatui_image_test
cargo run --release
```

Press `q` to quit.

## What You Should See
- A gradient test image (red-green gradient with blue channel)
- Image rendered inline in the TUI
- If WezTerm Kitty protocol works: full-color image display
- If not: fallback to half-blocks or other protocol

## Code Structure
- **Picker::from_query_stdio()**: Auto-detects terminal capabilities
- **picker.new_resize_protocol()**: Creates StatefulProtocol from DynamicImage
- **StatefulImage::new()**: Widget for rendering
- **render_stateful_widget()**: Renders image with stateful protocol

## Next Steps
If this works successfully:
1. Apply same pattern to doxx's `render_document()` function
2. Track image index alongside protocol storage
3. Handle multiple images in document
4. Add proper error handling and fallbacks

## Testing with Real Image
To test with a real image file instead of generated gradient, modify `App::new()`:

```rust
// Replace the gradient with:
let img = image::open("path/to/test.png")?;
let protocol = picker.new_resize_protocol(img);
```
