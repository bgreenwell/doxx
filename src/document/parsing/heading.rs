//! Heading detection and classification
//!
//! This module handles detection of headings from Word paragraphs,
//! including style-based detection, text-based heuristics, and
//! numbering extraction.

use super::super::cleanup::is_likely_sentence;
use super::super::models::*;
use super::formatting::{
    extract_numbering_info, extract_paragraph_text, reconstruct_heading_number,
};
use super::list::is_likely_list_item;
use super::numbering::{extract_heading_number_from_text, HeadingInfo};

/// Detect heading level from Word paragraph style
pub(crate) fn detect_heading_from_paragraph_style(para: &docx_rs::Paragraph) -> Option<u8> {
    // Try to access paragraph properties and style
    if let Some(style) = &para.property.style {
        // Check for heading styles (Heading1, Heading2, etc.)
        if style.val.starts_with("Heading") || style.val.starts_with("heading") {
            if let Some(level_char) = style.val.chars().last() {
                if let Some(level) = level_char.to_digit(10) {
                    return Some(level.min(6) as u8);
                }
            }
            // Default to level 1 for unspecified heading styles
            return Some(1);
        }
    }

    None
}

/// Detect heading with automatic or manual numbering
pub(crate) fn detect_heading_with_numbering(para: &docx_rs::Paragraph) -> Option<HeadingInfo> {
    // First check if this is a heading style
    let heading_level = detect_heading_from_paragraph_style(para)?;

    // Extract text using docx-rs proper text extraction
    let text = extract_paragraph_text(para);

    // Priority order for numbering detection:
    // 1. Manual numbering in text content (highest priority - user explicitly typed)
    // 2. Word's automatic numbering (w:numPr) - explicit numbering properties
    // 3. Style-based automatic generation (lowest priority - our inference)

    // First, check for manual numbering in text content
    if let Some((number, remaining_text)) = extract_heading_number_from_text(&text) {
        return Some(HeadingInfo {
            level: heading_level,
            number: Some(number),
            clean_text: Some(remaining_text),
        });
    }

    // Second, check for Word's automatic numbering
    if let Some(num_pr) = &para.property.numbering_property {
        // This is automatic Word numbering - try to reconstruct
        if let Some((num_id, level)) = extract_numbering_info(num_pr) {
            let number = reconstruct_heading_number(num_id, level, heading_level);
            return Some(HeadingInfo {
                level: heading_level,
                number: Some(number),
                clean_text: Some(text), // Keep original text since number is automatic
            });
        }
    }

    // If no numbering found, return heading info without number
    Some(HeadingInfo {
        level: heading_level,
        number: None,
        clean_text: None,
    })
}

/// Detect headings based on text content and formatting heuristics
pub(crate) fn detect_heading_from_text(text: &str, formatting: &TextFormatting) -> Option<u8> {
    let text = text.trim();

    // Be much more conservative and selective
    if text.len() < 100 && !text.contains('\n') {
        // Exclude common non-heading patterns first
        if is_likely_list_item(text) || is_likely_sentence(text) {
            return None;
        }

        // Exclude if it contains typical sentence patterns
        if text.contains(" the ")
            || text.contains(" and ")
            || text.contains(" with ")
            || text.contains(" for ")
        {
            return None;
        }

        // Strong indicators of headings
        if formatting.bold && text.len() < 60 && text.len() > 5 {
            // Bold text that's reasonably short is likely a heading
            if !text.ends_with('.')
                && !text.ends_with(',')
                && !text.ends_with(';')
                && !text.ends_with(':')
            {
                return Some(determine_heading_level_from_text(text));
            }
        }

        // Check if it's all caps (but not just a short word)
        if text.len() > 15
            && text.len() < 50
            && text.chars().all(|c| {
                c.is_uppercase() || c.is_whitespace() || c.is_numeric() || c.is_ascii_punctuation()
            })
        {
            return Some(1);
        }

        // Very specific patterns that indicate headings
        if text.starts_with("Chapter ") || text.starts_with("Section ") || text.starts_with("Part ")
        {
            return Some(determine_heading_level_from_text(text));
        }

        // Look for standalone phrases that could be headings (very conservative)
        if text.len() < 40
            && text.len() > 10
            && !text.ends_with('.')
            && !text.contains(',')
            && !text.contains('(')
            && !text.contains(':')
        {
            // Check if it has heading-like characteristics
            let words = text.split_whitespace().count();
            if (2..=5).contains(&words) {
                // Must contain at least one meaningful word (longer than 3 chars)
                let has_meaningful_word = text
                    .split_whitespace()
                    .any(|word| word.len() > 3 && word.chars().all(|c| c.is_alphabetic()));

                if has_meaningful_word && text.chars().next().is_some_and(|c| c.is_uppercase()) {
                    return Some(determine_heading_level_from_text(text));
                }
            }
        }
    }

    None
}

/// Determine heading level from text length heuristic
pub(crate) fn determine_heading_level_from_text(text: &str) -> u8 {
    // Simple heuristic: shorter text = higher level (lower number)
    if text.len() < 20 {
        1
    } else if text.len() < 40 {
        2
    } else {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_number_extraction() {
        // Test most common formats (decimal hierarchical)
        assert_eq!(
            extract_heading_number_from_text("1. Introduction"),
            Some(("1".to_string(), "Introduction".to_string()))
        );

        assert_eq!(
            extract_heading_number_from_text("1.1 Project Overview"),
            Some(("1.1".to_string(), "Project Overview".to_string()))
        );

        assert_eq!(
            extract_heading_number_from_text("2.1.1 Something Important"),
            Some(("2.1.1".to_string(), "Something Important".to_string()))
        );

        // Test alternative numbering schemes
        assert_eq!(
            extract_heading_number_from_text("A. First Section"),
            Some(("A".to_string(), "First Section".to_string()))
        );

        assert_eq!(
            extract_heading_number_from_text("I. Roman Numeral"),
            Some(("I".to_string(), "Roman Numeral".to_string()))
        );

        // Test section numbering
        assert_eq!(
            extract_heading_number_from_text("Section 1.2 Overview"),
            Some(("Section 1.2".to_string(), "Overview".to_string()))
        );

        // Test no numbering (should fall back to automatic generation)
        assert_eq!(extract_heading_number_from_text("Introduction"), None);

        // Test titles with numbers that should NOT be treated as numbered headings
        assert_eq!(extract_heading_number_from_text("Heading 1"), None);
        // Note: "Chapter 5 Summary" will match the section pattern, which is intentional
        // The section pattern is designed to match "Chapter 5 Something" formats
        assert_eq!(
            extract_heading_number_from_text("Chapter 5 Summary"),
            Some(("Chapter 5".to_string(), "Summary".to_string()))
        );
        assert_eq!(extract_heading_number_from_text("Version 2"), None);
    }
}
