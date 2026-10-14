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

#[cfg(test)]
mod tests {
    use super::*;
    use docx_rs::{Paragraph, Run, Table, TableRow};

    fn docx_cell(text: &str) -> docx_rs::TableCell {
        docx_rs::TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(text)))
    }

    fn docx_row(cells: &[&str]) -> TableRow {
        TableRow::new(cells.iter().map(|c| docx_cell(c)).collect())
    }

    // --- detect_cell_data_type ---

    #[test]
    fn detect_cell_data_type_empty() {
        assert_eq!(detect_cell_data_type(""), CellDataType::Empty);
        assert_eq!(detect_cell_data_type("   "), CellDataType::Empty);
    }

    #[test]
    fn detect_cell_data_type_currency() {
        assert_eq!(detect_cell_data_type("$1,234.56"), CellDataType::Currency);
        assert_eq!(detect_cell_data_type("\u{20AC}99"), CellDataType::Currency); // Euro
        assert_eq!(detect_cell_data_type("\u{00A3}50"), CellDataType::Currency);
        // Pound
    }

    #[test]
    fn detect_cell_data_type_percentage() {
        assert_eq!(detect_cell_data_type("42%"), CellDataType::Percentage);
    }

    #[test]
    fn detect_cell_data_type_boolean_is_case_insensitive() {
        for v in ["true", "FALSE", "Yes", "no", "Y", "n"] {
            assert_eq!(
                detect_cell_data_type(v),
                CellDataType::Boolean,
                "input: {v}"
            );
        }
    }

    #[test]
    fn detect_cell_data_type_number_handles_commas() {
        assert_eq!(detect_cell_data_type("1234"), CellDataType::Number);
        assert_eq!(detect_cell_data_type("1,234,567.89"), CellDataType::Number);
        assert_eq!(detect_cell_data_type("-42.5"), CellDataType::Number);
    }

    #[test]
    fn detect_cell_data_type_date_requires_three_numeric_parts() {
        assert_eq!(detect_cell_data_type("2026-07-07"), CellDataType::Date);
        assert_eq!(detect_cell_data_type("07/07/2026"), CellDataType::Date);
        // Two parts only (looks like a fraction or a range, not a date).
        assert_eq!(detect_cell_data_type("10/20"), CellDataType::Text);
    }

    #[test]
    fn detect_cell_data_type_falls_back_to_text() {
        assert_eq!(detect_cell_data_type("Widget A"), CellDataType::Text);
    }

    // --- default_alignment_for_type ---

    #[test]
    fn default_alignment_for_type_matches_expected_layout() {
        assert_eq!(
            default_alignment_for_type(CellDataType::Number),
            TextAlignment::Right
        );
        assert_eq!(
            default_alignment_for_type(CellDataType::Currency),
            TextAlignment::Right
        );
        assert_eq!(
            default_alignment_for_type(CellDataType::Percentage),
            TextAlignment::Right
        );
        assert_eq!(
            default_alignment_for_type(CellDataType::Boolean),
            TextAlignment::Center
        );
        assert_eq!(
            default_alignment_for_type(CellDataType::Text),
            TextAlignment::Left
        );
        assert_eq!(
            default_alignment_for_type(CellDataType::Empty),
            TextAlignment::Left
        );
    }

    // --- TableCell::new (public constructor, exercises the two functions above together) ---

    #[test]
    fn table_cell_new_infers_type_and_alignment_together() {
        let cell = TableCell::new("$1,000".to_string());
        assert_eq!(cell.data_type, CellDataType::Currency);
        assert_eq!(cell.alignment, TextAlignment::Right);
    }

    #[test]
    fn table_cell_display_width_counts_graphemes_not_bytes() {
        // A multi-byte emoji is one grapheme, and should count as 1, not 4.
        let cell = TableCell::new("a\u{1F600}b".to_string());
        assert_eq!(cell.display_width(), 3);
    }

    // --- appears_to_be_header ---

    #[test]
    fn appears_to_be_header_recognizes_short_labeled_columns() {
        let header = vec!["Name".to_string(), "Date".to_string(), "Amount".to_string()];
        assert!(appears_to_be_header(&header));
    }

    #[test]
    fn appears_to_be_header_rejects_long_prose_rows() {
        let data_row = vec![
            "This is a very long description of a line item that goes on and on".to_string(),
            "Another lengthy piece of free-form text content in this cell".to_string(),
        ];
        assert!(!appears_to_be_header(&data_row));
    }

    // --- calculate_column_widths ---

    #[test]
    fn calculate_column_widths_uses_widest_cell_per_column_with_minimum_of_three() {
        let headers = vec![
            TableCell::new("ID".to_string()),
            TableCell::new("Name".to_string()),
        ];
        let rows = vec![
            vec![
                TableCell::new("1".to_string()),
                TableCell::new("Widget".to_string()),
            ],
            vec![
                TableCell::new("2".to_string()),
                TableCell::new("A Much Longer Product Name".to_string()),
            ],
        ];
        let widths = calculate_column_widths(&headers, &rows);
        assert_eq!(widths[0], 3); // "ID" is 2 chars, clamped up to the minimum of 3
        assert_eq!(widths[1], "A Much Longer Product Name".len());
    }

    #[test]
    fn calculate_column_widths_empty_headers_returns_empty() {
        assert_eq!(
            calculate_column_widths(&[], &Vec::new()),
            Vec::<usize>::new()
        );
    }

    // --- determine_column_alignments ---

    #[test]
    fn determine_column_alignments_right_aligns_mostly_numeric_columns() {
        let headers = vec![
            TableCell::new("Item".to_string()),
            TableCell::new("Price".to_string()),
        ];
        let rows = vec![
            vec![
                TableCell::new("Widget".to_string()),
                TableCell::new("$10".to_string()),
            ],
            vec![
                TableCell::new("Gadget".to_string()),
                TableCell::new("$20".to_string()),
            ],
            vec![
                TableCell::new("Gizmo".to_string()),
                TableCell::new("$30".to_string()),
            ],
        ];
        let alignments = determine_column_alignments(&headers, &rows);
        assert_eq!(alignments[0], TextAlignment::Left);
        assert_eq!(alignments[1], TextAlignment::Right);
    }

    #[test]
    fn determine_column_alignments_leaves_mixed_columns_left_aligned() {
        let headers = vec![TableCell::new("Notes".to_string())];
        // Only 1 of 3 rows is numeric (33%) - below the 70% threshold.
        let rows = vec![
            vec![TableCell::new("42".to_string())],
            vec![TableCell::new("see attached".to_string())],
            vec![TableCell::new("n/a".to_string())],
        ];
        let alignments = determine_column_alignments(&headers, &rows);
        assert_eq!(alignments[0], TextAlignment::Left);
    }

    // --- extract_table_data (end-to-end through the real docx-rs types) ---

    #[test]
    fn extract_table_data_detects_header_row_and_parses_body() {
        let table = Table::new(vec![
            docx_row(&["Item", "Price"]),
            docx_row(&["Widget", "$10"]),
            docx_row(&["Gadget", "$20"]),
        ]);

        let element = extract_table_data(&table).expect("table should have content");
        let DocumentElement::Table { table: data } = element else {
            panic!("expected a Table element");
        };

        assert_eq!(data.headers.len(), 2);
        assert_eq!(data.headers[0].content, "Item");
        assert_eq!(data.headers[1].content, "Price");
        assert_eq!(data.rows.len(), 2);
        assert_eq!(data.rows[0][0].content, "Widget");
        assert_eq!(data.rows[0][1].content, "$10");
        assert_eq!(data.rows[0][1].data_type, CellDataType::Currency);
        assert!(data.metadata.has_headers);
    }

    #[test]
    fn extract_table_data_falls_back_to_first_row_when_no_header_detected() {
        // Long, prose-like first row - appears_to_be_header should reject it,
        // so it falls back to using the first row as headers anyway.
        let table = Table::new(vec![docx_row(&[
            "This first row is long enough that it will not look like a header",
        ])]);

        let element = extract_table_data(&table).expect("table should have content");
        let DocumentElement::Table { table: data } = element else {
            panic!("expected a Table element");
        };
        assert_eq!(data.headers.len(), 1);
        assert!(data.rows.is_empty());
    }

    #[test]
    fn extract_table_data_returns_none_for_empty_table() {
        let table = Table::new(vec![]);
        assert!(extract_table_data(&table).is_none());
    }
}
