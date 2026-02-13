//! Core data structures for document representation
//!
//! This module defines all the public types used to represent a parsed document,
//! including elements, formatting, tables, and metadata.

use serde::{Deserialize, Serialize};

// Type aliases for convenience
pub type TableRows = Vec<Vec<TableCell>>;
pub type NumberingInfo = (i32, u8);

/// Image rendering options
#[derive(Debug, Clone, Default)]
pub struct ImageOptions {
    pub enabled: bool,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub scale: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub metadata: DocumentMetadata,
    pub elements: Vec<DocumentElement>,
    #[serde(skip)]
    pub image_options: ImageOptions,
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
        runs: Vec<FormattedRun>,
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
        relationship_id: Option<String>, // Link to DOCX relationship for image extraction
        image_path: Option<std::path::PathBuf>, // Path to extracted image file
    },
    Equation {
        latex: String,
        fallback: String,
    },
    PageBreak,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TextFormatting {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_size: Option<f32>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedRun {
    pub text: String,
    pub formatting: TextFormatting,
}

impl FormattedRun {
    /// Consolidate adjacent runs with identical formatting into single runs
    pub fn consolidate_runs(runs: Vec<FormattedRun>) -> Vec<FormattedRun> {
        if runs.is_empty() {
            return runs;
        }

        let mut consolidated = Vec::new();
        let mut current_run = runs[0].clone();

        for run in runs.into_iter().skip(1) {
            if current_run.formatting == run.formatting {
                // Same formatting - merge the text
                current_run.text.push_str(&run.text);
            } else {
                // Different formatting - push current and start new
                consolidated.push(current_run);
                current_run = run;
            }
        }

        // last run
        consolidated.push(current_run);
        consolidated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub runs: Vec<FormattedRun>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineItem {
    pub title: String,
    pub level: u8,
    pub element_index: usize,
}
