/// Test case for reproducing Unicode safety issues in doxx
/// This creates a Word document with various Unicode characters that can cause panics
/// when sliced incorrectly in the search functionality

use std::fs::File;
use std::io::Write;

pub fn create_unicode_test_document() -> Result<(), Box<dyn std::error::Error>> {
    // For now, create a simple docx structure manually
    // Later we can use docx-rs for proper document creation
    
    // Create a minimal .docx file with problematic Unicode content
    let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:body>
        <w:p>
            <w:r>
                <w:t>Test with emojis: 🚀 🎉 💻 🔥</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:r>
                <w:t>Accented characters: café naïve résumé jalapeño</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:r>
                <w:t>CJK text: 你好世界 こんにちは 안녕하세요</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:r>
                <w:t>Complex Unicode: 👨‍💻 👩‍🔬 🏴‍☠️ 🏳️‍🌈</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:r>
                <w:t>Mathematical symbols: ∑ ∫ ∞ ≠ ≤ ≥ √ ∂</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:r>
                <w:t>Special characters: "curly quotes" 'single quotes' —em dash— …ellipsis</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:r>
                <w:t>Long line with emojis that will trigger the 77-character truncation bug in search: 🚀🎉💻🔥🌟⭐️✨💫🌈🦄🎯🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭</w:t>
            </w:r>
        </w:p>
    </w:body>
</w:document>"#;

    let mut file = File::create("tests/fixtures/unicode_test_content.xml")?;
    file.write_all(content.as_bytes())?;
    
    println!("Created Unicode test content file");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_content_creation() {
        create_unicode_test_document().expect("Failed to create Unicode test document");
    }
}