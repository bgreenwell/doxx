//! doxx: Terminal document viewer for .docx files
//!
//! This library provides functionality for parsing Microsoft Word documents
//! and displaying them in terminal environments with rich formatting support.

pub mod ansi;
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
    Ansi,
}

/// Color depth options for ANSI export
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ColorDepth {
    /// Auto-detect terminal color capabilities
    Auto,
    /// Monochrome (no colors)
    #[value(name = "1")]
    Monochrome,
    /// 16 colors
    #[value(name = "4")]
    Standard,
    /// 256 colors
    #[value(name = "8")]
    Extended,
    /// 24-bit true color
    #[value(name = "24")]
    TrueColor,
}

// Re-export commonly used types
pub use document::{Document, DocumentElement};
pub use image_extractor::ImageExtractor;
pub use terminal_image::{TerminalImageRenderer, TerminalImageSupport};
