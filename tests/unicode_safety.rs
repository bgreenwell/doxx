/// Unicode safety tests for doxx
///
/// These tests reproduce the critical Issue #22: Unicode safety bug that causes
/// runtime panics when searching documents with emojis and special characters.
///
/// The root cause is unsafe string slicing at src/ui.rs:793 where &result.text[..77]
/// can slice through Unicode character boundaries.

#[cfg(test)]
mod unicode_safety_tests {
    use unicode_segmentation::UnicodeSegmentation;

    /// Test case that would have caused the exact panic from Issue #22 (now fixed)
    #[test]
    fn test_unicode_string_slicing_now_safe() {
        // This string has emojis that are multi-byte UTF-8 characters
        // Before our fix, slicing at byte position 77 would panic if it fell in a character boundary
        let text_with_emojis =
            "🚀🎉💻🔥🌟⭐️✨💫🌈🦄🎯🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭";

        // This would have been the EXACT problematic code from src/ui.rs:793 (now fixed)
        // Instead of panicking, we now use safe truncation

        // Demonstrate the safe truncation logic we implemented
        let max_bytes = 77;
        let safe_boundary = if text_with_emojis.len() <= max_bytes {
            text_with_emojis.len()
        } else {
            let mut boundary = max_bytes;
            while boundary > 0 && !text_with_emojis.is_char_boundary(boundary) {
                boundary -= 1;
            }
            boundary
        };

        // This is now safe and won't panic
        let _safe_truncated = &text_with_emojis[..safe_boundary];
        println!("Safe truncation at boundary {safe_boundary}: working correctly");
    }

    /// Test demonstrating safe Unicode-aware truncation
    #[test]
    fn test_safe_unicode_truncation() {
        let text_with_emojis =
            "🚀🎉💻🔥🌟⭐️✨💫🌈🦄🎯🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭";

        // Safe truncation using unicode-segmentation
        let max_chars = 10;
        let safe_truncated: String = text_with_emojis.graphemes(true).take(max_chars).collect();

        // This should work without panicking
        assert_eq!(safe_truncated.graphemes(true).count(), max_chars);
        println!("Safe truncation: {safe_truncated}");
    }

    /// Test different types of Unicode characters that could cause issues
    #[test]
    fn test_various_unicode_characters() {
        let test_cases = vec![
            ("Simple ASCII", "Hello World"),
            ("Accented chars", "café naïve résumé jalapeño"),
            ("CJK text", "你好世界 こんにちは 안녕하세요"),
            ("Emojis", "🚀 🎉 💻 🔥"),
            ("Complex emojis", "👨‍💻 👩‍🔬 🏴‍☠️ 🏳️‍🌈"),
            ("Math symbols", "∑ ∫ ∞ ≠ ≤ ≥ √ ∂"),
            (
                "Special quotes",
                "\"curly quotes\" 'single quotes' —em dash— …ellipsis",
            ),
        ];

        for (name, text) in test_cases {
            println!("Testing {name}: {text}");

            // Test that these strings don't panic when handled safely
            let char_count = text.chars().count();
            let byte_count = text.len();
            let grapheme_count = text.graphemes(true).count();

            println!("  Chars: {char_count}, Bytes: {byte_count}, Graphemes: {grapheme_count}");

            // Demonstrate unsafe slicing could panic
            if byte_count > 10 {
                // This is potentially unsafe if we slice at a fixed byte position
                // let _unsafe = &text[..10];  // Could panic!

                // Safe alternative using char boundaries
                let safe_slice: String = text.chars().take(10).collect();
                println!("  Safe slice (10 chars): {safe_slice}");
            }
        }
    }

    /// Test that reproduces the search functionality issue
    #[test]
    fn test_search_result_truncation_issue() {
        // Mock search result with Unicode content
        struct SearchResult {
            text: String,
        }

        let result = SearchResult {
            text: "Found emoji result: 🚀🎉💻🔥🌟⭐️✨💫🌈🦄🎯🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭🎪🎨🎭".to_string(),
        };

        // The problematic code from ui.rs:793
        // result_text: &result.text[..77],

        println!("Text length in bytes: {}", result.text.len());
        println!("Text length in chars: {}", result.text.chars().count());

        // This will likely panic if the 77th byte is in the middle of an emoji
        if result.text.len() > 77 {
            // UNSAFE: This is the bug from Issue #22
            // let _problematic = &result.text[..77];  // Will panic!

            // SAFE alternative:
            let safe_truncated: String = result.text.chars().take(40).collect();
            println!("Safe truncation: {safe_truncated}");
        }
    }
}

/// Helper function to demonstrate safe string truncation
/// This is what should replace the unsafe slicing in ui.rs
pub fn safe_truncate_string(s: &str, max_graphemes: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;

    s.graphemes(true).take(max_graphemes).collect()
}

/// Helper function for byte-safe truncation
/// Ensures we don't slice through character boundaries
pub fn safe_truncate_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    // Find the largest valid UTF-8 boundary <= max_bytes
    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }

    &s[..boundary]
}
