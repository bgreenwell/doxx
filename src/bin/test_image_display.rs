use doxx::terminal_image::TerminalImageRenderer;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("🎨 doxx Terminal Image Display Test");
    println!("===================================");

    let renderer = TerminalImageRenderer::new();
    println!("Detected terminal image support: {:?}", renderer.support());
    renderer.print_capabilities();

    println!("\n🖼️  Testing image display with extracted images...");

    // Test with the extracted images
    let test_images = [
        "/tmp/doxx_test_images/image1.jpg",
        "/tmp/doxx_test_images/image2.jpeg",
        "/tmp/doxx_test_images/image3.png",
    ];

    for (i, image_path) in test_images.iter().enumerate() {
        let path = Path::new(image_path);
        if path.exists() {
            println!("\n📷 Displaying image {}: {}", i + 1, image_path);
            match renderer.render_image_from_path(path, &format!("Test image {}", i + 1)) {
                Ok(_) => println!("✅ Image displayed successfully"),
                Err(e) => println!("❌ Failed to display image: {e}"),
            }
            println!("---");
        } else {
            println!("⚠️  Image not found: {image_path}");
            println!("   Run: cargo run -- tests/fixtures/images.docx --extract-images /tmp/doxx_test_images");
        }
    }

    Ok(())
}
