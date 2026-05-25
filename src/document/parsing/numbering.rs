//! Numbering management for lists and headings
//!
//! This module handles stateful numbering tracking for both list items
//! and heading auto-numbering, including hierarchical number generation.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Parse a Word numFmt string into a `NumberingFormat` variant.
pub(crate) fn parse_numbering_format(fmt_str: &str) -> NumberingFormat {
    match fmt_str {
        "decimal" | "decimalZero" => NumberingFormat::Decimal,
        "lowerLetter" => NumberingFormat::LowerLetter,
        "upperLetter" => NumberingFormat::UpperLetter,
        "lowerRoman" => NumberingFormat::LowerRoman,
        "upperRoman" => NumberingFormat::UpperRoman,
        "parenLowerLetter" => NumberingFormat::ParenLowerLetter,
        "parenLowerRoman" => NumberingFormat::ParenLowerRoman,
        _ => NumberingFormat::Decimal,
    }
}

/// Format a counter value using the given numbering format.
pub(crate) fn format_number_static(counter: u32, format: NumberingFormat) -> String {
    match format {
        NumberingFormat::Decimal => format!("{counter}. "),
        NumberingFormat::LowerLetter => {
            if counter <= 26 {
                format!("{}. ", (b'a' + (counter - 1) as u8) as char)
            } else {
                format!("{counter}. ")
            }
        }
        NumberingFormat::UpperLetter => {
            if counter <= 26 {
                format!("{}. ", (b'A' + (counter - 1) as u8) as char)
            } else {
                format!("{counter}. ")
            }
        }
        NumberingFormat::LowerRoman => {
            format!("{}. ", roman_numeral(counter).to_lowercase())
        }
        NumberingFormat::UpperRoman => format!("{}. ", roman_numeral(counter)),
        NumberingFormat::ParenLowerLetter => {
            if counter <= 26 {
                format!("({}) ", (b'a' + (counter - 1) as u8) as char)
            } else {
                format!("({counter}) ")
            }
        }
        NumberingFormat::ParenLowerRoman => {
            format!("({}) ", roman_numeral(counter).to_lowercase())
        }
        NumberingFormat::Bullet => "* ".to_string(),
    }
}

fn roman_numeral(num: u32) -> String {
    const VALUES: &[u32] = &[1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
    const SYMBOLS: &[&str] = &[
        "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
    ];
    let mut result = String::new();
    let mut n = num;
    for (i, &value) in VALUES.iter().enumerate() {
        while n >= value {
            result.push_str(SYMBOLS[i]);
            n -= value;
        }
    }
    result
}

/// Resolves DOCX numbering definitions to determine list ordering and formatting.
///
/// Tracks counters per `(abstractNumId, level)` so that different `numId` values
/// that share the same abstract numbering definition continue counting sequentially.
/// Start overrides in a `numId`'s level overrides are applied once on first use.
pub(crate) struct NumberingResolver {
    /// (abstractNumId, level) → (numFmt string, default start value)
    abstract_levels: HashMap<(usize, usize), (String, usize)>,
    /// numId → (abstractNumId, level → start_override)
    num_instances: HashMap<usize, (usize, HashMap<usize, usize>)>,
    /// Counter state per (abstractNumId, level)
    counters: HashMap<(usize, usize), u32>,
    /// Tracks which (numId, level) pairs have already applied their start override
    applied_overrides: HashSet<(usize, usize)>,
}

impl NumberingResolver {
    /// Build a resolver from a parsed docx numbering table.
    pub(crate) fn build_from_docx(numberings: &docx_rs::Numberings) -> Self {
        let mut abstract_levels = HashMap::new();
        let mut num_instances = HashMap::new();

        for abstract_num in &numberings.abstract_nums {
            for level in &abstract_num.levels {
                let start: usize = serde_json::to_value(&level.start)
                    .ok()
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as usize;
                abstract_levels.insert(
                    (abstract_num.id, level.level),
                    (level.format.val.clone(), start),
                );
            }
        }

        for numbering in &numberings.numberings {
            let mut overrides = HashMap::new();
            for lo in &numbering.level_overrides {
                if let Some(start) = lo.override_start {
                    overrides.insert(lo.level, start);
                }
            }
            num_instances.insert(numbering.id, (numbering.abstract_num_id, overrides));
        }

        Self {
            abstract_levels,
            num_instances,
            counters: HashMap::new(),
            applied_overrides: HashSet::new(),
        }
    }

    /// Return true if the (numId, level) pair is an ordered (numbered) list.
    pub(crate) fn is_ordered(&self, num_id: i32, level: u8) -> bool {
        let num_id = num_id as usize;
        let level = level as usize;
        if let Some((abstract_num_id, _)) = self.num_instances.get(&num_id) {
            if let Some((fmt_str, _)) = self.abstract_levels.get(&(*abstract_num_id, level)) {
                return fmt_str != "bullet" && fmt_str != "none";
            }
        }
        true
    }

    /// Generate the next formatted number string for a (numId, level) pair.
    pub(crate) fn generate_number(&mut self, num_id: i32, level: u8) -> String {
        let num_id = num_id as usize;
        let level = level as usize;

        let Some((abstract_num_id, overrides)) = self.num_instances.get(&num_id) else {
            return format!("{}. ", level + 1);
        };
        let abstract_num_id = *abstract_num_id;
        let start_override = overrides.get(&level).copied();
        let format_str = self
            .abstract_levels
            .get(&(abstract_num_id, level))
            .map(|(s, _)| s.clone())
            .unwrap_or_else(|| "decimal".to_string());

        let counter_key = (abstract_num_id, level);
        let override_key = (num_id, level);

        // Apply a start override exactly once per (numId, level)
        if let Some(start) = start_override {
            if !self.applied_overrides.contains(&override_key) {
                self.applied_overrides.insert(override_key);
                *self.counters.entry(counter_key).or_insert(0) = (start as u32).saturating_sub(1);
            }
        }

        // Reset deeper levels for this abstract numbering
        let keys_to_reset: Vec<_> = self
            .counters
            .keys()
            .filter(|(aid, lvl)| *aid == abstract_num_id && *lvl > level)
            .cloned()
            .collect();
        for k in keys_to_reset {
            self.counters.remove(&k);
        }

        let counter = {
            let c = self.counters.entry(counter_key).or_insert(0);
            *c += 1;
            *c
        };

        let format = parse_numbering_format(&format_str);
        format_number_static(counter, format)
    }
}

/// Type alias for heading number and cleaned text
pub(crate) type HeadingNumberInfo = (String, String);

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
