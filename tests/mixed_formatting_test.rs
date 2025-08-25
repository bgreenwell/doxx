use std::process::Command;

#[test]
fn test_color_showcase_document_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "doxx", "tests/fixtures/color-showcase.docx"])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully parse color showcase document: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Basic check - should contain some content
    assert!(!stdout.is_empty(), "Output should not be empty");
}

#[test]
fn test_mixed_formatting_export_text() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/formatting-showcase.docx",
            "--export",
            "text",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export formatting showcase to text"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain some text content
    assert!(
        stdout.len() > 10,
        "Exported text should have reasonable length"
    );
}

#[test]
fn test_mixed_formatting_export_markdown() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/formatting-showcase.docx",
            "--export",
            "markdown",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export formatting showcase to markdown"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain markdown formatting indicators
    assert!(
        stdout.contains("**") || stdout.contains("*") || stdout.contains("#"),
        "Markdown export should contain formatting indicators"
    );
}

#[test]
fn test_mixed_formatting_export_json() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/formatting-showcase.docx",
            "--export",
            "json",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export formatting showcase to JSON"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON
    assert!(
        stdout.contains("{"),
        "JSON export should contain JSON structure"
    );
    assert!(
        stdout.contains("\""),
        "JSON export should contain quoted strings"
    );
}

#[test]
fn test_mixed_formatting_document_structure() {
    // Test that documents with mixed formatting are properly parsed
    // This test ensures that no crashes occur when parsing complex formatting
    let test_files = [
        "tests/fixtures/color-showcase.docx",
        "tests/fixtures/formatting-showcase.docx",
        "tests/fixtures/business-report.docx",
    ];

    for file_path in &test_files {
        let output = Command::new("cargo")
            .args(["run", "--bin", "doxx", file_path])
            .output()
            .expect("Failed to execute doxx");

        assert!(
            output.status.success(),
            "doxx should successfully parse {}: {}",
            file_path,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "Output should not be empty for {}",
            file_path
        );
    }
}
