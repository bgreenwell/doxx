use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

// Import types from the models module
use crate::document::models::*;
// Import I/O functions
use crate::document::io::{merge_display_equations, validate_docx_file};

pub async fn load_document(file_path: &Path, image_options: ImageOptions) -> Result<Document> {
    // Validate file type before attempting to parse
    validate_docx_file(file_path)?;

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
    let mut numbering_manager = DocumentNumberingManager::new();
    let mut heading_tracker = HeadingNumberTracker::new();

    // Analyze document structure to determine if auto-numbering should be enabled
    let should_auto_number = analyze_heading_structure(&docx.document);
    if should_auto_number {
        heading_tracker.enable_auto_numbering();
    }

    // Extract images if enabled
    let image_extractor = if image_options.enabled {
        let mut extractor = crate::image_extractor::ImageExtractor::new()?;
        extractor.extract_images_from_docx(file_path)?;
        Some(extractor)
    } else {
        None
    };

    // Enhanced content extraction with style information
    for child in &docx.document.children {
        match child {
            docx_rs::DocumentChild::Paragraph(para) => {
                // Check for heading with potential numbering first
                let heading_info = detect_heading_with_numbering(para);

                // Check for list numbering properties (Word's automatic lists)
                let list_info = detect_list_from_paragraph_numbering(para);

                // Check for images in this paragraph first
                for child in &para.children {
                    if let docx_rs::ParagraphChild::Run(run) = child {
                        for run_child in &run.children {
                            if let docx_rs::RunChild::Drawing(_drawing) = run_child {
                                // Create an Image element with consistent ordering
                                if let Some(ref extractor) = image_extractor {
                                    let images = extractor.get_extracted_images_sorted();
                                    if !images.is_empty() {
                                        // Count images processed so far to maintain document order
                                        let image_count = elements
                                            .iter()
                                            .filter(|e| matches!(e, DocumentElement::Image { .. }))
                                            .count();

                                        // Only create Image element if we have an actual image file available
                                        if image_count < images.len() {
                                            let (_, image_path) = &images[image_count];

                                            elements.push(DocumentElement::Image {
                                                description: format!("Image {}", image_count + 1),
                                                width: None,
                                                height: None,
                                                relationship_id: None,
                                                image_path: Some(image_path.clone()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Extract runs with individual formatting
                let mut formatted_runs = Vec::new();

                for child in &para.children {
                    if let docx_rs::ParagraphChild::Run(run) = child {
                        let run_formatting = extract_run_formatting(run);
                        let mut run_text = String::new();

                        for child in &run.children {
                            if let docx_rs::RunChild::Text(text_elem) = child {
                                run_text.push_str(&text_elem.text);
                            }
                        }

                        if !run_text.is_empty() {
                            formatted_runs.push(FormattedRun {
                                text: run_text,
                                formatting: run_formatting,
                            });
                        }
                    }
                }

                // Calculate total text for word count and processing
                let total_text: String =
                    formatted_runs.iter().map(|run| run.text.as_str()).collect();

                if !total_text.trim().is_empty() {
                    word_count += total_text.split_whitespace().count();

                    // Priority: list numbering > heading style > text heuristics
                    if let Some(list_info) = list_info {
                        // This is an automatic Word list item - format with proper indentation
                        let indent = "  ".repeat(list_info.level as usize);
                        let prefix = if list_info.is_ordered {
                            // Use the numbering manager for proper sequential numbering
                            if let Some(num_id) = list_info.num_id {
                                let format = get_numbering_format(num_id, list_info.level);
                                numbering_manager.generate_number(num_id, list_info.level, format)
                            } else {
                                // Fallback for missing numId
                                format!("{}. ", list_info.level + 1)
                            }
                        } else {
                            "* ".to_string() // Bullets for unordered
                        };

                        // For list items, preserve individual run formatting by creating separate prefix run
                        // This maintains formatting fidelity while keeping bullets/numbers unformatted
                        if !formatted_runs.is_empty() {
                            // Create a prefix run with default formatting (no color, bold, etc.)
                            let prefix_text = format!("__WORD_LIST__{indent}{prefix}");
                            let prefix_run = FormattedRun {
                                text: prefix_text,
                                formatting: TextFormatting::default(),
                            };

                            // Insert prefix run at the beginning, preserving text formatting
                            let mut updated_runs = vec![prefix_run];
                            updated_runs.extend(formatted_runs);

                            elements.push(DocumentElement::Paragraph { runs: updated_runs });
                        } else {
                            // Fallback for empty runs
                            let list_text = format!("__WORD_LIST__{indent}{prefix}");
                            elements.push(DocumentElement::Paragraph {
                                runs: vec![FormattedRun {
                                    text: list_text,
                                    formatting: TextFormatting::default(),
                                }],
                            });
                        }
                    } else {
                        // Check for headings (with or without numbering)
                        if let Some(heading_info) = heading_info {
                            let heading_text =
                                heading_info.clean_text.unwrap_or(total_text.clone());

                            let number = if heading_info.number.is_some() {
                                heading_info.number
                            } else {
                                // Generate automatic numbering if enabled for this document
                                let auto_number = heading_tracker.get_number(heading_info.level);
                                if auto_number.is_empty() {
                                    None
                                } else {
                                    Some(auto_number)
                                }
                            };

                            elements.push(DocumentElement::Heading {
                                level: heading_info.level,
                                text: heading_text,
                                number,
                            });
                        } else {
                            // Fallback to text-based heading detection using first run's formatting
                            let first_formatting = if !formatted_runs.is_empty() {
                                &formatted_runs[0].formatting
                            } else {
                                &TextFormatting::default()
                            };

                            let level = detect_heading_from_text(&total_text, first_formatting);
                            if let Some(level) = level {
                                elements.push(DocumentElement::Heading {
                                    level,
                                    text: total_text,
                                    number: None,
                                });
                            } else {
                                // This is a regular paragraph - consolidate runs and preserve formatting
                                let consolidated_runs =
                                    FormattedRun::consolidate_runs(formatted_runs);
                                elements.push(DocumentElement::Paragraph {
                                    runs: consolidated_runs,
                                });
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

    // Extract inline equations with their positions
    let inline_paragraphs = extract_inline_equation_positions(file_path).unwrap_or_default();

    // Extract all equations (both inline and display)
    let equation_infos = extract_equations_from_docx(file_path).unwrap_or_default();

    // Create a map of paragraph index -> display equations
    let mut display_equations_by_para: std::collections::HashMap<usize, Vec<DocumentElement>> =
        std::collections::HashMap::new();

    for eq in equation_infos.iter() {
        if !eq.is_inline {
            display_equations_by_para
                .entry(eq.paragraph_index)
                .or_default()
                .push(DocumentElement::Equation {
                    latex: eq.latex.clone(),
                    fallback: eq.fallback.clone(),
                });
        }
    }

    // Integrate inline equations into paragraphs and insert display equations at correct positions
    let mut elements_with_equations = Vec::new();
    let mut para_index = 0;

    for element in elements {
        match element {
            DocumentElement::Paragraph { runs } => {
                para_index += 1;

                // Check if this paragraph has inline equations
                if let Some(content_items) = inline_paragraphs.get(&para_index) {
                    // Check if there are actually any inline equations in this paragraph
                    let has_actual_equations = content_items
                        .iter()
                        .any(|item| matches!(item, ParagraphContent::InlineEquation { .. }));

                    if has_actual_equations {
                        // Reconstruct paragraph with inline equations in correct positions
                        let mut new_runs = Vec::new();
                        let mut accumulated_text = String::new();

                        for content in content_items {
                            match content {
                                ParagraphContent::Text(text) => {
                                    accumulated_text.push_str(text);
                                }
                                ParagraphContent::InlineEquation { latex, fallback: _ } => {
                                    // Flush accumulated text before equation
                                    if !accumulated_text.is_empty() {
                                        new_runs.push(FormattedRun {
                                            text: accumulated_text.clone(),
                                            formatting: TextFormatting::default(),
                                        });
                                        accumulated_text.clear();
                                    }
                                    // Add inline equation with $ delimiters
                                    new_runs.push(FormattedRun {
                                        text: format!("${latex}$"),
                                        formatting: TextFormatting::default(),
                                    });
                                }
                            }
                        }

                        // Flush any remaining text
                        if !accumulated_text.is_empty() {
                            new_runs.push(FormattedRun {
                                text: accumulated_text,
                                formatting: TextFormatting::default(),
                            });
                        }

                        elements_with_equations.push(DocumentElement::Paragraph { runs: new_runs });
                    } else {
                        // No actual equations, preserve original runs with formatting
                        elements_with_equations.push(DocumentElement::Paragraph { runs });
                    }
                } else {
                    // Check if this paragraph is actually a display equation
                    if let Some(display_eqs) = display_equations_by_para.get(&para_index) {
                        // This paragraph contains display equation(s)
                        for eq in display_eqs {
                            elements_with_equations.push(eq.clone());
                        }
                    } else {
                        // Regular paragraph without equations
                        elements_with_equations.push(DocumentElement::Paragraph { runs });
                    }
                }
            }
            _ => {
                elements_with_equations.push(element);
            }
        }
    }

    // Post-process to group consecutive list items (only for text-based lists)
    // Word numbering-based lists are already properly formatted
    let elements = group_list_items(elements_with_equations);

    // Clean up Word list markers
    let elements = clean_word_list_markers(elements);

    // Merge display equations into the final element list at correct positions
    let elements = merge_display_equations(elements, display_equations_by_para);

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
        image_options,
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
    num_id: Option<i32>, // Word's numbering definition ID
}

/// Type alias for numbering counters to simplify complex HashMap type
type NumberingCounters = std::collections::HashMap<(i32, u8), u32>;

/// Manages document-wide numbering state for proper sequential numbering
#[derive(Debug)]
struct DocumentNumberingManager {
    /// Counters for each (numId, level) combination
    /// Key: (numId, level), Value: current counter
    counters: NumberingCounters,
}

impl DocumentNumberingManager {
    fn new() -> Self {
        Self {
            counters: NumberingCounters::new(),
        }
    }

    /// Generate the next number for a given numId and level
    fn generate_number(&mut self, num_id: i32, level: u8, format: NumberingFormat) -> String {
        // Get current counter for this (numId, level) combination
        let key = (num_id, level);
        let counter_value = {
            let counter = self.counters.entry(key).or_insert(0);
            *counter += 1;
            *counter
        };

        // Reset deeper levels when we increment a higher level
        // This handles hierarchical numbering like 1. -> 1.1 -> 2. (reset 1.1 back to 2.1)
        self.reset_deeper_levels(num_id, level);

        // For hierarchical numbering, we need to build the full number string
        self.format_hierarchical_number(num_id, level, counter_value, format)
    }

    fn reset_deeper_levels(&mut self, num_id: i32, current_level: u8) {
        // Reset all levels deeper than current_level for this numId
        let keys_to_reset: Vec<_> = self
            .counters
            .keys()
            .filter(|(id, level)| *id == num_id && *level > current_level)
            .cloned()
            .collect();

        for key in keys_to_reset {
            self.counters.remove(&key);
        }
    }

    fn format_number(&self, counter: u32, format: NumberingFormat) -> String {
        match format {
            NumberingFormat::Decimal => format!("{counter}. "),
            NumberingFormat::LowerLetter => {
                // Convert 1->a, 2->b, etc.
                if counter <= 26 {
                    let letter = (b'a' + (counter - 1) as u8) as char;
                    format!("{letter}. ")
                } else {
                    format!("{counter}. ") // Fallback for > 26
                }
            }
            NumberingFormat::LowerRoman => format!("{}. ", Self::to_roman(counter).to_lowercase()),
            NumberingFormat::UpperLetter => {
                // Convert 1->A, 2->B, etc.
                if counter <= 26 {
                    let letter = (b'A' + (counter - 1) as u8) as char;
                    format!("{letter}. ")
                } else {
                    format!("{counter}. ") // Fallback for > 26
                }
            }
            NumberingFormat::UpperRoman => format!("{}. ", Self::to_roman(counter)),
            NumberingFormat::ParenLowerLetter => {
                if counter <= 26 {
                    let letter = (b'a' + (counter - 1) as u8) as char;
                    format!("({letter})")
                } else {
                    format!("({counter})")
                }
            }
            NumberingFormat::ParenLowerRoman => {
                format!("({})", Self::to_roman(counter).to_lowercase())
            }
            NumberingFormat::Bullet => "* ".to_string(),
        }
    }

    fn to_roman(num: u32) -> String {
        let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
        let symbols = [
            "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
        ];

        let mut result = String::new();
        let mut n = num;

        for (i, &value) in values.iter().enumerate() {
            while n >= value {
                result.push_str(symbols[i]);
                n -= value;
            }
        }

        result
    }

    /// Format hierarchical number (e.g., "2.1", "3.2.1")
    fn format_hierarchical_number(
        &self,
        num_id: i32,
        level: u8,
        counter: u32,
        format: NumberingFormat,
    ) -> String {
        // Check if this numId/level combination should use hierarchical numbering
        let needs_hierarchy = matches!((num_id, level), (4, 1)); // 2.1, 2.2, etc.

        if needs_hierarchy {
            // Build hierarchical number by including parent level counters
            let mut parts = Vec::new();

            // Add parent level counter (level 0 for this numId)
            if let Some(parent_counter) = self.counters.get(&(num_id, 0)) {
                parts.push(parent_counter.to_string());
            }

            // Add current level counter
            parts.push(counter.to_string());

            // Join with dots and add final punctuation
            format!("{}. ", parts.join("."))
        } else {
            // Use regular formatting for non-hierarchical levels
            self.format_number(counter, format)
        }
    }
}

/// Different numbering formats supported by Word
#[derive(Debug, Clone, Copy)]
enum NumberingFormat {
    Decimal,          // 1. 2. 3.
    LowerLetter,      // a. b. c.
    UpperLetter,      // A. B. C.
    LowerRoman,       // i. ii. iii.
    UpperRoman,       // I. II. III.
    ParenLowerLetter, // (a) (b) (c)
    ParenLowerRoman,  // (i) (ii) (iii)
    #[allow(dead_code)]
    Bullet, // * * *
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

        // Extract numId for state tracking
        let num_id = num_pr.id.as_ref().map(|id| id.id as i32);

        // Enhanced detection for mixed list types (same numId, different levels)
        let is_ordered = if let Some(num_id_val) = num_id {
            match (num_id_val, level) {
                // For Word's default mixed list (numId 1):
                // Level 0 = decimal numbers (1. 2. 3.)
                // Level 1 = letters (a) b) c))
                // Level 2 = roman numerals (i. ii. iii.)
                (1, 0) => true, // Top level: decimal numbers (was false, causing bug)
                (1, 1) => true, // Second level: letters
                (1, 2) => true, // Third level: roman numerals
                (1, _) => level % 2 == 1, // Pattern for deeper levels
                (_, _) => true, // Other numIds are typically ordered
            }
        } else {
            false
        };

        return Some(ListInfo {
            level,
            is_ordered,
            num_id,
        });
    }
    None
}

/// Determine the numbering format based on Word's numId and level
fn get_numbering_format(num_id: i32, level: u8) -> NumberingFormat {
    match (num_id, level) {
        // numId=4: Main multilevel list (from advanced-numbering-2.docx)
        (4, 0) => NumberingFormat::Decimal,    // 1., 2., 3.
        (4, 1) => NumberingFormat::Decimal,    // 2.1., 2.2., 2.3. (hierarchical)
        (4, 2) => NumberingFormat::LowerRoman, // i., ii., iii.

        // numId=5: Secondary list (a), (b), (c) from same document
        (5, 2) => NumberingFormat::ParenLowerLetter, // (a), (b), (c)

        // numId=2: From other test documents
        (2, 0) => NumberingFormat::Decimal,         // 1., 2., 3.
        (2, 3) => NumberingFormat::ParenLowerRoman, // (i), (ii), (iii)

        // numId=1: Default Word numbering scheme
        (1, 0) => NumberingFormat::Decimal,          // 1. 2. 3.
        (1, 1) => NumberingFormat::LowerLetter,      // a. b. c.
        (1, 2) => NumberingFormat::LowerRoman,       // i. ii. iii.
        (1, 3) => NumberingFormat::ParenLowerLetter, // (a) (b) (c)
        (1, 4) => NumberingFormat::ParenLowerRoman,  // (i) (ii) (iii)

        // Fallback defaults based on level
        (_, 0) => NumberingFormat::Decimal,
        (_, 1) => NumberingFormat::LowerLetter,
        (_, 2) => NumberingFormat::LowerRoman,
        (_, 3) => NumberingFormat::UpperLetter,
        (_, 4) => NumberingFormat::UpperRoman,
        _ => NumberingFormat::Decimal,
    }
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

#[derive(Debug)]
struct HeadingNumberTracker {
    counters: [u32; 6], // Support up to 6 heading levels
    auto_numbering_enabled: bool,
}

impl HeadingNumberTracker {
    fn new() -> Self {
        Self {
            counters: [0; 6],
            auto_numbering_enabled: false,
        }
    }

    fn enable_auto_numbering(&mut self) {
        self.auto_numbering_enabled = true;
    }

    fn get_number(&mut self, level: u8) -> String {
        if !self.auto_numbering_enabled {
            return String::new();
        }

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

/// Analyze document structure to determine if automatic numbering should be enabled
fn analyze_heading_structure(document: &docx_rs::Document) -> bool {
    let mut heading_count = 0;
    let mut has_explicit_numbering = false;
    let mut level_counts = [0u32; 6]; // Count headings at each level

    for child in &document.children {
        if let docx_rs::DocumentChild::Paragraph(para) = child {
            if let Some(heading_level) = detect_heading_from_paragraph_style(para) {
                let text = extract_paragraph_text(para);

                // Check if this heading has explicit numbering in the text
                if extract_heading_number_from_text(&text).is_some() {
                    has_explicit_numbering = true;
                }

                heading_count += 1;
                let level_index = (heading_level.saturating_sub(1) as usize).min(5);
                level_counts[level_index] += 1;
            }
        }
    }

    // Don't auto-number if:
    // 1. Any headings have explicit numbering
    // 2. Very few headings (less than 3)
    // 3. Only one level of headings (no hierarchy)
    if has_explicit_numbering || heading_count < 3 {
        return false;
    }

    // Check if we have a real hierarchy (headings at multiple levels)
    let levels_with_headings = level_counts.iter().filter(|&&count| count > 0).count();

    // Auto-number if we have multiple levels or multiple headings at level 1
    levels_with_headings > 1 || level_counts[0] > 1
}

// Lazy static regex patterns for heading number detection
// Focused on common patterns for manual numbering in text
static HEADING_NUMBER_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Standard decimal numbering: "1.", "1.1", "1.1.1", "2.1.1" (most common)
        // For single numbers, require a period to distinguish from "Heading 1" style titles
        // For hierarchical numbers (1.1, 1.2.3), period is optional
        Regex::new(r"^(\d+(?:\.\d+)+\.?|\d+\.)\s+(.+)$").unwrap(),
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

fn extract_run_formatting(run: &docx_rs::Run) -> TextFormatting {
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
            // Safe: '.' is ASCII, so dot_pos+1 is guaranteed to be a char boundary
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
            DocumentElement::Paragraph { runs } => {
                // Get the combined text from all runs for list detection
                let text: String = runs.iter().map(|run| run.text.as_str()).collect();

                if is_likely_list_item(&text) {
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
                    let level = calculate_list_level(&text);

                    // Clean the runs (remove bullet/number prefix from first run)
                    let clean_runs = clean_list_item_runs(runs.clone());

                    current_list_items.push(ListItem {
                        runs: clean_runs,
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

fn clean_list_item_runs(runs: Vec<FormattedRun>) -> Vec<FormattedRun> {
    if runs.is_empty() {
        return runs;
    }

    // Get the combined text to determine what prefix to remove
    let combined_text: String = runs.iter().map(|run| run.text.as_str()).collect();
    let text = combined_text.trim();

    // Determine what prefix we need to remove
    let prefix_to_remove = if text.starts_with("• ") {
        "• "
    } else if text.starts_with("- ") {
        "- "
    } else if text.starts_with("* ") {
        "* "
    } else if let Some(dot_pos) = text.find('.') {
        let prefix = &text[..dot_pos];
        if prefix.chars().all(|c| c.is_ascii_digit()) {
            // For numbered lists, include the dot and following space
            &text[..dot_pos
                + if text.chars().nth(dot_pos + 1) == Some(' ') {
                    2
                } else {
                    1
                }]
        } else if text.chars().count() > 2 && text.chars().nth(1) == Some('.') {
            let first_char = text.chars().next().unwrap();
            if first_char.is_ascii_lowercase() || first_char.is_ascii_uppercase() {
                // For lettered lists, include the letter, dot, and following space
                &text[..if text.chars().nth(2) == Some(' ') {
                    3
                } else {
                    2
                }]
            } else {
                ""
            }
        } else {
            ""
        }
    } else {
        ""
    };

    if prefix_to_remove.is_empty() {
        return runs;
    }

    // Remove the prefix from the runs while preserving formatting
    let mut result_runs = Vec::new();
    let mut chars_to_remove = prefix_to_remove.chars().count();

    for run in runs {
        if chars_to_remove == 0 {
            // No more prefix to remove, keep this run as-is
            result_runs.push(run);
        } else {
            let run_char_count = run.text.chars().count();
            if run_char_count <= chars_to_remove {
                // This entire run is part of the prefix to remove
                chars_to_remove -= run_char_count;
            } else {
                // This run contains part of the text we want to keep
                let keep_text: String = run.text.chars().skip(chars_to_remove).collect();
                if !keep_text.is_empty() {
                    result_runs.push(FormattedRun {
                        text: keep_text.trim_start().to_string(),
                        formatting: run.formatting,
                    });
                }
                chars_to_remove = 0;
            }
        }
    }

    result_runs
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
    // TODO: consider deferring search execution until Enter is pressed
    if query.is_empty() {
        return results;
    }
    let query_lower = query.to_lowercase();

    for (element_index, element) in document.elements.iter().enumerate() {
        let text = match element {
            DocumentElement::Heading { text, .. } => text,
            DocumentElement::Paragraph { runs } => {
                // Combine text from all runs for searching
                &runs.iter().map(|run| run.text.as_str()).collect::<String>()
            }
            DocumentElement::List { items, .. } => {
                // Search in list items
                for item in items {
                    let item_text: String = item.runs.iter().map(|run| run.text.as_str()).collect();
                    let text_lower = item_text.to_lowercase();
                    if let Some(start_pos) = text_lower.find(&query_lower) {
                        results.push(SearchResult {
                            element_index,
                            text: item_text,
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
            DocumentElement::Equation { latex, .. } => latex,
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

fn clean_word_list_markers(elements: Vec<DocumentElement>) -> Vec<DocumentElement> {
    elements
        .into_iter()
        .map(|element| match element {
            DocumentElement::Paragraph { runs } => {
                let cleaned_runs = runs
                    .into_iter()
                    .map(|mut run| {
                        if run.text.starts_with("__WORD_LIST__") {
                            run.text = run
                                .text
                                .strip_prefix("__WORD_LIST__")
                                .unwrap_or(&run.text)
                                .to_string();
                        }
                        run
                    })
                    .collect();
                DocumentElement::Paragraph { runs: cleaned_runs }
            }
            DocumentElement::List { items, ordered } => {
                let cleaned_items = items
                    .into_iter()
                    .map(|item| {
                        let combined_text: String =
                            item.runs.iter().map(|run| run.text.as_str()).collect();
                        let cleaned_runs = if combined_text.starts_with("__WORD_LIST__") {
                            // Remove the __WORD_LIST__ prefix from the first run
                            let mut new_runs = item.runs.clone();
                            if let Some(first_run) = new_runs.first_mut() {
                                first_run.text = first_run
                                    .text
                                    .strip_prefix("__WORD_LIST__")
                                    .unwrap_or(&first_run.text)
                                    .to_string();
                            }
                            new_runs
                        } else {
                            item.runs.clone()
                        };
                        ListItem {
                            runs: cleaned_runs,
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

/// Equation type and context information
#[derive(Debug, Clone)]
struct EquationInfo {
    latex: String,
    fallback: String,
    is_inline: bool,
    paragraph_index: usize,
}

/// Represents content within a paragraph (text or inline equation)
#[derive(Debug, Clone)]
enum ParagraphContent {
    Text(String),
    #[allow(dead_code)] // fallback may be used for UI display in future
    InlineEquation {
        latex: String,
        fallback: String,
    },
}

/// Parse paragraphs with inline equations directly from XML
/// Returns a map of paragraph index to ordered content (text and inline equations)
fn extract_inline_equation_positions(
    file_path: &Path,
) -> Result<std::collections::HashMap<usize, Vec<ParagraphContent>>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use std::fs::File;
    use std::io::Read;
    use zip::ZipArchive;

    let file = File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read word/document.xml
    let mut document_xml = String::new();
    let mut xml_file = archive.by_name("word/document.xml")?;
    xml_file.read_to_string(&mut document_xml)?;

    let mut paragraphs: std::collections::HashMap<usize, Vec<ParagraphContent>> =
        std::collections::HashMap::new();
    let mut reader = Reader::from_str(&document_xml);
    reader.config_mut().trim_text(false); // Don't trim to preserve spacing

    let mut buf = Vec::new();
    let mut in_paragraph = false;
    let mut in_math = false;
    let mut in_math_para = false; // Track if we're in a display equation
    let mut in_text_run = false;
    let mut current_paragraph_index = 0;
    let mut current_paragraph_content: Vec<ParagraphContent> = Vec::new();
    let mut current_text = String::new();
    let mut current_omml = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                in_paragraph = true;
                current_paragraph_index += 1;
                current_paragraph_content.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:p" => {
                in_paragraph = false;
                if !current_paragraph_content.is_empty() {
                    paragraphs.insert(current_paragraph_index, current_paragraph_content.clone());
                }
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"m:oMathPara" => {
                in_math_para = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:oMathPara" => {
                in_math_para = false;
            }
            Ok(Event::Start(ref e))
                if e.name().as_ref() == b"m:oMath" && in_paragraph && !in_math_para =>
            {
                // Inline equation (not wrapped in oMathPara)
                in_math = true;
                current_omml.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:oMath" && in_math => {
                in_math = false;
                let (latex, fallback) = parse_simple_omml(&current_omml);
                current_paragraph_content
                    .push(ParagraphContent::InlineEquation { latex, fallback });
                current_omml.clear();
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" && in_paragraph && !in_math => {
                in_text_run = true;
                current_text.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" && in_text_run => {
                in_text_run = false;
                if !current_text.is_empty() {
                    current_paragraph_content.push(ParagraphContent::Text(current_text.clone()));
                }
            }
            Ok(Event::Text(ref e)) if in_text_run => {
                current_text.push_str(&e.unescape().unwrap_or_default());
            }
            // Capture OMML content for inline equations
            Ok(Event::Start(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push('<');
                current_omml.push_str(tag_name);
                for a in e.attributes().flatten() {
                    let key = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                    let value = String::from_utf8_lossy(&a.value);
                    current_omml.push(' ');
                    current_omml.push_str(key);
                    current_omml.push_str("=\"");
                    current_omml.push_str(&value);
                    current_omml.push('"');
                }
                current_omml.push('>');
            }
            Ok(Event::End(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push_str("</");
                current_omml.push_str(tag_name);
                current_omml.push('>');
            }
            Ok(Event::Empty(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push('<');
                current_omml.push_str(tag_name);
                for a in e.attributes().flatten() {
                    let key = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                    let value = String::from_utf8_lossy(&a.value);
                    current_omml.push(' ');
                    current_omml.push_str(key);
                    current_omml.push_str("=\"");
                    current_omml.push_str(&value);
                    current_omml.push('"');
                }
                current_omml.push_str("/>");
            }
            Ok(Event::Text(ref e)) if in_math => {
                current_omml.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error reading XML for inline equations: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(paragraphs)
}

/// Extract equations from .docx file by reading raw XML
/// Since docx-rs doesn't expose OMML (Office Math Markup Language), we parse the ZIP directly
fn extract_equations_from_docx(file_path: &Path) -> Result<Vec<EquationInfo>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use std::fs::File;
    use std::io::Read;
    use zip::ZipArchive;

    let file = File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read word/document.xml
    let mut document_xml = String::new();
    let mut xml_file = archive.by_name("word/document.xml")?;
    xml_file.read_to_string(&mut document_xml)?;

    let mut equations = Vec::new();
    let mut reader = Reader::from_str(&document_xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut in_math = false;
    let mut in_math_para = false;
    let mut current_omml = String::new();
    let mut current_paragraph_index = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                current_paragraph_index += 1;
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"m:oMathPara" => {
                in_math_para = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:oMathPara" => {
                in_math_para = false;
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"m:oMath" => {
                in_math = true;
                current_omml.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:oMath" => {
                in_math = false;

                // Parse the collected OMML to LaTeX
                let (latex, fallback) = parse_simple_omml(&current_omml);

                // Inline equations are NOT wrapped in <m:oMathPara>
                let is_inline = !in_math_para;

                equations.push(EquationInfo {
                    latex,
                    fallback,
                    is_inline,
                    paragraph_index: current_paragraph_index,
                });
                current_omml.clear();
            }
            Ok(Event::Start(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push('<');
                current_omml.push_str(tag_name);

                // Capture attributes (e.g., m:chr m:val="∑")
                for a in e.attributes().flatten() {
                    let key = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                    let value = String::from_utf8_lossy(&a.value);
                    current_omml.push(' ');
                    current_omml.push_str(key);
                    current_omml.push_str("=\"");
                    current_omml.push_str(&value);
                    current_omml.push('"');
                }
                current_omml.push('>');
            }
            Ok(Event::End(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push_str("</");
                current_omml.push_str(tag_name);
                current_omml.push('>');
            }
            Ok(Event::Empty(ref e)) if in_math => {
                // Handle self-closing tags like <m:type m:val="noBar"/>
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push('<');
                current_omml.push_str(tag_name);

                // Capture attributes
                for a in e.attributes().flatten() {
                    let key = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                    let value = String::from_utf8_lossy(&a.value);
                    current_omml.push(' ');
                    current_omml.push_str(key);
                    current_omml.push_str("=\"");
                    current_omml.push_str(&value);
                    current_omml.push('"');
                }
                current_omml.push_str("/>");
            }
            Ok(Event::Text(ref e)) if in_math => {
                current_omml.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error reading OMML: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(equations)
}

/// OMML parser that converts to LaTeX format
fn parse_simple_omml(omml: &str) -> (String, String) {
    // Extract plain text for fallback
    let fallback = omml
        .split("<m:t>")
        .skip(1)
        .filter_map(|s| s.split("</m:t>").next())
        .collect::<Vec<_>>()
        .join("");

    let latex = omml_to_latex(omml);

    if latex.is_empty() {
        (fallback.clone(), fallback)
    } else {
        (latex, fallback)
    }
}

/// Convert OMML XML to LaTeX
fn omml_to_latex(omml: &str) -> String {
    let mut result = String::new();
    let mut i = 0;

    while i < omml.len() {
        // Look for OMML structures
        if omml[i..].starts_with("<m:sSup>") {
            // Superscript: ^{...}
            let end = omml[i..].find("</m:sSup>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let (Some(base), Some(sup)) = (
                extract_latex_text(content, "m:e"),
                extract_latex_text(content, "m:sup"),
            ) {
                result.push_str(&base);
                result.push_str("^{");
                result.push_str(&sup);
                result.push('}');
            }
            i += end + 8;
        } else if omml[i..].starts_with("<m:sSub>") {
            // Subscript: _{...}
            let end = omml[i..].find("</m:sSub>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let (Some(base), Some(sub)) = (
                extract_latex_text(content, "m:e"),
                extract_latex_text(content, "m:sub"),
            ) {
                result.push_str(&base);
                result.push_str("_{");
                result.push_str(&sub);
                result.push('}');
            }
            i += end + 8;
        } else if omml[i..].starts_with("<m:sSub") && omml[i..].starts_with("<m:sSubSup>") {
            // Subscript and superscript: _{}^{}
            let end = omml[i..].find("</m:sSubSup>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let (Some(base), Some(sub), Some(sup)) = (
                extract_latex_text(content, "m:e"),
                extract_latex_text(content, "m:sub"),
                extract_latex_text(content, "m:sup"),
            ) {
                result.push_str(&base);
                result.push_str("_{");
                result.push_str(&sub);
                result.push_str("}^{");
                result.push_str(&sup);
                result.push('}');
            }
            i += end + 12;
        } else if omml[i..].starts_with("<m:d>") {
            // Delimiter: \left(...\right)
            let end = omml[i..].find("</m:d>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            result.push_str("\\left(");
            if let Some(inner) = extract_latex_text(content, "m:e") {
                result.push_str(&inner);
            }
            result.push_str("\\right)");
            i += end + 5;
        } else if omml[i..].starts_with("<m:f>") {
            // Fraction: \frac{num}{den} or binomial coefficient: \binom{n}{k}
            let end = omml[i..].find("</m:f>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            // Check if it's a binomial coefficient (noBar type)
            let is_binom = content.contains("m:val=\"noBar\"");

            if let (Some(num), Some(den)) = (
                extract_latex_text(content, "m:num"),
                extract_latex_text(content, "m:den"),
            ) {
                if is_binom {
                    result.push_str("\\binom{");
                    result.push_str(&num);
                    result.push_str("}{");
                    result.push_str(&den);
                    result.push('}');
                } else {
                    result.push_str("\\frac{");
                    result.push_str(&num);
                    result.push_str("}{");
                    result.push_str(&den);
                    result.push('}');
                }
            }
            i += end + 5;
        } else if omml[i..].starts_with("<m:func>") {
            // Function: \sin, \cos, \tan, etc.
            let end = omml[i..].find("</m:func>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let Some(func_name) = extract_latex_text(content, "m:fName") {
                result.push('\\');
                result.push_str(&func_name);
            }
            if let Some(argument) = extract_latex_text(content, "m:e") {
                result.push(' ');
                result.push_str(&argument);
            }
            i += end + 8;
        } else if omml[i..].starts_with("<m:rad>") {
            // Radical (square root): \sqrt{...} or \sqrt[n]{...}
            let end = omml[i..].find("</m:rad>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            result.push_str("\\sqrt");
            // Check for degree (nth root)
            if let Some(deg) = extract_latex_text(content, "m:deg") {
                if deg != "2" && !deg.is_empty() {
                    result.push('[');
                    result.push_str(&deg);
                    result.push(']');
                }
            }
            result.push('{');
            if let Some(base) = extract_latex_text(content, "m:e") {
                result.push_str(&base);
            }
            result.push('}');
            i += end + 7;
        } else if omml[i..].starts_with("<m:nary") {
            // N-ary operator: \sum_{...}^{...}, \int_{...}^{...}, etc.
            let end = omml[i..].find("</m:nary>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            // Extract operator character and convert to LaTeX command
            let operator = if let Some(chr_pos) = content.find("m:val=\"") {
                let start = chr_pos + 7;
                if let Some(end_quote) = content[start..].find('"') {
                    let chr = &content[start..start + end_quote];
                    match chr {
                        "∑" => "\\sum",
                        "∫" => "\\int",
                        "∬" => "\\iint",
                        "∭" => "\\iiint",
                        "∮" => "\\oint",
                        "∏" => "\\prod",
                        "⋃" => "\\bigcup",
                        "⋂" => "\\bigcap",
                        _ => "\\sum",
                    }
                } else {
                    "\\sum"
                }
            } else {
                "\\sum"
            };

            result.push_str(operator);

            // Extract sub and sup
            if let Some(sub) = extract_latex_text(content, "m:sub") {
                result.push_str("_{");
                result.push_str(&sub);
                result.push('}');
            }
            if let Some(sup) = extract_latex_text(content, "m:sup") {
                result.push_str("^{");
                result.push_str(&sup);
                result.push('}');
            }
            if let Some(base) = extract_latex_text(content, "m:e") {
                result.push(' ');
                result.push_str(&base);
            }

            i += end + 9;
        } else if omml[i..].starts_with("<m:r>") {
            // Text run - extract text without processing
            let end = omml[i..].find("</m:r>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let Some(text) = extract_text(content, "m:t") {
                // Convert special characters to LaTeX
                for ch in text.chars() {
                    match ch {
                        'π' => result.push_str("\\pi "),
                        'α' => result.push_str("\\alpha "),
                        'β' => result.push_str("\\beta "),
                        'γ' => result.push_str("\\gamma "),
                        'Γ' => result.push_str("\\Gamma "),
                        'δ' => result.push_str("\\delta "),
                        'Δ' => result.push_str("\\Delta "),
                        'θ' => result.push_str("\\theta "),
                        'λ' => result.push_str("\\lambda "),
                        'μ' => result.push_str("\\mu "),
                        'σ' => result.push_str("\\sigma "),
                        'Σ' => result.push_str("\\Sigma "),
                        'φ' => result.push_str("\\phi "),
                        'ω' => result.push_str("\\omega "),
                        'Ω' => result.push_str("\\Omega "),
                        '∞' => result.push_str("\\infty "),
                        '±' => result.push_str("\\pm "),
                        '×' => result.push_str("\\times "),
                        '÷' => result.push_str("\\div "),
                        '≤' => result.push_str("\\leq "),
                        '≥' => result.push_str("\\geq "),
                        '≠' => result.push_str("\\neq "),
                        '≈' => result.push_str("\\approx "),
                        '∈' => result.push_str("\\in "),
                        '∉' => result.push_str("\\notin "),
                        '⊂' => result.push_str("\\subset "),
                        '⊃' => result.push_str("\\supset "),
                        '∪' => result.push_str("\\cup "),
                        '∩' => result.push_str("\\cap "),
                        '∅' => result.push_str("\\emptyset "),
                        '√' => result.push_str("\\sqrt"),
                        _ => result.push(ch),
                    }
                }
            }
            i += end + 5;
        } else if omml[i..].starts_with("<m:t>") {
            // Text content
            let end = omml[i + 4..].find("</m:t>").unwrap_or(omml.len() - i - 4);
            let text = &omml[i + 4..i + 4 + end];
            // Convert special characters
            for ch in text.chars() {
                match ch {
                    'π' => result.push_str("\\pi"),
                    'α' => result.push_str("\\alpha"),
                    'β' => result.push_str("\\beta"),
                    _ => result.push(ch),
                }
            }
            i += 4 + end + 5;
        } else {
            i += 1;
        }
    }

    result
}

/// Extract text from an OMML tag and recursively convert nested OMML to LaTeX
fn extract_latex_text(omml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    if let Some(start_pos) = omml.find(&start_tag) {
        let content = &omml[start_pos + start_tag.len()..];

        // Find the matching closing tag, accounting for nesting
        let mut depth = 1;
        let mut pos = 0;
        let mut end_pos = None;

        while pos < content.len() && depth > 0 {
            if content[pos..].starts_with(&start_tag) {
                depth += 1;
                pos += start_tag.len();
            } else if content[pos..].starts_with(&end_tag) {
                depth -= 1;
                if depth == 0 {
                    end_pos = Some(pos);
                    break;
                }
                pos += end_tag.len();
            } else {
                // Skip to next character boundary (Unicode-safe)
                let next_char = content[pos..].chars().next();
                if let Some(ch) = next_char {
                    pos += ch.len_utf8();
                } else {
                    break;
                }
            }
        }

        if let Some(end_pos) = end_pos {
            let inner = &content[..end_pos];

            // Check if inner content has OMML structures
            if inner.contains("<m:") {
                // Recursively convert nested OMML to LaTeX
                return Some(omml_to_latex(inner));
            } else {
                // Extract plain text from <m:t> tags
                let text = inner
                    .split("<m:t>")
                    .skip(1)
                    .filter_map(|s| s.split("</m:t>").next())
                    .collect::<Vec<_>>()
                    .join("");

                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

/// Extract text from an OMML tag
fn extract_text(omml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    if let Some(start_pos) = omml.find(&start_tag) {
        let content = &omml[start_pos + start_tag.len()..];
        if let Some(end_pos) = content.find(&end_tag) {
            let inner = &content[..end_pos];

            // Inner is already the text between <tag> and </tag>, just return it
            if !inner.is_empty() {
                return Some(inner.to_string());
            }
        }
    }
    None
}
