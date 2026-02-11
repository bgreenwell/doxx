//! Document search and navigation operations
//!
//! This module provides read-only querying operations on parsed documents,
//! including full-text search and outline generation.

use super::models::*;

pub fn search_document(document: &Document, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    // TODO: consider deferring search execution until Enter is pressed
    if query.is_empty() {
        return results;
    }
    let query_lower = query.to_lowercase();

    for (element_index, element) in document.elements.iter().enumerate() {
        let text = match element {
            DocumentElement::Heading { text, .. } => text,
            DocumentElement::Paragraph { runs } => {
                // Combine text from all runs for searching
                &runs.iter().map(|run| run.text.as_str()).collect::<String>()
            }
            DocumentElement::List { items, .. } => {
                // Search in list items
                for item in items {
                    let item_text: String = item.runs.iter().map(|run| run.text.as_str()).collect();
                    let text_lower = item_text.to_lowercase();
                    if let Some(start_pos) = text_lower.find(&query_lower) {
                        results.push(SearchResult {
                            element_index,
                            text: item_text,
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
            DocumentElement::Equation { latex, .. } => latex,
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
        if let DocumentElement::Heading {
            level,
            text,
            number,
        } = element
        {
            let title = if let Some(number) = number {
                format!("{number} {text}")
            } else {
                text.clone()
            };
            outline.push(OutlineItem {
                title,
                level: *level,
                element_index: index,
            });
        }
    }

    outline
}
