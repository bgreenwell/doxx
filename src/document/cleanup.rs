//! Post-processing and cleanup utilities
//!
//! This module provides helper functions for cleaning and processing
//! document elements after initial parsing.

use super::models::*;

pub(crate) fn is_likely_sentence(text: &str) -> bool {
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
    if text.contains(" and ")
        || text.contains(" but ")
        || text.contains(" however ")
        || text.contains(" therefore ")
    {
        return true;
    }

    false
}

pub(crate) fn estimate_page_count(word_count: usize) -> usize {
    // Rough estimate: 250 words per page
    (word_count as f32 / 250.0).ceil() as usize
}

pub(crate) fn clean_word_list_markers(elements: Vec<DocumentElement>) -> Vec<DocumentElement> {
    elements
        .into_iter()
        .map(|element| match element {
            DocumentElement::Paragraph { runs } => {
                let cleaned_runs = runs
                    .into_iter()
                    .map(|mut run| {
                        if run.text.starts_with("__WORD_LIST__") {
                            run.text = run
                                .text
                                .strip_prefix("__WORD_LIST__")
                                .unwrap_or(&run.text)
                                .to_string();
                        }
                        run
                    })
                    .collect();
                DocumentElement::Paragraph { runs: cleaned_runs }
            }
            DocumentElement::List { items, ordered } => {
                let cleaned_items = items
                    .into_iter()
                    .map(|item| {
                        let combined_text: String =
                            item.runs.iter().map(|run| run.text.as_str()).collect();
                        let cleaned_runs = if combined_text.starts_with("__WORD_LIST__") {
                            // Remove the __WORD_LIST__ prefix from the first run
                            let mut new_runs = item.runs.clone();
                            if let Some(first_run) = new_runs.first_mut() {
                                first_run.text = first_run
                                    .text
                                    .strip_prefix("__WORD_LIST__")
                                    .unwrap_or(&first_run.text)
                                    .to_string();
                            }
                            new_runs
                        } else {
                            item.runs.clone()
                        };
                        ListItem {
                            runs: cleaned_runs,
                            level: item.level,
                        }
                    })
                    .collect();
                DocumentElement::List {
                    items: cleaned_items,
                    ordered,
                }
            }
            other => other,
        })
        .collect()
}
