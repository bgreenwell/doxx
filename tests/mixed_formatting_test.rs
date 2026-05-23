use std::process::Command;

#[test]
fn test_color_document_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "doxx", "tests/fixtures/colors.docx"])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully parse colors document: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Output should not be empty");
}

#[test]
fn test_mixed_formatting_export_text() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/comprehensive.docx",
            "--export",
            "text",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export comprehensive doc to text"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
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
            "tests/fixtures/comprehensive.docx",
            "--export",
            "markdown",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export comprehensive doc to markdown"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
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
            "tests/fixtures/comprehensive.docx",
            "--export",
            "json",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export comprehensive doc to JSON"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
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
    let test_files = [
        "tests/fixtures/colors.docx",
        "tests/fixtures/comprehensive.docx",
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
            "Output should not be empty for {file_path}"
        );
    }
}
