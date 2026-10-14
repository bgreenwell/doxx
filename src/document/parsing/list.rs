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

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_run(text: &str) -> FormattedRun {
        FormattedRun {
            text: text.to_string(),
            formatting: TextFormatting::default(),
        }
    }

    fn bold_run(text: &str) -> FormattedRun {
        let formatting = TextFormatting {
            bold: true,
            ..TextFormatting::default()
        };
        FormattedRun {
            text: text.to_string(),
            formatting,
        }
    }

    fn paragraph(text: &str) -> DocumentElement {
        DocumentElement::Paragraph {
            runs: vec![plain_run(text)],
        }
    }

    // --- is_likely_list_item ---

    #[test]
    fn is_likely_list_item_recognizes_bullets_and_numbers() {
        assert!(is_likely_list_item(
            "1. This is a numbered item with enough content to count"
        ));
        assert!(is_likely_list_item("• Bullet item"));
        assert!(is_likely_list_item("- Dash item"));
        assert!(is_likely_list_item("* Star item"));
        assert!(is_likely_list_item("a. Lettered item"));
        assert!(is_likely_list_item("B. Uppercase lettered item"));
    }

    #[test]
    fn is_likely_list_item_rejects_short_numbered_text_as_a_heading_not_a_list() {
        // Short text after "N." reads as a heading ("1. Introduction"), not a
        // list item - the >20-char threshold is what distinguishes them.
        assert!(!is_likely_list_item("1. Introduction"));
    }

    #[test]
    fn is_likely_list_item_skips_already_processed_word_lists() {
        assert!(!is_likely_list_item(
            "__WORD_LIST__1. Already handled elsewhere"
        ));
    }

    #[test]
    fn is_likely_list_item_rejects_plain_text() {
        assert!(!is_likely_list_item("Just a regular sentence."));
        assert!(!is_likely_list_item(""));
    }

    #[test]
    fn is_likely_list_item_handles_multibyte_first_char_without_panicking() {
        // Regression guard for the text.chars().next().unwrap() call: a
        // multi-byte UTF-8 leading character must not panic, and since it's
        // not ASCII a-z/A-Z it should correctly be rejected as a list item.
        assert!(!is_likely_list_item("é. Not a lettered list item"));
    }

    #[test]
    fn is_likely_list_item_lettered_requires_more_than_three_bytes() {
        // "a." alone is only 2 bytes, below the `text.len() > 3` guard, so
        // it's correctly rejected rather than reaching the unwrap() path.
        assert!(!is_likely_list_item("a."));
    }

    // --- calculate_list_level ---

    #[test]
    fn calculate_list_level_from_leading_spaces() {
        assert_eq!(calculate_list_level("no indent"), 0);
        assert_eq!(calculate_list_level("  two spaces"), 1);
        assert_eq!(calculate_list_level("    four spaces"), 2);
    }

    // --- clean_list_item_runs ---

    #[test]
    fn clean_list_item_runs_strips_bullet_prefix() {
        let runs = vec![plain_run("• First item")];
        let cleaned = clean_list_item_runs(runs);
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].text, "First item");
    }

    #[test]
    fn clean_list_item_runs_strips_numbered_prefix() {
        let runs = vec![plain_run("12. Twelfth item")];
        let cleaned = clean_list_item_runs(runs);
        assert_eq!(cleaned[0].text, "Twelfth item");
    }

    #[test]
    fn clean_list_item_runs_strips_lettered_prefix() {
        let runs = vec![plain_run("a. Lettered item")];
        let cleaned = clean_list_item_runs(runs);
        assert_eq!(cleaned[0].text, "Lettered item");
    }

    #[test]
    fn clean_list_item_runs_preserves_formatting_across_multiple_runs() {
        // Prefix ("1. ") spans entirely within the first run; the second run
        // (bold) should be kept untouched, formatting intact.
        let runs = vec![plain_run("1. "), bold_run("Important text")];
        let cleaned = clean_list_item_runs(runs);
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].text, "Important text");
        assert!(cleaned[0].formatting.bold);
    }

    #[test]
    fn clean_list_item_runs_prefix_split_across_runs_keeps_remainder() {
        // The prefix "1. " straddles run boundaries: "1" in the first run,
        // ". Rest" in the second.
        let runs = vec![plain_run("1"), plain_run(". Rest of the text")];
        let cleaned = clean_list_item_runs(runs);
        let combined: String = cleaned.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(combined, "Rest of the text");
    }

    #[test]
    fn clean_list_item_runs_no_prefix_returns_runs_unchanged() {
        let runs = vec![plain_run("Just a normal paragraph.")];
        let cleaned = clean_list_item_runs(runs.clone());
        assert_eq!(cleaned, runs);
    }

    #[test]
    fn clean_list_item_runs_empty_input_returns_empty() {
        assert_eq!(clean_list_item_runs(vec![]), Vec::<FormattedRun>::new());
    }

    // --- group_list_items ---

    #[test]
    fn group_list_items_wraps_consecutive_numbered_paragraphs_into_one_ordered_list() {
        let elements = vec![
            paragraph("Some intro text that is not a list item at all"),
            paragraph("1. This is the first list item with enough text"),
            paragraph("2. This is the second list item with enough text"),
            paragraph("Some outro text that is also not a list item"),
        ];
        let result = group_list_items(elements);

        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], DocumentElement::Paragraph { .. }));
        match &result[1] {
            DocumentElement::List { items, ordered } => {
                assert!(ordered);
                assert_eq!(items.len(), 2);
                assert_eq!(
                    items[0].runs[0].text,
                    "This is the first list item with enough text"
                );
            }
            other => panic!("expected a List element, got {other:?}"),
        }
        assert!(matches!(result[2], DocumentElement::Paragraph { .. }));
    }

    #[test]
    fn group_list_items_flushes_the_list_when_switching_ordered_to_bulleted() {
        let elements = vec![
            paragraph("1. This is a numbered item with enough text to count"),
            paragraph("• A bullet item"),
        ];
        let result = group_list_items(elements);

        assert_eq!(result.len(), 2);
        match (&result[0], &result[1]) {
            (
                DocumentElement::List {
                    ordered: ordered_a, ..
                },
                DocumentElement::List {
                    ordered: ordered_b, ..
                },
            ) => {
                assert!(*ordered_a);
                assert!(!*ordered_b);
            }
            other => panic!("expected two separate List elements, got {other:?}"),
        }
    }

    #[test]
    fn group_list_items_flushes_a_trailing_list_at_end_of_document() {
        let elements = vec![paragraph(
            "1. Only item in a list that ends the document right here",
        )];
        let result = group_list_items(elements);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], DocumentElement::List { .. }));
    }

    #[test]
    fn group_list_items_non_paragraph_element_closes_an_open_list() {
        let elements = vec![
            paragraph("1. A list item with plenty of content to qualify as one"),
            DocumentElement::Heading {
                level: 1,
                text: "A Heading".to_string(),
                number: None,
            },
        ];
        let result = group_list_items(elements);
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], DocumentElement::List { .. }));
        assert!(matches!(result[1], DocumentElement::Heading { .. }));
    }

    #[test]
    fn group_list_items_no_lists_present_returns_elements_unchanged() {
        let elements = vec![paragraph("Just one plain paragraph.")];
        let result = group_list_items(elements.clone());
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], DocumentElement::Paragraph { .. }));
    }
}
