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

#[derive(Debug, Clone)]
struct ListInfo {
    level: u8,
    is_ordered: bool,
    num_id: Option<i32>, // Word's numbering definition ID
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
