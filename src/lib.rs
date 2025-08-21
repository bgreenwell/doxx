//! doxx: Terminal document viewer for .docx files
//!
//! This library provides functionality for parsing Microsoft Word documents
//! and displaying them in terminal environments with rich formatting support.

pub mod document;
pub mod export;
pub mod image_extractor;
pub mod terminal_image;

/// Export format options
#[derive(clap::ValueEnum, Clone)]
pub enum ExportFormat {
    Markdown,
    Text,
    Csv,
    Json,
}

// Re-export commonly used types
pub use document::{Document, DocumentElement};
pub use image_extractor::ImageExtractor;
pub use terminal_image::{TerminalImageRenderer, TerminalImageSupport};
