use doxx::document::{search_document, load_document, ImageOptions};
use std::path::Path;

async fn load_test_document() -> doxx::document::Document {
    let path = Path::new("tests/fixtures/business-report.docx");
    load_document(path, ImageOptions::default())
        .await
        .expect("Failed to load test document")
}

#[cfg(test)]
mod search_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_search_returns_no_results() {
        let document = load_test_document().await;

        // Test empty string
        let results = search_document(&document, "");
        assert!(results.is_empty(), "Empty search should return no results");

        // Test whitespace-only string
        let results = search_document(&document, "   ");
        assert!(results.is_empty(), "Whitespace-only search should return no results");
    }

    #[tokio::test]
    async fn test_normal_search_returns_results() {
        let document = load_test_document().await;

        let results = search_document(&document, "revenue");
        assert!(!results.is_empty(), "Search for 'revenue' should return results");
        assert!(results.len() >= 3, "Should find multiple matches for 'revenue' in business report");
    }

    #[tokio::test]
    async fn test_case_insensitive_search() {
        let document = load_test_document().await;

        let results_lower = search_document(&document, "revenue");
        let results_upper = search_document(&document, "REVENUE");
        let results_mixed = search_document(&document, "Revenue");

        assert_eq!(results_lower.len(), results_upper.len(), "Search should be case insensitive");
        assert_eq!(results_lower.len(), results_mixed.len(), "Search should be case insensitive");
        assert!(!results_lower.is_empty(), "Should find revenue mentions");
    }

    #[tokio::test]
    async fn test_search_multiple_elements() {
        let document = load_test_document().await;

        // Search for content that appears in multiple elements
        let results = search_document(&document, "Q4");
        assert!(results.len() >= 1, "Should find 'Q4' in the business report");
    }
}

// Note: UI-specific tests for toggle_search_state are tested via integration tests
// since the App struct and UI module are not exported from the library.

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_search_with_special_characters() {
        let document = load_test_document().await;

        // Test search with various special characters
        let results = search_document(&document, ".");
        assert!(!results.is_empty(), "Should find periods in the text");

        let results = search_document(&document, "$");
        assert!(!results.is_empty(), "Should find dollar signs in financial data");
    }

    #[tokio::test]
    async fn test_search_preserves_element_index() {
        let document = load_test_document().await;

        let results = search_document(&document, "Executive");
        assert!(!results.is_empty(), "Should find 'Executive' heading");
        assert!(results[0].element_index < document.elements.len(), "Element index should be valid");
    }

    #[tokio::test]
    async fn test_search_result_positions() {
        let document = load_test_document().await;

        let results = search_document(&document, "revenue");
        assert!(!results.is_empty(), "Should find revenue in document");

        let result = &results[0];
        assert!(result.start_pos < result.end_pos, "Start position should be before end position");
        assert_eq!(result.end_pos - result.start_pos, 7, "Should match the length of 'revenue'");
    }

    #[tokio::test]
    async fn test_search_in_table_content() {
        let document = load_test_document().await;

        // Business report has table content
        let results = search_document(&document, "Growth");
        // Should find text in various document elements
        assert!(!results.is_empty(), "Should find search terms across different element types");
    }
}