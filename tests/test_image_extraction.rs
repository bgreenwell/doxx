use doxx::image_extractor::ImageExtractor;
use std::path::Path;

#[test]
fn test_image_extraction_from_images_docx() {
    let docx_path = Path::new("tests/fixtures/images.docx");
    assert!(docx_path.exists(), "images.docx test file should exist");

    let mut extractor = ImageExtractor::new().expect("Should create image extractor");

    // Extract images from the DOCX
    let result = extractor.extract_images_from_docx(docx_path);
    assert!(
        result.is_ok(),
        "Image extraction should succeed: {result:?}"
    );

    // Check if any images were extracted
    let images = extractor.list_images();
    println!("Extracted {} images:", images.len());
    for (rel_id, path) in &images {
        println!("  {}: {}", rel_id, path.display());
        assert!(path.exists(), "Extracted image file should exist");
    }

    // If there are images, test that we can read the data
    if !images.is_empty() {
        let (first_rel_id, _) = &images[0];
        let image_data = extractor.get_image_data(first_rel_id);
        assert!(image_data.is_ok(), "Should be able to read image data");
        assert!(
            !image_data.unwrap().is_empty(),
            "Image data should not be empty"
        );
    }
}

#[test]
fn test_terminal_image_capabilities() {
    use doxx::terminal_image::{TerminalImageRenderer, TerminalImageSupport};

    // Test that we can create renderers with specific support
    let kitty_renderer = TerminalImageRenderer::with_support(TerminalImageSupport::Kitty);
    assert!(kitty_renderer.can_display_images());

    let no_support_renderer = TerminalImageRenderer::with_support(TerminalImageSupport::None);
    assert!(!no_support_renderer.can_display_images());
}
