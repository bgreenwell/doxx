use anyhow::Result;

use crate::{document::*, ExportFormat};

pub fn export_document(document: &Document, format: &ExportFormat) -> Result<()> {
    match format {
        ExportFormat::Markdown => export_to_markdown(document),
        ExportFormat::Text => export_to_text(document),
        ExportFormat::Csv => export_to_csv(document),
        ExportFormat::Json => export_to_json(document),
    }
}

pub fn export_to_markdown(document: &Document) -> Result<()> {
    let mut markdown = String::new();

    // Add document title
    markdown.push_str(&format!("# {}\n\n", document.title));

    // Add metadata
    markdown.push_str("## Document Information\n\n");
    markdown.push_str(&format!("- **File**: {}\n", document.metadata.file_path));
    markdown.push_str(&format!("- **Pages**: {}\n", document.metadata.page_count));
    markdown.push_str(&format!("- **Words**: {}\n", document.metadata.word_count));
    if let Some(author) = &document.metadata.author {
        markdown.push_str(&format!("- **Author**: {author}\n"));
    }
    markdown.push_str("\n---\n\n");

    // Convert document content
    for element in &document.elements {
        match element {
            DocumentElement::Heading {
                level,
                text,
                number,
            } => {
                let prefix = "#".repeat(*level as usize + 1); // +1 because title is h1
                let heading_text = if let Some(number) = number {
                    format!("{number} {text}")
                } else {
                    text.clone()
                };
                markdown.push_str(&format!("{prefix} {heading_text}\n\n"));
            }
            DocumentElement::Paragraph { text, formatting } => {
                let mut formatted_text = text.clone();

                if formatting.bold {
                    formatted_text = format!("**{formatted_text}**");
                }
                if formatting.italic {
                    formatted_text = format!("*{formatted_text}*");
                }

                markdown.push_str(&format!("{formatted_text}\n\n"));
            }
            DocumentElement::List { items, ordered } => {
                for (i, item) in items.iter().enumerate() {
                    let indent = "  ".repeat(item.level as usize);
                    let bullet = if *ordered {
                        format!("{}. ", i + 1)
                    } else {
                        "- ".to_string()
                    };

                    let mut item_text = item.text.clone();
                    if false
                    /* simplified */
                    {
                        item_text = format!("**{item_text}**");
                    }
                    if false
                    /* simplified */
                    {
                        item_text = format!("*{item_text}*");
                    }

                    markdown.push_str(&format!("{indent}{bullet}{item_text}\n"));
                }
                markdown.push('\n');
            }
            DocumentElement::Table { table } => {
                // Add table title if present
                if let Some(title) = &table.metadata.title {
                    markdown.push_str(&format!("### {title}\n\n"));
                }

                // Markdown table header
                let header_content: Vec<String> =
                    table.headers.iter().map(|h| h.content.clone()).collect();
                markdown.push_str(&format!("| {} |\n", header_content.join(" | ")));

                // Generate alignment indicators
                let alignment_row: Vec<String> = table
                    .metadata
                    .column_alignments
                    .iter()
                    .map(|align| match align {
                        TextAlignment::Left => ":---".to_string(),
                        TextAlignment::Right => "---:".to_string(),
                        TextAlignment::Center => ":---:".to_string(),
                        TextAlignment::Justify => ":---".to_string(),
                    })
                    .collect();
                markdown.push_str(&format!("| {} |\n", alignment_row.join(" | ")));

                // Table rows
                for row in &table.rows {
                    let row_content: Vec<String> =
                        row.iter().map(|cell| cell.content.clone()).collect();
                    markdown.push_str(&format!("| {} |\n", row_content.join(" | ")));
                }
                markdown.push('\n');
            }
            DocumentElement::Image {
                description,
                width,
                height,
                image_path,
                ..
            } => {
                let alt = description;
                let url = image_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| description.clone());
                let dimensions = match (width, height) {
                    (Some(w), Some(h)) => format!(" <!-- {w}x{h} -->"),
                    _ => String::new(),
                };
                markdown.push_str(&format!("![{alt}]({url}){dimensions}\n\n"));
            }
            DocumentElement::PageBreak => {
                markdown.push_str("\n---\n\n");
            }
        }
    }

    print!("{markdown}");
    Ok(())
}

pub fn format_as_text(document: &Document) -> String {
    let mut text = String::new();

    // Add document title
    text.push_str(&format!("{}\n", document.title));
    text.push_str(&"=".repeat(document.title.len()));
    text.push_str("\n\n");

    // Convert document content
    for element in &document.elements {
        match element {
            DocumentElement::Heading {
                level,
                text: heading_text,
                ..
            } => {
                let underline = match level {
                    1 => "=",
                    2 => "-",
                    _ => "~",
                };
                text.push_str(&format!("{heading_text}\n"));
                text.push_str(&underline.repeat(heading_text.len()));
                text.push_str("\n\n");
            }
            DocumentElement::Paragraph {
                text: para_text, ..
            } => {
                text.push_str(&format!("{para_text}\n\n"));
            }
            DocumentElement::List { items, ordered } => {
                for (i, item) in items.iter().enumerate() {
                    let bullet = if *ordered {
                        format!("{}. ", i + 1)
                    } else {
                        "* ".to_string()
                    };

                    let indent = "  ".repeat(item.level as usize);
                    text.push_str(&format!("{indent}{bullet}{}\n", item.text));
                }
                text.push('\n');
            }
            DocumentElement::Table { table } => {
                // Add table title if present
                if let Some(title) = &table.metadata.title {
                    text.push_str(&format!("{title}\n"));
                    text.push_str(&"=".repeat(title.len()));
                    text.push_str("\n\n");
                }

                // Use the calculated column widths from metadata
                let col_widths = &table.metadata.column_widths;

                // Top border
                let top_border = generate_text_table_border(col_widths, "┌", "┬", "┐", "─");
                text.push_str(&format!("{top_border}\n"));

                // Header with proper alignment
                let header_line = render_text_table_row(&table.headers, col_widths, true);
                text.push_str(&format!("{header_line}\n"));

                // Header separator
                let separator = generate_text_table_border(col_widths, "├", "┼", "┤", "─");
                text.push_str(&format!("{separator}\n"));

                // Data rows
                for row in &table.rows {
                    let row_line = render_text_table_row(row, col_widths, false);
                    text.push_str(&format!("{row_line}\n"));
                }

                // Bottom border
                let bottom_border = generate_text_table_border(col_widths, "└", "┴", "┘", "─");
                text.push_str(&format!("{bottom_border}\n"));

                text.push('\n');
            }
            DocumentElement::PageBreak => {
                text.push_str("---\n\n");
            }
            DocumentElement::Image {
                description,
                image_path,
                ..
            } => {
                // Try to render the image inline if available
                if let Some(path) = image_path {
                    match crate::terminal_image::TerminalImageRenderer::with_options(
                        document.image_options.max_width,
                        document.image_options.max_height,
                        document.image_options.scale,
                    )
                    .render_image_from_path(path, description)
                    {
                        Ok(_) => {
                            // Image displayed successfully, add spacing
                            text.push('\n');
                        }
                        Err(_) => {
                            // Fallback to text description
                            text.push_str(&format!("[Image: {description}]\n\n"));
                        }
                    }
                } else {
                    text.push_str(&format!("[Image: {description}]\n\n"));
                }
            }
        }
    }

    text
}

pub fn export_to_text(document: &Document) -> Result<()> {
    export_to_text_with_images(document);
    Ok(())
}

fn export_to_text_with_images(document: &Document) {
    // Print title
    println!("{}\n", document.title);

    // Print metadata
    println!("Document Information:");
    println!("- File: {}", document.metadata.file_path);
    println!("- Pages: {}", document.metadata.page_count);
    println!("- Words: {}", document.metadata.word_count);
    if let Some(author) = &document.metadata.author {
        println!("- Author: {author}");
    }
    println!("\n{}\n", "=".repeat(50));

    // Process elements in order, printing immediately
    for element in &document.elements {
        match element {
            DocumentElement::Heading {
                level,
                text,
                number,
            } => {
                let prefix = "#".repeat(*level as usize);
                let heading_text = if let Some(number) = number {
                    format!("{number} {text}")
                } else {
                    text.clone()
                };
                println!("{prefix} {heading_text}\n");
            }
            DocumentElement::Paragraph { text, formatting } => {
                let mut formatted_text = text.clone();

                if formatting.bold {
                    formatted_text = format!("**{formatted_text}**");
                }
                if formatting.italic {
                    formatted_text = format!("*{formatted_text}*");
                }
                if formatting.underline {
                    formatted_text = format!("_{formatted_text}_");
                }

                println!("{formatted_text}\n");
            }
            DocumentElement::List { items, .. } => {
                for item in items {
                    println!("- {}", item.text);
                }
                println!();
            }
            DocumentElement::Table { table } => {
                // Simple table rendering for text export
                for row in &table.rows {
                    let row_content: Vec<String> =
                        row.iter().map(|cell| cell.content.clone()).collect();
                    println!("| {} |", row_content.join(" | "));
                }
                println!();
            }
            DocumentElement::Image {
                description,
                image_path,
                ..
            } => {
                // Render image immediately in the correct position
                if let Some(path) = image_path {
                    match crate::terminal_image::TerminalImageRenderer::with_options(
                        document.image_options.max_width,
                        document.image_options.max_height,
                        document.image_options.scale,
                    )
                    .render_image_from_path(path, description)
                    {
                        Ok(_) => {
                            // Image displayed successfully, add spacing
                            println!();
                        }
                        Err(_) => {
                            // Fallback to text description
                            println!("[Image: {description}]\n");
                        }
                    }
                } else {
                    println!("[Image: {description}]\n");
                }
            }
            DocumentElement::PageBreak => {
                println!("{}\n", "-".repeat(50));
            }
        }
    }
}

pub fn export_to_csv(document: &Document) -> Result<()> {
    let mut csv_output = Vec::new();

    // Find all tables in the document
    for (table_index, element) in document.elements.iter().enumerate() {
        if let DocumentElement::Table { table } = element {
            if table_index > 0 {
                csv_output.push(String::new()); // Empty line between tables
                csv_output.push(format!("# Table {}", table_index + 1));
            }

            // Add table title as comment if present
            if let Some(title) = &table.metadata.title {
                csv_output.push(format!("# {title}"));
            }

            // CSV header
            let header_line = table
                .headers
                .iter()
                .map(|h| escape_csv_field(&h.content))
                .collect::<Vec<_>>()
                .join(",");
            csv_output.push(header_line);

            // CSV rows
            for row in &table.rows {
                let row_line = row
                    .iter()
                    .map(|cell| escape_csv_field(&cell.content))
                    .collect::<Vec<_>>()
                    .join(",");
                csv_output.push(row_line);
            }
        }
    }

    if csv_output.is_empty() {
        println!("No tables found in document");
    } else {
        for line in csv_output {
            println!("{line}");
        }
    }

    Ok(())
}

pub fn export_to_json(document: &Document) -> Result<()> {
    let json_output = serde_json::to_string_pretty(document)?;
    println!("{json_output}");
    Ok(())
}

#[allow(dead_code)]
pub fn extract_citations(document: &Document) -> Result<Vec<Citation>> {
    let mut citations = Vec::new();

    // Simple citation extraction - look for common citation patterns
    for (index, element) in document.elements.iter().enumerate() {
        let text = match element {
            DocumentElement::Heading { text, .. } | DocumentElement::Paragraph { text, .. } => text,
            _ => continue,
        };

        // Look for citation patterns like (Author, Year) or [1]
        let citation_patterns = [
            r"\([A-Z][a-z]+,\s*\d{4}\)",             // (Author, 2024)
            r"\[[0-9]+\]",                           // [1]
            r"\([A-Z][a-z]+\s+et\s+al\.,\s*\d{4}\)", // (Author et al., 2024)
        ];

        for pattern in &citation_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for mat in regex.find_iter(text) {
                    citations.push(Citation {
                        text: mat.as_str().to_string(),
                        element_index: index,
                        citation_type: CitationType::InText,
                    });
                }
            }
        }
    }

    Ok(citations)
}

#[allow(dead_code)]
pub fn extract_bibliography(document: &Document) -> Result<Vec<Citation>> {
    let mut bibliography = Vec::new();

    // Look for bibliography or references section
    for (index, element) in document.elements.iter().enumerate() {
        if let DocumentElement::Heading { text, .. } = element {
            if text.to_lowercase().contains("reference")
                || text.to_lowercase().contains("bibliography")
                || text.to_lowercase().contains("works cited")
            {
                // Process following elements as bibliography entries
                for (bib_index, bib_element) in document.elements[index + 1..].iter().enumerate() {
                    match bib_element {
                        DocumentElement::Paragraph { text, .. } => {
                            if !text.trim().is_empty() {
                                bibliography.push(Citation {
                                    text: text.clone(),
                                    element_index: index + bib_index + 1,
                                    citation_type: CitationType::Bibliography,
                                });
                            }
                        }
                        DocumentElement::List { items, .. } => {
                            for item in items {
                                bibliography.push(Citation {
                                    text: item.text.clone(),
                                    element_index: index + bib_index + 1,
                                    citation_type: CitationType::Bibliography,
                                });
                            }
                        }
                        DocumentElement::Heading { .. } => break, // Next section
                        _ => {}
                    }
                }
                break;
            }
        }
    }

    Ok(bibliography)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Citation {
    pub text: String,
    pub element_index: usize,
    pub citation_type: CitationType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CitationType {
    InText,
    Bibliography,
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

// Helper functions for text table rendering
fn generate_text_table_border(
    column_widths: &[usize],
    left: &str,
    middle: &str,
    right: &str,
    fill: &str,
) -> String {
    let mut border = String::new();
    border.push_str(left);

    for (i, &width) in column_widths.iter().enumerate() {
        border.push_str(&fill.repeat(width + 2)); // +2 for padding
        if i < column_widths.len() - 1 {
            border.push_str(middle);
        }
    }

    border.push_str(right);
    border
}

fn render_text_table_row(cells: &[TableCell], column_widths: &[usize], _is_header: bool) -> String {
    let mut row = String::new();
    row.push('│');

    for (i, cell) in cells.iter().enumerate() {
        let width = column_widths.get(i).copied().unwrap_or(10);
        let aligned_content = align_text_cell_content(&cell.content, cell.alignment, width);

        row.push(' ');
        row.push_str(&aligned_content);
        row.push(' ');
        row.push('│');
    }

    row
}

fn align_text_cell_content(content: &str, alignment: TextAlignment, width: usize) -> String {
    let trimmed = content.trim();

    match alignment {
        TextAlignment::Left => format!("{trimmed:<width$}"),
        TextAlignment::Right => format!("{trimmed:>width$}"),
        TextAlignment::Center => {
            let padding = width.saturating_sub(trimmed.len());
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            format!(
                "{}{}{}",
                " ".repeat(left_pad),
                trimmed,
                " ".repeat(right_pad)
            )
        }
        TextAlignment::Justify => {
            // For export, treat justify as left-aligned
            format!("{trimmed:<width$}")
        }
    }
}
