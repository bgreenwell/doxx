use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

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
    Heading { level: u8, text: String },
    Paragraph { text: String, formatting: TextFormatting },
    List { items: Vec<ListItem>, ordered: bool },
    Table { table: TableData },
    Image { description: String, width: Option<u32>, height: Option<u32> },
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
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

    // Enhanced content extraction with style information
    for child in &docx.document.children {
        match child {
            docx_rs::DocumentChild::Paragraph(para) => {
                let mut text = String::new();
                let mut formatting = TextFormatting::default();
                
                // Check paragraph style for heading information
                let heading_level = detect_heading_from_paragraph_style(para);
                
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
                    
                    // Use paragraph style first, then fallback to text heuristics
                    let level = heading_level.or_else(|| detect_heading_from_text(&text, &formatting));
                    
                    if let Some(level) = level {
                        elements.push(DocumentElement::Heading { level, text });
                    } else {
                        elements.push(DocumentElement::Paragraph { text, formatting });
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

fn extract_run_formatting(run: &docx_rs::Run) -> TextFormatting {
    let mut formatting = TextFormatting::default();
    
    // Access run properties directly (they're not optional in current API)
    let props = &run.run_property;
    formatting.bold = props.bold.is_some();
    formatting.italic = props.italic.is_some();
    formatting.underline = props.underline.is_some();
    
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
        if text.starts_with("⏺") || text.starts_with("⎿") || text.starts_with("☐") || text.starts_with("☒") {
            return None;
        }
        
        // Exclude if it contains typical sentence patterns
        if text.contains(" the ") || text.contains(" and ") || text.contains(" with ") || text.contains(" for ") {
            return None;
        }
        
        // Strong indicators of headings
        if formatting.bold && text.len() < 60 && text.len() > 5 {
            // Bold text that's reasonably short is likely a heading
            if !text.ends_with('.') && !text.ends_with(',') && !text.ends_with(';') && !text.ends_with(':') {
                return Some(determine_heading_level_from_text(text));
            }
        }
        
        // Check if it's all caps (but not just a short word)
        if text.len() > 15 && text.len() < 50 && text.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c.is_numeric() || c.is_ascii_punctuation()) {
            return Some(1);
        }
        
        // Very specific patterns that indicate headings
        if text.starts_with("Chapter ") || text.starts_with("Section ") || text.starts_with("Part ") {
            return Some(determine_heading_level_from_text(text));
        }
        
        // Look for standalone phrases that could be headings (very conservative)
        if text.len() < 40 && text.len() > 10
            && !text.ends_with('.') && !text.contains(',') && !text.contains('(') && !text.contains(':') {
                // Check if it has heading-like characteristics
                let words = text.split_whitespace().count();
                if (2..=5).contains(&words) {
                    // Must contain at least one meaningful word (longer than 3 chars)
                    let has_meaningful_word = text.split_whitespace()
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
    if text.contains(" and ") || text.contains(" but ") || text.contains(" however ") || text.contains(" therefore ") {
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
        if let DocumentElement::Heading { level, text } = element {
            outline.push(OutlineItem {
                title: text.clone(),
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
            
            let table_cell = TableCell::new(cell_text.trim().to_string())
                .with_formatting(cell_formatting);
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
    let avg_length = if !row.is_empty() { total_chars / row.len() } else { 0 };
    
    // Headers tend to be shorter and more concise
    if avg_length > 50 {
        return false;
    }
    
    // Check if most cells contain typical header words or are short phrases
    let header_indicators = row.iter().filter(|cell| {
        let cell_lower = cell.to_lowercase();
        let word_count = cell.split_whitespace().count();
        
        // Short phrases (1-3 words) are often headers
        if word_count <= 3 && !cell.trim().is_empty() {
            return true;
        }
        
        // Common header words
        if cell_lower.contains("name") || cell_lower.contains("date") || 
           cell_lower.contains("amount") || cell_lower.contains("type") ||
           cell_lower.contains("status") || cell_lower.contains("id") ||
           cell_lower.contains("description") || cell_lower.contains("count") {
            return true;
        }
        
        false
    }).count();
    
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
        self.metadata.column_widths.get(column_index).copied().unwrap_or(10)
    }
    
    pub fn _get_column_alignment(&self, column_index: usize) -> TextAlignment {
        self.metadata.column_alignments.get(column_index).copied().unwrap_or(TextAlignment::Left)
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

fn calculate_column_widths(headers: &[TableCell], rows: &[Vec<TableCell>]) -> Vec<usize> {
    if headers.is_empty() {
        return Vec::new();
    }
    
    let mut widths = headers.iter().map(|h| h.display_width()).collect::<Vec<_>>();
    
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

fn determine_column_alignments(headers: &[TableCell], rows: &[Vec<TableCell>]) -> Vec<TextAlignment> {
    let column_count = headers.len();
    let mut alignments = vec![TextAlignment::Left; column_count];
    
    for (col_index, alignment) in alignments.iter_mut().enumerate().take(column_count) {
        let mut numeric_count = 0;
        let mut total_count = 0;
        
        // Check data types in this column
        for row in rows {
            if let Some(cell) = row.get(col_index) {
                total_count += 1;
                if matches!(cell.data_type, CellDataType::Number | CellDataType::Currency | CellDataType::Percentage) {
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
        CellDataType::Number | CellDataType::Currency | CellDataType::Percentage => TextAlignment::Right,
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