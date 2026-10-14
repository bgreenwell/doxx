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

#[cfg(test)]
mod tests {
    use super::*;
    use docx_rs::{BreakType, IndentLevel, NumberingId, NumberingProperty, Paragraph, Run};

    #[test]
    fn extract_run_text_handles_text_tab_and_break() {
        let run = Run::new()
            .add_text("hello")
            .add_tab()
            .add_text("world")
            .add_break(BreakType::TextWrapping);
        assert_eq!(extract_run_text(&run), "hello\tworld\n");
    }

    #[test]
    fn extract_paragraph_text_concatenates_runs_and_trims() {
        let para = Paragraph::new()
            .add_run(Run::new().add_text("  Hello, "))
            .add_run(Run::new().add_text("world!  "));
        assert_eq!(extract_paragraph_text(&para), "Hello, world!");
    }

    #[test]
    fn extract_paragraph_text_empty_paragraph_is_empty_string() {
        let para = Paragraph::new();
        assert_eq!(extract_paragraph_text(&para), "");
    }

    #[test]
    fn extract_run_formatting_detects_bold_italic_underline_strike() {
        let run = Run::new()
            .add_text("styled")
            .bold()
            .italic()
            .underline("single")
            .strike();
        let fmt = extract_run_formatting(&run);
        assert!(fmt.bold);
        assert!(fmt.italic);
        assert!(fmt.underline);
        assert!(fmt.strikethrough);
    }

    #[test]
    fn extract_run_formatting_dstrike_also_sets_strikethrough() {
        let run = Run::new().add_text("x").dstrike();
        assert!(extract_run_formatting(&run).strikethrough);
    }

    #[test]
    fn extract_run_formatting_plain_run_has_no_formatting() {
        let run = Run::new().add_text("plain");
        let fmt = extract_run_formatting(&run);
        assert!(!fmt.bold);
        assert!(!fmt.italic);
        assert!(!fmt.underline);
        assert!(!fmt.strikethrough);
        assert!(fmt.color.is_none());
    }

    #[test]
    fn extract_run_formatting_extracts_color_hex() {
        // Regression test for the Debug-string-scraping workaround in
        // extract_run_formatting - docx-rs doesn't expose the color field
        // publicly, so this parses `format!("{color:?}")` output. If that
        // Debug format ever changes, this test should catch it.
        let run = Run::new().add_text("red").color("FF0000");
        let fmt = extract_run_formatting(&run);
        assert_eq!(fmt.color.as_deref(), Some("FF0000"));
    }

    #[test]
    fn extract_numbering_info_returns_id_and_level() {
        let num_pr = NumberingProperty::new().add_num(NumberingId::new(3), IndentLevel::new(1));
        assert_eq!(extract_numbering_info(&num_pr), Some((3, 1)));
    }

    #[test]
    fn extract_numbering_info_defaults_level_to_zero_when_absent() {
        let num_pr = NumberingProperty::new().id(NumberingId::new(5));
        assert_eq!(extract_numbering_info(&num_pr), Some((5, 0)));
    }

    #[test]
    fn extract_numbering_info_none_without_an_id() {
        let num_pr = NumberingProperty::new();
        assert_eq!(extract_numbering_info(&num_pr), None);
    }

    #[test]
    fn reconstruct_heading_number_maps_level_to_dotted_number() {
        assert_eq!(reconstruct_heading_number(1, 0, 1), "1");
        assert_eq!(reconstruct_heading_number(1, 1, 2), "1.1");
        assert_eq!(reconstruct_heading_number(1, 2, 3), "1.1.1");
        assert_eq!(reconstruct_heading_number(1, 3, 4), "1.1.1.1");
    }

    #[test]
    fn reconstruct_heading_number_falls_back_on_heading_level_alone() {
        // (num_id, level) combination doesn't match a known pattern - falls
        // back to heading_level-only logic.
        assert_eq!(reconstruct_heading_number(1, 9, 2), "1.1");
        assert_eq!(reconstruct_heading_number(1, 9, 5), "1.1.1.1");
    }
}
