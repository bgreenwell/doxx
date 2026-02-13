//! Numbering management for lists and headings
//!
//! This module handles stateful numbering tracking for both list items
//! and heading auto-numbering, including hierarchical number generation.

use once_cell::sync::Lazy;
use regex::Regex;

/// Type alias for numbering counters to simplify complex HashMap type
pub(crate) type NumberingCounters = std::collections::HashMap<(i32, u8), u32>;

/// Type alias for heading number and cleaned text
pub(crate) type HeadingNumberInfo = (String, String);

/// Manages document-wide numbering state for proper sequential numbering
#[derive(Debug)]
pub(crate) struct DocumentNumberingManager {
    /// Counters for each (numId, level) combination
    /// Key: (numId, level), Value: current counter
    counters: NumberingCounters,
}

impl DocumentNumberingManager {
    pub(crate) fn new() -> Self {
        Self {
            counters: NumberingCounters::new(),
        }
    }

    /// Generate the next number for a given numId and level
    pub(crate) fn generate_number(
        &mut self,
        num_id: i32,
        level: u8,
        format: NumberingFormat,
    ) -> String {
        // Get current counter for this (numId, level) combination
        let key = (num_id, level);
        let counter_value = {
            let counter = self.counters.entry(key).or_insert(0);
            *counter += 1;
            *counter
        };

        // Reset deeper levels when we increment a higher level
        // This handles hierarchical numbering like 1. -> 1.1 -> 2. (reset 1.1 back to 2.1)
        self.reset_deeper_levels(num_id, level);

        // For hierarchical numbering, we need to build the full number string
        self.format_hierarchical_number(num_id, level, counter_value, format)
    }

    fn reset_deeper_levels(&mut self, num_id: i32, current_level: u8) {
        // Reset all levels deeper than current_level for this numId
        let keys_to_reset: Vec<_> = self
            .counters
            .keys()
            .filter(|(id, level)| *id == num_id && *level > current_level)
            .cloned()
            .collect();

        for key in keys_to_reset {
            self.counters.remove(&key);
        }
    }

    fn format_number(&self, counter: u32, format: NumberingFormat) -> String {
        match format {
            NumberingFormat::Decimal => format!("{counter}. "),
            NumberingFormat::LowerLetter => {
                // Convert 1->a, 2->b, etc.
                if counter <= 26 {
                    let letter = (b'a' + (counter - 1) as u8) as char;
                    format!("{letter}. ")
                } else {
                    format!("{counter}. ") // Fallback for > 26
                }
            }
            NumberingFormat::LowerRoman => format!("{}. ", Self::to_roman(counter).to_lowercase()),
            NumberingFormat::UpperLetter => {
                // Convert 1->A, 2->B, etc.
                if counter <= 26 {
                    let letter = (b'A' + (counter - 1) as u8) as char;
                    format!("{letter}. ")
                } else {
                    format!("{counter}. ") // Fallback for > 26
                }
            }
            NumberingFormat::UpperRoman => format!("{}. ", Self::to_roman(counter)),
            NumberingFormat::ParenLowerLetter => {
                if counter <= 26 {
                    let letter = (b'a' + (counter - 1) as u8) as char;
                    format!("({letter})")
                } else {
                    format!("({counter})")
                }
            }
            NumberingFormat::ParenLowerRoman => {
                format!("({})", Self::to_roman(counter).to_lowercase())
            }
            NumberingFormat::Bullet => "* ".to_string(),
        }
    }

    fn to_roman(num: u32) -> String {
        let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
        let symbols = [
            "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
        ];

        let mut result = String::new();
        let mut n = num;

        for (i, &value) in values.iter().enumerate() {
            while n >= value {
                result.push_str(symbols[i]);
                n -= value;
            }
        }

        result
    }

    /// Format hierarchical number (e.g., "2.1", "3.2.1")
    fn format_hierarchical_number(
        &self,
        num_id: i32,
        level: u8,
        counter: u32,
        format: NumberingFormat,
    ) -> String {
        // Check if this numId/level combination should use hierarchical numbering
        let needs_hierarchy = matches!((num_id, level), (4, 1)); // 2.1, 2.2, etc.

        if needs_hierarchy {
            // Build hierarchical number by including parent level counters
            let mut parts = Vec::new();

            // Add parent level counter (level 0 for this numId)
            if let Some(parent_counter) = self.counters.get(&(num_id, 0)) {
                parts.push(parent_counter.to_string());
            }

            // Add current level counter
            parts.push(counter.to_string());

            // Join with dots and add final punctuation
            format!("{}. ", parts.join("."))
        } else {
            // Use regular formatting for non-hierarchical levels
            self.format_number(counter, format)
        }
    }
}

/// Different numbering formats supported by Word
#[derive(Debug, Clone, Copy)]
pub(crate) enum NumberingFormat {
    Decimal,          // 1. 2. 3.
    LowerLetter,      // a. b. c.
    UpperLetter,      // A. B. C.
    LowerRoman,       // i. ii. iii.
    UpperRoman,       // I. II. III.
    ParenLowerLetter, // (a) (b) (c)
    ParenLowerRoman,  // (i) (ii) (iii)
    #[allow(dead_code)]
    Bullet, // * * *
}

#[derive(Debug, Clone)]
pub(crate) struct HeadingInfo {
    pub(crate) level: u8,
    pub(crate) number: Option<String>,
    pub(crate) clean_text: Option<String>, // Text with number removed
}

pub(crate) struct HeadingNumberTracker {
    counters: [u32; 6], // Support up to 6 heading levels
    auto_numbering_enabled: bool,
}

impl HeadingNumberTracker {
    pub(crate) fn new() -> Self {
        Self {
            counters: [0; 6],
            auto_numbering_enabled: false,
        }
    }

    pub(crate) fn enable_auto_numbering(&mut self) {
        self.auto_numbering_enabled = true;
    }

    pub(crate) fn get_number(&mut self, level: u8) -> String {
        if !self.auto_numbering_enabled {
            return String::new();
        }

        let level_index = (level.saturating_sub(1) as usize).min(5);

        // Increment current level
        self.counters[level_index] += 1;

        // Reset all deeper levels
        for i in (level_index + 1)..6 {
            self.counters[i] = 0;
        }

        // Build number string (1.2.3 format)
        let mut parts = Vec::new();
        for i in 0..=level_index {
            if self.counters[i] > 0 {
                parts.push(self.counters[i].to_string());
            }
        }

        parts.join(".")
    }
}

/// Analyze document structure to determine if automatic numbering should be enabled
pub(crate) fn analyze_heading_structure(document: &docx_rs::Document) -> bool {
    let mut heading_count = 0;
    let mut has_explicit_numbering = false;
    let mut level_counts = [0u32; 6]; // Count headings at each level

    for child in &document.children {
        if let docx_rs::DocumentChild::Paragraph(para) = child {
            // Note: detect_heading_from_paragraph_style and extract_paragraph_text
            // will be in the heading/formatting modules, but we use them here
            // This creates a circular dependency that we'll resolve in later phases
            if let Some(heading_level) = super::heading::detect_heading_from_paragraph_style(para) {
                let text = super::formatting::extract_paragraph_text(para);

                // Check if this heading has explicit numbering in the text
                if extract_heading_number_from_text(&text).is_some() {
                    has_explicit_numbering = true;
                }

                heading_count += 1;
                let level_index = (heading_level.saturating_sub(1) as usize).min(5);
                level_counts[level_index] += 1;
            }
        }
    }

    // Don't auto-number if:
    // 1. Any headings have explicit numbering
    // 2. Very few headings (less than 3)
    // 3. Only one level of headings (no hierarchy)
    if has_explicit_numbering || heading_count < 3 {
        return false;
    }

    // Check if we have a real hierarchy (headings at multiple levels)
    let levels_with_headings = level_counts.iter().filter(|&&count| count > 0).count();

    // Auto-number if we have multiple levels or multiple headings at level 1
    levels_with_headings > 1 || level_counts[0] > 1
}

// Lazy static regex patterns for heading number detection
// Focused on common patterns for manual numbering in text
static HEADING_NUMBER_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Standard decimal numbering: "1.", "1.1", "1.1.1", "2.1.1" (most common)
        // For single numbers, require a period to distinguish from "Heading 1" style titles
        // For hierarchical numbers (1.1, 1.2.3), period is optional
        Regex::new(r"^(\d+(?:\.\d+)+\.?|\d+\.)\s+(.+)$").unwrap(),
        // Section numbering: "Section 1.2", "Chapter 3"
        Regex::new(r"^((?:Section|Chapter|Part)\s+\d+(?:\.\d+)*\.?)\s+(.+)$").unwrap(),
        // Alternative numbering schemes (less common, but still useful)
        Regex::new(r"^([A-Z]\.)\s+(.+)$").unwrap(), // "A. Introduction"
        Regex::new(r"^([IVX]+\.)\s+(.+)$").unwrap(), // "I. Overview"
    ]
});

pub(crate) fn extract_heading_number_from_text(text: &str) -> Option<HeadingNumberInfo> {
    let text = text.trim();

    // Early return for empty text
    if text.is_empty() {
        return None;
    }

    // Try each pattern until one matches
    for pattern in HEADING_NUMBER_PATTERNS.iter() {
        if let Some(captures) = pattern.captures(text) {
            if let (Some(number_match), Some(text_match)) = (captures.get(1), captures.get(2)) {
                let number = number_match.as_str().trim_end_matches('.');
                let remaining_text = text_match.as_str().trim();

                // Only return if we have both number and meaningful text
                if !number.is_empty() && !remaining_text.is_empty() {
                    return Some((number.to_string(), remaining_text.to_string()));
                }
            }
        }
    }

    None
}
