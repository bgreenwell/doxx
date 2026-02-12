//! Text extraction and formatting utilities
//!
//! This module handles extraction of text and formatting information
//! from docx-rs paragraph and run elements.

use super::super::models::*;

/// Extract plain text from a paragraph, handling various child elements
pub(crate) fn extract_paragraph_text(para: &docx_rs::Paragraph) -> String {
    let mut text = String::new();

    for child in &para.children {
        match child {
            docx_rs::ParagraphChild::Run(run) => {
                text.push_str(&extract_run_text(run));
            }
            docx_rs::ParagraphChild::Insert(insert) => {
                // Handle insertions (track changes) - simplified approach
                // Since InsertChild might be different from Run, we'll extract text differently
                // This is a placeholder - in practice we'd need to handle the specific types
                for child in &insert.children {
                    if let docx_rs::InsertChild::Run(run) = child {
                        text.push_str(&extract_run_text(run));
                    }
                }
            }
            docx_rs::ParagraphChild::Delete(_) => {
                // Skip deletions (track changes)
            }
            _ => {
                // Handle other paragraph children if needed
            }
        }
    }

    text.trim().to_string()
}

/// Extract text from a run using docx-rs features
pub(crate) fn extract_run_text(run: &docx_rs::Run) -> String {
    let mut text = String::new();

    for child in &run.children {
        match child {
            docx_rs::RunChild::Text(text_elem) => {
                text.push_str(&text_elem.text);
            }
            docx_rs::RunChild::Tab(_) => {
                text.push('\t');
            }
            docx_rs::RunChild::Break(_) => {
                // Break types are private, so we'll just add a line break
                text.push('\n');
            }
            docx_rs::RunChild::Drawing(_) => {
                text.push_str("[Image]");
            }
            _ => {
                // Handle other run children
            }
        }
    }

    text
}

/// Extract formatting information from a run
pub(crate) fn extract_run_formatting(run: &docx_rs::Run) -> TextFormatting {
    let mut formatting = TextFormatting::default();

    // Access run properties directly (they're not optional in current API)
    let props = &run.run_property;
    formatting.bold = props.bold.is_some();
    formatting.italic = props.italic.is_some();
    formatting.underline = props.underline.is_some();

    formatting.strikethrough = props.strike.is_some() || props.dstrike.is_some();

    // Extract color information
    if let Some(color) = &props.color {
        // Extract color value through debug formatting as a workaround for private field access
        let color_debug = format!("{color:?}");
        if let Some(start) = color_debug.find("val: \"") {
            // Safe: searching for ASCII strings in debug output
            let search_from = start + 6; // length of "val: \""
            if let Some(end) = color_debug[search_from..].find("\"") {
                let color_val = &color_debug[search_from..search_from + end];
                formatting.color = Some(color_val.to_string());
            }
        }
    }

    // For now, skip font size extraction due to API complexity
    // TODO: Add font size extraction when we understand the API better

    formatting
}

/// Extract numbering information from docx-rs numbering properties
pub(crate) fn extract_numbering_info(num_pr: &docx_rs::NumberingProperty) -> Option<NumberingInfo> {
    let num_id = num_pr.id.as_ref()?.id as i32;
    let level = num_pr.level.as_ref().map(|l| l.val as u8).unwrap_or(0);
    Some((num_id, level))
}

/// Reconstruct heading number from Word's numbering system
pub(crate) fn reconstruct_heading_number(num_id: i32, level: u8, heading_level: u8) -> String {
    // This is a simplified reconstruction
    // In a full implementation, we'd need to access the numbering definitions
    // and track the current state across the document
    match (num_id, level, heading_level) {
        // Standard heading numbering schemes
        (_, 0, 1) => "1".to_string(),
        (_, 1, 2) => "1.1".to_string(),
        (_, 2, 3) => "1.1.1".to_string(),
        (_, 3, 4) => "1.1.1.1".to_string(),
        _ => {
            // Fallback based on heading level
            match heading_level {
                1 => "1".to_string(),
                2 => "1.1".to_string(),
                3 => "1.1.1".to_string(),
                _ => "1.1.1.1".to_string(),
            }
        }
    }
}
