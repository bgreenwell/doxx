use doxx::{
    ansi::{export_to_ansi_with_options, AnsiOptions},
    document::{Document, DocumentElement, FormattedRun, TextFormatting},
    ColorDepth,
};

#[test]
fn test_ansi_export_basic() {
    let document = create_test_document();
    let options = AnsiOptions {
        terminal_width: 80,
        color_depth: ColorDepth::TrueColor,
    };

    let result = export_to_ansi_with_options(&document, &options);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Test Document"));
    assert!(output.contains("Document Information"));
}

#[test]
fn test_ansi_export_formatting() {
    let document = create_formatted_document();
    let options = AnsiOptions {
        terminal_width: 80,
        color_depth: ColorDepth::TrueColor,
    };

    let result = export_to_ansi_with_options(&document, &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Check for ANSI formatting codes
    assert!(output.contains("[1m")); // Bold
    assert!(output.contains("[3m")); // Italic
    assert!(output.contains("[4m")); // Underline
    assert!(output.contains("[9m")); // Strikethrough
    assert!(output.contains("[38;2;")); // RGB color
    assert!(output.contains("[0m")); // Reset
}

#[test]
fn test_ansi_export_color_depths() {
    let document = create_colored_document();

    // Test monochrome (no colors)
    let monochrome_options = AnsiOptions {
        terminal_width: 80,
        color_depth: ColorDepth::Monochrome,
    };
    let mono_output = export_to_ansi_with_options(&document, &monochrome_options).unwrap();
    assert!(!mono_output.contains("[38;2;")); // No RGB colors
    assert!(!mono_output.contains("[38;5;")); // No ANSI colors

    // Test 16 colors
    let standard_options = AnsiOptions {
        terminal_width: 80,
        color_depth: ColorDepth::Standard,
    };
    let standard_output = export_to_ansi_with_options(&document, &standard_options).unwrap();
    assert!(standard_output.contains("[38;5;")); // ANSI colors
    assert!(!standard_output.contains("[38;2;")); // No RGB colors

    // Test true color
    let true_color_options = AnsiOptions {
        terminal_width: 80,
        color_depth: ColorDepth::TrueColor,
    };
    let true_color_output = export_to_ansi_with_options(&document, &true_color_options).unwrap();
    assert!(true_color_output.contains("[38;2;")); // RGB colors
}

#[test]
fn test_ansi_export_terminal_width() {
    let document = create_test_document();

    // Test narrow width
    let narrow_options = AnsiOptions {
        terminal_width: 40,
        color_depth: ColorDepth::Auto,
    };
    let narrow_output = export_to_ansi_with_options(&document, &narrow_options).unwrap();

    // Check that separator respects width
    let lines: Vec<&str> = narrow_output.lines().collect();
    let separator_line = lines.iter().find(|line| line.contains("====")).unwrap();
    // Should be 40 characters or close to it (accounting for ANSI codes)
    let clean_line = strip_ansi_codes(separator_line);
    assert_eq!(clean_line.len(), 40);

    // Test wide width
    let wide_options = AnsiOptions {
        terminal_width: 120,
        color_depth: ColorDepth::Auto,
    };
    let wide_output = export_to_ansi_with_options(&document, &wide_options).unwrap();
    let wide_lines: Vec<&str> = wide_output.lines().collect();
    let wide_separator = wide_lines.iter().find(|line| line.contains("====")).unwrap();
    let wide_clean = strip_ansi_codes(wide_separator);
    assert_eq!(wide_clean.len(), 50); // Limited by min(50, width)
}

#[test]
fn test_ansi_export_lists() {
    let document = create_list_document();
    let options = AnsiOptions::default();

    let result = export_to_ansi_with_options(&document, &options);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("1. ")); // Ordered list marker
    assert!(output.contains("• ")); // Unordered list marker
    assert!(output.contains("  ")); // Indentation for nested items
}

#[test]
fn test_ansi_export_tables() {
    let document = create_table_document();
    let options = AnsiOptions::default();

    let result = export_to_ansi_with_options(&document, &options);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("│")); // Table borders
    assert!(output.contains("─")); // Table borders
    assert!(output.contains("📊")); // Table icon
}

// Helper functions to create test documents

fn create_test_document() -> Document {
    use doxx::document::DocumentMetadata;

    Document {
        title: "Test Document".to_string(),
        metadata: DocumentMetadata {
            file_path: "test.docx".to_string(),
            file_size: 1024,
            word_count: 10,
            page_count: 1,
            created: None,
            modified: None,
            author: Some("Test Author".to_string()),
        },
        elements: vec![
            DocumentElement::Paragraph {
                runs: vec![FormattedRun {
                    text: "This is a simple paragraph.".to_string(),
                    formatting: TextFormatting::default(),
                }],
            }
        ],
        image_options: Default::default(),
    }
}

fn create_formatted_document() -> Document {
    use doxx::document::DocumentMetadata;

    let mut bold_formatting = TextFormatting::default();
    bold_formatting.bold = true;

    let mut italic_formatting = TextFormatting::default();
    italic_formatting.italic = true;

    let mut underline_formatting = TextFormatting::default();
    underline_formatting.underline = true;

    let mut strikethrough_formatting = TextFormatting::default();
    strikethrough_formatting.strikethrough = true;

    Document {
        title: "Formatted Document".to_string(),
        metadata: DocumentMetadata {
            file_path: "formatted.docx".to_string(),
            file_size: 2048,
            word_count: 20,
            page_count: 1,
            created: None,
            modified: None,
            author: None,
        },
        elements: vec![
            DocumentElement::Paragraph {
                runs: vec![
                    FormattedRun {
                        text: "Bold text ".to_string(),
                        formatting: bold_formatting,
                    },
                    FormattedRun {
                        text: "italic text ".to_string(),
                        formatting: italic_formatting,
                    },
                    FormattedRun {
                        text: "underlined text ".to_string(),
                        formatting: underline_formatting,
                    },
                    FormattedRun {
                        text: "strikethrough text".to_string(),
                        formatting: strikethrough_formatting,
                    },
                ],
            }
        ],
        image_options: Default::default(),
    }
}

fn create_colored_document() -> Document {
    use doxx::document::DocumentMetadata;

    let mut red_formatting = TextFormatting::default();
    red_formatting.color = Some("#FF0000".to_string());

    let mut blue_formatting = TextFormatting::default();
    blue_formatting.color = Some("#0000FF".to_string());

    Document {
        title: "Colored Document".to_string(),
        metadata: DocumentMetadata {
            file_path: "colored.docx".to_string(),
            file_size: 1536,
            word_count: 15,
            page_count: 1,
            created: None,
            modified: None,
            author: None,
        },
        elements: vec![
            DocumentElement::Paragraph {
                runs: vec![
                    FormattedRun {
                        text: "Red text ".to_string(),
                        formatting: red_formatting,
                    },
                    FormattedRun {
                        text: "Blue text".to_string(),
                        formatting: blue_formatting,
                    },
                ],
            }
        ],
        image_options: Default::default(),
    }
}

fn create_list_document() -> Document {
    use doxx::document::{DocumentMetadata, ListItem};

    Document {
        title: "List Document".to_string(),
        metadata: DocumentMetadata {
            file_path: "lists.docx".to_string(),
            file_size: 1280,
            word_count: 12,
            page_count: 1,
            created: None,
            modified: None,
            author: None,
        },
        elements: vec![
            DocumentElement::List {
                items: vec![
                    ListItem {
                        runs: vec![FormattedRun {
                            text: "First item".to_string(),
                            formatting: TextFormatting::default(),
                        }],
                        level: 0,
                    },
                    ListItem {
                        runs: vec![FormattedRun {
                            text: "Second item".to_string(),
                            formatting: TextFormatting::default(),
                        }],
                        level: 0,
                    },
                    ListItem {
                        runs: vec![FormattedRun {
                            text: "Nested item".to_string(),
                            formatting: TextFormatting::default(),
                        }],
                        level: 1,
                    },
                ],
                ordered: true,
            },
            DocumentElement::List {
                items: vec![
                    ListItem {
                        runs: vec![FormattedRun {
                            text: "Bullet item".to_string(),
                            formatting: TextFormatting::default(),
                        }],
                        level: 0,
                    },
                ],
                ordered: false,
            },
        ],
        image_options: Default::default(),
    }
}

fn create_table_document() -> Document {
    use doxx::document::{DocumentMetadata, TableData, TableMetadata, TableCell, TextAlignment, CellDataType};

    let table = TableData {
        headers: vec![
            TableCell {
                content: "Name".to_string(),
                alignment: TextAlignment::Left,
                formatting: TextFormatting::default(),
                data_type: CellDataType::Text,
            },
            TableCell {
                content: "Age".to_string(),
                alignment: TextAlignment::Right,
                formatting: TextFormatting::default(),
                data_type: CellDataType::Number,
            },
        ],
        rows: vec![
            vec![
                TableCell {
                    content: "Alice".to_string(),
                    alignment: TextAlignment::Left,
                    formatting: TextFormatting::default(),
                    data_type: CellDataType::Text,
                },
                TableCell {
                    content: "30".to_string(),
                    alignment: TextAlignment::Right,
                    formatting: TextFormatting::default(),
                    data_type: CellDataType::Number,
                },
            ],
        ],
        metadata: TableMetadata {
            title: Some("Test Table".to_string()),
            column_widths: vec![10, 5],
            column_alignments: vec![TextAlignment::Left, TextAlignment::Right],
            column_count: 2,
            row_count: 1,
            has_headers: true,
        },
    };

    Document {
        title: "Table Document".to_string(),
        metadata: DocumentMetadata {
            file_path: "table.docx".to_string(),
            file_size: 1792,
            word_count: 8,
            page_count: 1,
            created: None,
            modified: None,
            author: None,
        },
        elements: vec![DocumentElement::Table { table }],
        image_options: Default::default(),
    }
}

fn strip_ansi_codes(text: &str) -> String {
    // Simple ANSI code stripping for testing
    let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    ansi_regex.replace_all(text, "").to_string()
}