//! Table extraction and processing
//!
//! This module handles extraction of table data from Word documents,
//! including header detection, column width calculation, and data type
//! inference for proper alignment.

use super::super::models::*;
use super::formatting::extract_run_formatting;

/// Extract table data from a docx-rs Table
pub(crate) fn extract_table_data(table: &docx_rs::Table) -> Option<DocumentElement> {
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

/// Detect if a row appears to be a header based on heuristics
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

/// Calculate optimal column widths based on content
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

/// Determine column alignments based on data types
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

/// Detect the data type of a cell's content
fn detect_cell_data_type(content: &str) -> CellDataType {
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return CellDataType::Empty;
    }

    // Check for currency
    if trimmed.starts_with('$')
        || trimmed.starts_with('\u{20AC}')
        || trimmed.starts_with('\u{00A3}')
    {
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

/// Get default alignment for a data type
fn default_alignment_for_type(data_type: CellDataType) -> TextAlignment {
    match data_type {
        CellDataType::Number | CellDataType::Currency | CellDataType::Percentage => {
            TextAlignment::Right
        }
        CellDataType::Boolean => TextAlignment::Center,
        _ => TextAlignment::Left,
    }
}
