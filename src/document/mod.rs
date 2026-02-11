//! Document parsing and data structures module
//!
//! This module provides functionality for parsing Microsoft Word (.docx) documents
//! and converting them into a structured representation.
//!
//! During refactoring: Incrementally extracting modules

pub mod models;

// Re-export all models
pub use models::*;

// Temporary: re-export everything from legacy file
pub use crate::document_legacy::*;
