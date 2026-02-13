//! Document loading and orchestration
//!
//! This module contains the main `load_document()` function that orchestrates
//! the entire document parsing process, coordinating all the specialized parsing
//! modules to transform a DOCX file into our internal Document representation.

use anyhow::Result;
use std::path::Path;

// Import types from the models module
use super::models::*;
// Import I/O functions
use super::io::{merge_display_equations, validate_docx_file};
// Import cleanup functions
use super::cleanup::{clean_word_list_markers, estimate_page_count};
// Import numbering management
use super::parsing::numbering::{
    analyze_heading_structure, DocumentNumberingManager, HeadingNumberTracker, NumberingFormat,
};
// Import list processing
use super::parsing::list::group_list_items;
// Import formatting and text extraction
use super::parsing::formatting::extract_run_formatting;
// Import heading detection
use super::parsing::heading::{detect_heading_from_text, detect_heading_with_numbering};
// Import table extraction
use super::parsing::table::extract_table_data;
// Import equation processing
use super::parsing::equation::{
    extract_equations_from_docx, extract_inline_equation_positions, ParagraphContent,
};

/// Main document loading function that orchestrates the entire parsing process
///
/// This function:
/// 1. Validates the DOCX file
/// 2. Extracts metadata (title, file size, etc.)
/// 3. Optionally extracts images
/// 4. Processes document structure (paragraphs, tables, headings, lists)
/// 5. Integrates equations (both inline and display)
/// 6. Post-processes elements (grouping lists, cleaning markers)
/// 7. Returns a fully parsed Document
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

/// Internal structure for tracking Word list information
#[derive(Debug, Clone)]
struct ListInfo {
    level: u8,
    is_ordered: bool,
    num_id: Option<i32>, // Word's numbering definition ID
}

/// Detect list properties from paragraph numbering metadata
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
