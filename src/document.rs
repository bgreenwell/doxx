use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

type TableRows = Vec<Vec<TableCell>>;
type NumberingInfo = (i32, u8);
type HeadingNumberInfo = (String, String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub metadata: DocumentMetadata,
    pub elements: Vec<DocumentElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_path: String,
    pub file_size: u64,
    pub word_count: usize,
    pub page_count: usize,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentElement {
    Heading {
        level: u8,
        text: String,
        number: Option<String>,
    },
    Paragraph {
        text: String,
        formatting: TextFormatting,
    },
    List {
        items: Vec<ListItem>,
        ordered: bool,
    },
    Table {
        table: TableData,
    },
    Image {
        description: String,
        width: Option<u32>,
        height: Option<u32>,
    },
    PageBreak,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextFormatting {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub font_size: Option<f32>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub text: String,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<TableCell>,
    pub rows: Vec<Vec<TableCell>>,
    pub metadata: TableMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: String,
    pub alignment: TextAlignment,
    pub formatting: TextFormatting,
    pub data_type: CellDataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    pub column_count: usize,
    pub row_count: usize,
    pub has_headers: bool,
    pub column_widths: Vec<usize>,
    pub column_alignments: Vec<TextAlignment>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum CellDataType {
    #[default]
    Text,
    Number,
    Currency,
    Percentage,
    Date,
    Boolean,
    Empty,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub element_index: usize,
    pub text: String,
    #[allow(dead_code)]
    pub start_pos: usize,
    #[allow(dead_code)]
    pub end_pos: usize,
}

pub async fn load_document(file_path: &Path) -> Result<Document> {
    let file_size = std::fs::metadata(file_path)?.len();

    // For now, create a simple implementation that reads the docx file
    // This is a simplified version to get the project compiling
    let file_data = std::fs::read(file_path)?;
    let docx = docx_rs::read_docx(&file_data)?;

    let title = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled Document")
        .to_string();

    let mut elements = Vec::new();
    let mut word_count = 0;
    let mut heading_tracker = HeadingNumberTracker::new();

    // Enhanced content extraction with style information
    for child in &docx.document.children {
        match child {
            docx_rs::DocumentChild::Paragraph(para) => {
                let mut text = String::new();
                let mut formatting = TextFormatting::default();

                // Check for heading with potential numbering first
                let heading_info = detect_heading_with_numbering(para);

                // Check for list numbering properties (Word's automatic lists)
                let list_info = detect_list_from_paragraph_numbering(para);

                // Extract text and formatting from runs
                for child in &para.children {
                    if let docx_rs::ParagraphChild::Run(run) = child {
                        // Extract formatting from run properties
                        if !formatting.bold && !formatting.italic {
                            // Only extract formatting from the first run with properties
                            formatting = extract_run_formatting(run);
                        }

                        for child in &run.children {
                            if let docx_rs::RunChild::Text(text_elem) = child {
                                text.push_str(&text_elem.text);
                            }
                        }
                    }
                }

                if !text.trim().is_empty() {
                    word_count += text.split_whitespace().count();

                    // Priority: list numbering > heading style > text heuristics
                    if let Some(list_info) = list_info {
                        // This is an automatic Word list item - format with proper indentation
                        let indent = "  ".repeat(list_info.level as usize);
                        let prefix = if list_info.is_ordered {
                            // Level-based formatting to match Word's mixed lists
                            let result = match list_info.level {
                                0 => "1. ".to_string(),                    // Top level numbers
                                1 => "a) ".to_string(),                    // Second level letters
                                2 => "i. ".to_string(), // Third level roman numerals
                                3 => "A. ".to_string(), // Fourth level caps
                                4 => "I. ".to_string(), // Fifth level caps roman
                                _ => format!("{}. ", list_info.level + 1), // Fallback to numbers
                            };
                            result
                        } else {
                            "* ".to_string() // Bullets for unordered
                        };
                        // Mark Word-formatted list items with a special prefix to avoid reprocessing
                        elements.push(DocumentElement::Paragraph {
                            text: format!("__WORD_LIST__{}{}{}", indent, prefix, text.trim()),
                            formatting,
                        });
                    } else {
                        // Check for headings (with or without numbering)
                        if let Some(heading_info) = heading_info {
                            let heading_text = heading_info.clean_text.unwrap_or(text.clone());

                            // If no explicit numbering found, generate automatic numbering
                            let number = if heading_info.number.is_some() {
                                heading_info.number
                            } else {
                                // Generate automatic numbering for consistent heading styles
                                Some(heading_tracker.get_number(heading_info.level))
                            };

                            elements.push(DocumentElement::Heading {
                                level: heading_info.level,
                                text: heading_text,
                                number,
                            });
                        } else {
                            // Fallback to text-based heading detection
                            let level = detect_heading_from_text(&text, &formatting);
                            if let Some(level) = level {
                                elements.push(DocumentElement::Heading {
                                    level,
                                    text,
                                    number: None,
                                });
                            } else {
                                elements.push(DocumentElement::Paragraph { text, formatting });
                            }
                        }
                    }
                }
            }
            docx_rs::DocumentChild::Table(table) => {
                // Extract table data
                if let Some(table_element) = extract_table_data(table) {
                    elements.push(table_element);
                }
            }
            _ => {
                // Handle other document elements (images, etc.) in future
            }
        }
    }

    // Post-process to group consecutive list items (only for text-based lists)
    // Word numbering-based lists are already properly formatted
    let elements = group_list_items(elements);

    // Clean up Word list markers
    let elements = clean_word_list_markers(elements);

    let metadata = DocumentMetadata {
        file_path: file_path.to_string_lossy().to_string(),
        file_size,
        word_count,
        page_count: estimate_page_count(word_count),
        created: None, // Simplified for now
        modified: None,
        author: None,
    };

    Ok(Document {
        title,
        metadata,
        elements,
    })
}

fn detect_heading_from_paragraph_style(para: &docx_rs::Paragraph) -> Option<u8> {
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

#[derive(Debug, Clone)]
struct ListInfo {
    level: u8,
    is_ordered: bool,
}

#[derive(Debug, Clone)]
struct HeadingInfo {
    level: u8,
    number: Option<String>,
    clean_text: Option<String>, // Text with number removed
}

fn detect_list_from_paragraph_numbering(para: &docx_rs::Paragraph) -> Option<ListInfo> {
    // Check if paragraph has numbering properties
    if let Some(num_pr) = &para.property.numbering_property {
        // Extract numbering level (default to 0 if not specified)
        let level = num_pr.level.as_ref().map(|l| l.val as u8).unwrap_or(0);

        // Clean implementation without debug output

        // Enhanced detection for mixed list types (same numId, different levels)
        let is_ordered = if let Some(num_id) = &num_pr.id {
            match (num_id.id, level) {
                // For Word's default mixed list (numId 1):
                // Level 0 = bullets (•)
                // Level 1 = letters (a), b), c))
                // Level 2 = roman numerals (i., ii., iii.)
                (1, 0) => false,          // Top level: bullets
                (1, 1) => true,           // Second level: letters
                (1, 2) => true,           // Third level: roman numerals
                (1, _) => level % 2 == 1, // Pattern for deeper levels
                (_, _) => true,           // Other numIds are typically ordered
            }
        } else {
            false
        };

        return Some(ListInfo { level, is_ordered });
    }
    None
}

fn detect_heading_with_numbering(para: &docx_rs::Paragraph) -> Option<HeadingInfo> {
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

/// Extract text from paragraph using docx-rs properly
fn extract_paragraph_text(para: &docx_rs::Paragraph) -> String {
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
fn extract_run_text(run: &docx_rs::Run) -> String {
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

/// Extract numbering information from docx-rs numbering properties
fn extract_numbering_info(num_pr: &docx_rs::NumberingProperty) -> Option<NumberingInfo> {
    let num_id = num_pr.id.as_ref()?.id as i32;
    let level = num_pr.level.as_ref().map(|l| l.val as u8).unwrap_or(0);
    Some((num_id, level))
}

/// Reconstruct heading number from Word's numbering system
fn reconstruct_heading_number(num_id: i32, level: u8, heading_level: u8) -> String {
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

/// Track heading numbering state across the document
#[derive(Debug)]
struct HeadingNumberTracker {
    counters: [u32; 6], // Support up to 6 heading levels
}

impl HeadingNumberTracker {
    fn new() -> Self {
        Self { counters: [0; 6] }
    }

    fn get_number(&mut self, level: u8) -> String {
        let level_index = (level.saturating_sub(1) as usize).min(5);

        // Increment current level
        self.counters[level_index] += 1;

        // Reset all deeper levels
        for i in (level_index + 1)..6 {
            self.counters[i] = 0;
        }

        // Build number string (1.2.3 format)
        let mut parts = Vec::new();
        for i in 0..=level_index {
            if self.counters[i] > 0 {
                parts.push(self.counters[i].to_string());
            }
        }

        parts.join(".")
    }
}

// Lazy static regex patterns for heading number detection
// Focused on common patterns for manual numbering in text
static HEADING_NUMBER_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Standard decimal numbering: "1.", "1.1", "1.1.1", "2.1.1" (most common)
        Regex::new(r"^(\d+(?:\.\d+)*\.?)\s+(.+)$").unwrap(),
        // Section numbering: "Section 1.2", "Chapter 3"
        Regex::new(r"^((?:Section|Chapter|Part)\s+\d+(?:\.\d+)*\.?)\s+(.+)$").unwrap(),
        // Alternative numbering schemes (less common, but still useful)
        Regex::new(r"^([A-Z]\.)\s+(.+)$").unwrap(), // "A. Introduction"
        Regex::new(r"^([IVX]+\.)\s+(.+)$").unwrap(), // "I. Overview"
    ]
});

fn extract_heading_number_from_text(text: &str) -> Option<HeadingNumberInfo> {
    let text = text.trim();

    // Early return for empty text
    if text.is_empty() {
        return None;
    }

    // Try each pattern until one matches
    for pattern in HEADING_NUMBER_PATTERNS.iter() {
        if let Some(captures) = pattern.captures(text) {
            if let (Some(number_match), Some(text_match)) = (captures.get(1), captures.get(2)) {
                let number = number_match.as_str().trim_end_matches('.');
                let remaining_text = text_match.as_str().trim();

                // Only return if we have both number and meaningful text
                if !number.is_empty() && !remaining_text.is_empty() {
                    return Some((number.to_string(), remaining_text.to_string()));
                }
            }
        }
    }

    None
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
    }
}

fn extract_run_formatting(run: &docx_rs::Run) -> TextFormatting {
    let mut formatting = TextFormatting::default();

    // Access run properties directly (they're not optional in current API)
    let props = &run.run_property;
    formatting.bold = props.bold.is_some();
    formatting.italic = props.italic.is_some();
    formatting.underline = props.underline.is_some();

    // Extract color information
    if let Some(color) = &props.color {
        // Extract color value through debug formatting as a workaround for private field access
        let color_debug = format!("{color:?}");
        if let Some(start) = color_debug.find("val: \"") {
            if let Some(end) = color_debug[start + 6..].find("\"") {
                let color_val = &color_debug[start + 6..start + 6 + end];
                formatting.color = Some(color_val.to_string());
            }
        }
    }

    // For now, skip font size extraction due to API complexity
    // TODO: Add font size extraction when we understand the API better

    formatting
}

fn detect_heading_from_text(text: &str, formatting: &TextFormatting) -> Option<u8> {
    let text = text.trim();

    // Be much more conservative and selective
    if text.len() < 100 && !text.contains('\n') {
        // Exclude common non-heading patterns first
        if is_likely_list_item(text) || is_likely_sentence(text) {
            return None;
        }

        // Exclude patterns that are clearly not headings
        if text.starts_with("⏺")
            || text.starts_with("⎿")
            || text.starts_with("☐")
            || text.starts_with("☒")
        {
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

fn determine_heading_level_from_text(text: &str) -> u8 {
    // Simple heuristic: shorter text = higher level (lower number)
    if text.len() < 20 {
        1
    } else if text.len() < 40 {
        2
    } else {
        3
    }
}

fn is_likely_list_item(text: &str) -> bool {
    let text = text.trim();

    // Skip Word-formatted list items to avoid reprocessing
    if text.starts_with("__WORD_LIST__") {
        return false;
    }

    // Check for numbered list patterns that are NOT headings
    if text.starts_with(char::is_numeric) {
        // If it starts with a number followed by "." and then has substantial content,
        // it's likely a list item, not a heading
        if let Some(dot_pos) = text.find('.') {
            let after_dot = &text[dot_pos + 1..].trim();
            // If there's substantial content after the number and dot, it's likely a list item
            if after_dot.len() > 20 {
                return true;
            }
        }
    }

    // Check for bullet point patterns
    if text.starts_with("• ") || text.starts_with("- ") || text.starts_with("* ") {
        return true;
    }

    // Check for lettered lists
    if text.len() > 3 && text.chars().nth(1) == Some('.') {
        let first_char = text.chars().next().unwrap();
        if first_char.is_ascii_lowercase() || first_char.is_ascii_uppercase() {
            return true;
        }
    }

    false
}

fn group_list_items(elements: Vec<DocumentElement>) -> Vec<DocumentElement> {
    let mut result = Vec::new();
    let mut current_list_items = Vec::new();
    let mut current_list_ordered = false;

    for element in elements {
        match &element {
            DocumentElement::Paragraph { text, .. } => {
                if is_likely_list_item(text) {
                    // Determine if this is an ordered list item
                    let is_ordered = text.trim().starts_with(char::is_numeric);

                    // If we're starting a new list or switching list types, finish the current list
                    if !current_list_items.is_empty() && is_ordered != current_list_ordered {
                        result.push(DocumentElement::List {
                            items: std::mem::take(&mut current_list_items),
                            ordered: current_list_ordered,
                        });
                    }

                    current_list_ordered = is_ordered;

                    // Calculate nesting level from indentation
                    let level = calculate_list_level(text);

                    // Clean the text (remove bullet/number prefix)
                    let clean_text = clean_list_item_text(text);

                    current_list_items.push(ListItem {
                        text: clean_text,
                        level,
                    });
                } else {
                    // Not a list item, so finish any current list
                    if !current_list_items.is_empty() {
                        result.push(DocumentElement::List {
                            items: std::mem::take(&mut current_list_items),
                            ordered: current_list_ordered,
                        });
                    }
                    result.push(element);
                }
            }
            _ => {
                // Non-paragraph element, finish any current list
                if !current_list_items.is_empty() {
                    result.push(DocumentElement::List {
                        items: std::mem::take(&mut current_list_items),
                        ordered: current_list_ordered,
                    });
                }
                result.push(element);
            }
        }
    }

    // Don't forget the last list if the document ends with one
    if !current_list_items.is_empty() {
        result.push(DocumentElement::List {
            items: current_list_items,
            ordered: current_list_ordered,
        });
    }

    result
}

fn calculate_list_level(text: &str) -> u8 {
    // Count leading whitespace to determine nesting level
    let leading_spaces = text.len() - text.trim_start().len();

    // Convert spaces to levels (every 2-4 spaces = 1 level)
    // Use 2 spaces per level as it's common in Word documents
    (leading_spaces / 2) as u8
}

fn clean_list_item_text(text: &str) -> String {
    let text = text.trim();

    // Remove bullet points (handle Unicode characters properly)
    if text.starts_with("• ") {
        return text.strip_prefix("• ").unwrap_or(text).trim().to_string();
    }
    if text.starts_with("- ") || text.starts_with("* ") {
        return text[2..].trim().to_string();
    }

    // Remove numbered list prefixes
    if let Some(dot_pos) = text.find('.') {
        let prefix = &text[..dot_pos];
        if prefix.chars().all(|c| c.is_ascii_digit()) {
            return text[dot_pos + 1..].trim().to_string();
        }
    }

    // Remove lettered list prefixes
    if text.len() > 3 && text.chars().nth(1) == Some('.') {
        let first_char = text.chars().next().unwrap();
        if first_char.is_ascii_lowercase() || first_char.is_ascii_uppercase() {
            return text[2..].trim().to_string();
        }
    }

    text.to_string()
}

fn is_likely_sentence(text: &str) -> bool {
    let text = text.trim();

    // If it contains multiple sentences, it's probably not a heading
    if text.matches(". ").count() > 1 {
        return true;
    }

    // If it ends with common sentence endings and is long, it's probably a sentence
    if text.len() > 80 && (text.ends_with('.') || text.ends_with('!') || text.ends_with('?')) {
        return true;
    }

    // If it contains common sentence connectors, it's likely a sentence
    if text.contains(" and ")
        || text.contains(" but ")
        || text.contains(" however ")
        || text.contains(" therefore ")
    {
        return true;
    }

    false
}

fn estimate_page_count(word_count: usize) -> usize {
    // Rough estimate: 250 words per page
    (word_count as f32 / 250.0).ceil() as usize
}

pub fn search_document(document: &Document, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for (element_index, element) in document.elements.iter().enumerate() {
        let text = match element {
            DocumentElement::Heading { text, .. } => text,
            DocumentElement::Paragraph { text, .. } => text,
            DocumentElement::List { items, .. } => {
                // Search in list items
                for item in items {
                    let text_lower = item.text.to_lowercase();
                    if let Some(start_pos) = text_lower.find(&query_lower) {
                        results.push(SearchResult {
                            element_index,
                            text: item.text.clone(),
                            start_pos,
                            end_pos: start_pos + query.len(),
                        });
                    }
                }
                continue;
            }
            DocumentElement::Table { table } => {
                // Search in table content
                for header in &table.headers {
                    let text_lower = header.content.to_lowercase();
                    if let Some(start_pos) = text_lower.find(&query_lower) {
                        results.push(SearchResult {
                            element_index,
                            text: header.content.clone(),
                            start_pos,
                            end_pos: start_pos + query.len(),
                        });
                    }
                }
                for row in &table.rows {
                    for cell in row {
                        let text_lower = cell.content.to_lowercase();
                        if let Some(start_pos) = text_lower.find(&query_lower) {
                            results.push(SearchResult {
                                element_index,
                                text: cell.content.clone(),
                                start_pos,
                                end_pos: start_pos + query.len(),
                            });
                        }
                    }
                }
                continue;
            }
            DocumentElement::Image { description, .. } => description,
            DocumentElement::PageBreak => continue,
        };

        let text_lower = text.to_lowercase();
        if let Some(start_pos) = text_lower.find(&query_lower) {
            results.push(SearchResult {
                element_index,
                text: text.clone(),
                start_pos,
                end_pos: start_pos + query.len(),
            });
        }
    }

    results
}

pub fn generate_outline(document: &Document) -> Vec<OutlineItem> {
    let mut outline = Vec::new();

    for (index, element) in document.elements.iter().enumerate() {
        if let DocumentElement::Heading {
            level,
            text,
            number,
        } = element
        {
            let title = if let Some(number) = number {
                format!("{number} {text}")
            } else {
                text.clone()
            };
            outline.push(OutlineItem {
                title,
                level: *level,
                element_index: index,
            });
        }
    }

    outline
}

fn extract_table_data(table: &docx_rs::Table) -> Option<DocumentElement> {
    let mut header_cells = Vec::new();
    let mut data_rows = Vec::new();

    let mut is_first_row = true;
    let mut _raw_headers = Vec::new();
    let mut raw_rows = Vec::new();

    // First pass: extract raw text content
    for table_child in &table.rows {
        let docx_rs::TableChild::TableRow(row) = table_child;
        let mut row_cells = Vec::new();

        for row_child in &row.cells {
            let docx_rs::TableRowChild::TableCell(cell) = row_child;
            let mut cell_text = String::new();
            let mut cell_formatting = TextFormatting::default();

            // Extract text and formatting from all content in the cell
            for content in &cell.children {
                match content {
                    docx_rs::TableCellContent::Paragraph(para) => {
                        for para_child in &para.children {
                            if let docx_rs::ParagraphChild::Run(run) = para_child {
                                // Extract formatting from the first run
                                if !cell_formatting.bold && !cell_formatting.italic {
                                    cell_formatting = extract_run_formatting(run);
                                }

                                for run_child in &run.children {
                                    if let docx_rs::RunChild::Text(text_elem) = run_child {
                                        if !cell_text.is_empty() && !cell_text.ends_with(' ') {
                                            cell_text.push(' ');
                                        }
                                        cell_text.push_str(&text_elem.text);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // Handle nested tables or other content if needed
                    }
                }
            }

            let table_cell =
                TableCell::new(cell_text.trim().to_string()).with_formatting(cell_formatting);
            row_cells.push(table_cell);
        }

        if !row_cells.is_empty() {
            let raw_text: Vec<String> = row_cells.iter().map(|c| c.content.clone()).collect();

            if is_first_row && appears_to_be_header(&raw_text) {
                _raw_headers = raw_text;
                header_cells = row_cells;
                is_first_row = false;
            } else {
                raw_rows.push(raw_text);
                data_rows.push(row_cells);
                is_first_row = false;
            }
        }
    }

    // If no headers were detected, use the first row as headers
    if header_cells.is_empty() && !data_rows.is_empty() {
        header_cells = data_rows.remove(0);
        raw_rows.remove(0);
    }

    // Return table only if it has content
    if !header_cells.is_empty() || !data_rows.is_empty() {
        let table_data = TableData::new(header_cells, data_rows);
        Some(DocumentElement::Table { table: table_data })
    } else {
        None
    }
}

fn appears_to_be_header(row: &[String]) -> bool {
    // Heuristics to detect if a row is likely a header
    let total_chars: usize = row.iter().map(|cell| cell.len()).sum();
    let avg_length = if !row.is_empty() {
        total_chars / row.len()
    } else {
        0
    };

    // Headers tend to be shorter and more concise
    if avg_length > 50 {
        return false;
    }

    // Check if most cells contain typical header words or are short phrases
    let header_indicators = row
        .iter()
        .filter(|cell| {
            let cell_lower = cell.to_lowercase();
            let word_count = cell.split_whitespace().count();

            // Short phrases (1-3 words) are often headers
            if word_count <= 3 && !cell.trim().is_empty() {
                return true;
            }

            // Common header words
            if cell_lower.contains("name")
                || cell_lower.contains("date")
                || cell_lower.contains("amount")
                || cell_lower.contains("type")
                || cell_lower.contains("status")
                || cell_lower.contains("id")
                || cell_lower.contains("description")
                || cell_lower.contains("count")
            {
                return true;
            }

            false
        })
        .count();

    // If more than half the cells look like headers, treat the row as a header
    header_indicators > row.len() / 2
}

// Enhanced table processing functions
impl TableData {
    pub fn new(headers: Vec<TableCell>, rows: Vec<Vec<TableCell>>) -> Self {
        let column_count = headers.len();
        let row_count = rows.len();
        let has_headers = !headers.is_empty();

        // Calculate optimal column widths
        let column_widths = calculate_column_widths(&headers, &rows);

        // Determine column alignments
        let column_alignments = determine_column_alignments(&headers, &rows);

        let metadata = TableMetadata {
            column_count,
            row_count,
            has_headers,
            column_widths,
            column_alignments,
            title: None,
        };

        Self {
            headers,
            rows,
            metadata,
        }
    }

    pub fn _get_column_width(&self, column_index: usize) -> usize {
        self.metadata
            .column_widths
            .get(column_index)
            .copied()
            .unwrap_or(10)
    }

    pub fn _get_column_alignment(&self, column_index: usize) -> TextAlignment {
        self.metadata
            .column_alignments
            .get(column_index)
            .copied()
            .unwrap_or(TextAlignment::Left)
    }
}

impl TableCell {
    pub fn new(content: String) -> Self {
        let data_type = detect_cell_data_type(&content);
        let alignment = default_alignment_for_type(data_type);

        Self {
            content,
            alignment,
            formatting: TextFormatting::default(),
            data_type,
        }
    }

    pub fn _with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_formatting(mut self, formatting: TextFormatting) -> Self {
        self.formatting = formatting;
        self
    }

    pub fn display_width(&self) -> usize {
        // Calculate display width considering unicode characters
        unicode_segmentation::UnicodeSegmentation::graphemes(self.content.as_str(), true).count()
    }
}

fn calculate_column_widths(headers: &[TableCell], rows: &TableRows) -> Vec<usize> {
    if headers.is_empty() {
        return Vec::new();
    }

    let mut widths = headers
        .iter()
        .map(|h| h.display_width())
        .collect::<Vec<_>>();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if let Some(current_width) = widths.get_mut(i) {
                *current_width = (*current_width).max(cell.display_width());
            }
        }
    }

    // Ensure minimum width of 3 characters per column
    widths.iter_mut().for_each(|w| *w = (*w).max(3));

    widths
}

fn determine_column_alignments(headers: &[TableCell], rows: &TableRows) -> Vec<TextAlignment> {
    let column_count = headers.len();
    let mut alignments = vec![TextAlignment::Left; column_count];

    for (col_index, alignment) in alignments.iter_mut().enumerate().take(column_count) {
        let mut numeric_count = 0;
        let mut total_count = 0;

        // Check data types in this column
        for row in rows {
            if let Some(cell) = row.get(col_index) {
                total_count += 1;
                if matches!(
                    cell.data_type,
                    CellDataType::Number | CellDataType::Currency | CellDataType::Percentage
                ) {
                    numeric_count += 1;
                }
            }
        }

        // If more than 70% of cells are numeric, right-align the column
        if total_count > 0 && (numeric_count as f32 / total_count as f32) > 0.7 {
            *alignment = TextAlignment::Right;
        }
    }

    alignments
}

fn detect_cell_data_type(content: &str) -> CellDataType {
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return CellDataType::Empty;
    }

    // Check for currency
    if trimmed.starts_with('$') || trimmed.starts_with('€') || trimmed.starts_with('£') {
        return CellDataType::Currency;
    }

    // Check for percentage
    if trimmed.ends_with('%') {
        return CellDataType::Percentage;
    }

    // Check for boolean
    let lower = trimmed.to_lowercase();
    if matches!(lower.as_str(), "true" | "false" | "yes" | "no" | "y" | "n") {
        return CellDataType::Boolean;
    }

    // Check for number (including with commas)
    let number_candidate = trimmed.replace(',', "");
    if number_candidate.parse::<f64>().is_ok() {
        return CellDataType::Number;
    }

    // Check for date patterns (basic)
    if trimmed.contains('/') || trimmed.contains('-') {
        let parts: Vec<&str> = trimmed.split(['/', '-']).collect();
        if parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok()) {
            return CellDataType::Date;
        }
    }

    CellDataType::Text
}

fn default_alignment_for_type(data_type: CellDataType) -> TextAlignment {
    match data_type {
        CellDataType::Number | CellDataType::Currency | CellDataType::Percentage => {
            TextAlignment::Right
        }
        CellDataType::Boolean => TextAlignment::Center,
        _ => TextAlignment::Left,
    }
}

#[derive(Debug, Clone)]
pub struct OutlineItem {
    pub title: String,
    pub level: u8,
    pub element_index: usize,
}

fn clean_word_list_markers(elements: Vec<DocumentElement>) -> Vec<DocumentElement> {
    elements
        .into_iter()
        .map(|element| match element {
            DocumentElement::Paragraph { text, formatting } => {
                let cleaned_text = if text.starts_with("__WORD_LIST__") {
                    text.strip_prefix("__WORD_LIST__")
                        .unwrap_or(&text)
                        .to_string()
                } else {
                    text
                };
                DocumentElement::Paragraph {
                    text: cleaned_text,
                    formatting,
                }
            }
            DocumentElement::List { items, ordered } => {
                let cleaned_items = items
                    .into_iter()
                    .map(|item| {
                        let cleaned_text = if item.text.starts_with("__WORD_LIST__") {
                            item.text
                                .strip_prefix("__WORD_LIST__")
                                .unwrap_or(&item.text)
                                .to_string()
                        } else {
                            item.text
                        };
                        ListItem {
                            text: cleaned_text,
                            level: item.level,
                        }
                    })
                    .collect();
                DocumentElement::List {
                    items: cleaned_items,
                    ordered,
                }
            }
            other => other,
        })
        .collect()
}
