//! List processing and detection
//!
//! This module handles detection of list items from paragraphs and
//! grouping them into hierarchical list structures.

use super::super::models::*;

pub(crate) fn is_likely_list_item(text: &str) -> bool {
    let text = text.trim();

    // Skip Word-formatted list items to avoid reprocessing
    if text.starts_with("__WORD_LIST__") {
        return false;
    }

    // Check for numbered list patterns that are NOT headings
    if text.starts_with(char::is_numeric) {
        // If it starts with a number followed by "." and then has substantial content,
        // it's likely a list item, not a heading
        if let Some(dot_pos) = text.find('.') {
            // Safe: '.' is ASCII, so dot_pos+1 is guaranteed to be a char boundary
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

pub(crate) fn group_list_items(elements: Vec<DocumentElement>) -> Vec<DocumentElement> {
    let mut result = Vec::new();
    let mut current_list_items = Vec::new();
    let mut current_list_ordered = false;

    for element in elements {
        match &element {
            DocumentElement::Paragraph { runs } => {
                // Get the combined text from all runs for list detection
                let text: String = runs.iter().map(|run| run.text.as_str()).collect();

                if is_likely_list_item(&text) {
                    // Determine if this is an ordered list item
                    let is_ordered = text.trim().starts_with(char::is_numeric);

                    // If we're starting a new list or switching list types, finish the current list
                    if !current_list_items.is_empty() && is_ordered != current_list_ordered {
                        result.push(DocumentElement::List {
                            items: std::mem::take(&mut current_list_items),
                            ordered: current_list_ordered,
                        });
                    }

                    current_list_ordered = is_ordered;

                    // Calculate nesting level from indentation
                    let level = calculate_list_level(&text);

                    // Clean the runs (remove bullet/number prefix from first run)
                    let clean_runs = clean_list_item_runs(runs.clone());

                    current_list_items.push(ListItem {
                        runs: clean_runs,
                        level,
                    });
                } else {
                    // Not a list item, so finish any current list
                    if !current_list_items.is_empty() {
                        result.push(DocumentElement::List {
                            items: std::mem::take(&mut current_list_items),
                            ordered: current_list_ordered,
                        });
                    }
                    result.push(element);
                }
            }
            _ => {
                // Non-paragraph element, finish any current list
                if !current_list_items.is_empty() {
                    result.push(DocumentElement::List {
                        items: std::mem::take(&mut current_list_items),
                        ordered: current_list_ordered,
                    });
                }
                result.push(element);
            }
        }
    }

    // Don't forget the last list if the document ends with one
    if !current_list_items.is_empty() {
        result.push(DocumentElement::List {
            items: current_list_items,
            ordered: current_list_ordered,
        });
    }

    result
}

fn calculate_list_level(text: &str) -> u8 {
    // Count leading whitespace to determine nesting level
    let leading_spaces = text.len() - text.trim_start().len();

    // Convert spaces to levels (every 2-4 spaces = 1 level)
    // Use 2 spaces per level as it's common in Word documents
    (leading_spaces / 2) as u8
}

fn clean_list_item_runs(runs: Vec<FormattedRun>) -> Vec<FormattedRun> {
    if runs.is_empty() {
        return runs;
    }

    // Get the combined text to determine what prefix to remove
    let combined_text: String = runs.iter().map(|run| run.text.as_str()).collect();
    let text = combined_text.trim();

    // Determine what prefix we need to remove
    let prefix_to_remove = if text.starts_with("• ") {
        "• "
    } else if text.starts_with("- ") {
        "- "
    } else if text.starts_with("* ") {
        "* "
    } else if let Some(dot_pos) = text.find('.') {
        let prefix = &text[..dot_pos];
        if prefix.chars().all(|c| c.is_ascii_digit()) {
            // For numbered lists, include the dot and following space
            &text[..dot_pos
                + if text.chars().nth(dot_pos + 1) == Some(' ') {
                    2
                } else {
                    1
                }]
        } else if text.chars().count() > 2 && text.chars().nth(1) == Some('.') {
            let first_char = text.chars().next().unwrap();
            if first_char.is_ascii_lowercase() || first_char.is_ascii_uppercase() {
                // For lettered lists, include the letter, dot, and following space
                &text[..if text.chars().nth(2) == Some(' ') {
                    3
                } else {
                    2
                }]
            } else {
                ""
            }
        } else {
            ""
        }
    } else {
        ""
    };

    if prefix_to_remove.is_empty() {
        return runs;
    }

    // Remove the prefix from the runs while preserving formatting
    let mut result_runs = Vec::new();
    let mut chars_to_remove = prefix_to_remove.chars().count();

    for run in runs {
        if chars_to_remove == 0 {
            // No more prefix to remove, keep this run as-is
            result_runs.push(run);
        } else {
            let run_char_count = run.text.chars().count();
            if run_char_count <= chars_to_remove {
                // This entire run is part of the prefix to remove
                chars_to_remove -= run_char_count;
            } else {
                // This run contains part of the text we want to keep
                let keep_text: String = run.text.chars().skip(chars_to_remove).collect();
                if !keep_text.is_empty() {
                    result_runs.push(FormattedRun {
                        text: keep_text.trim_start().to_string(),
                        formatting: run.formatting,
                    });
                }
                chars_to_remove = 0;
            }
        }
    }

    result_runs
}
