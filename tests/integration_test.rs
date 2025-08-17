use std::path::Path;
use std::process::Command;

#[test]
fn test_minimal_document_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "doxx", "tests/fixtures/minimal.docx"])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully parse minimal.docx"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Minimal Test"),
        "Should contain document title"
    );
}

#[test]
fn test_tables_csv_export() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/tables-heavy.docx",
            "--export",
            "csv",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export tables to CSV"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Name,Age,City"),
        "Should contain CSV headers"
    );
}

#[test]
fn test_headings_outline() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/headings-hierarchy.docx",
            "--outline",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully generate outline"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Level 1:"), "Should contain heading levels");
}

#[test]
fn test_formatting_markdown_export() {
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
        "doxx should successfully export to markdown"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("**bold"),
        "Should contain markdown formatting"
    );
}

#[test]
fn test_unicode_document() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/unicode-special.docx",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully parse unicode document"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Unicode"), "Should contain unicode content");
}

#[test]
fn test_business_report_parsing() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/business-report.docx",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully parse business report"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("TechCorp"),
        "Should contain business content"
    );
}

#[test]
fn test_export_test_json() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/export-test.docx",
            "--export",
            "json",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export to JSON"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("{"), "Should contain JSON output");
}

#[test]
fn test_search_functionality() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "doxx",
            "tests/fixtures/business-report.docx",
            "--search",
            "revenue",
        ])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully search document"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Search Results"),
        "Should contain search results"
    );
}

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "doxx", "--", "--help"])
        .output()
        .expect("Failed to execute doxx");

    assert!(output.status.success(), "doxx should show help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("doxx"), "Should contain program name");
    assert!(
        stdout.contains("Beautiful .docx viewing"),
        "Should contain description"
    );
}

#[test]
fn test_all_fixtures_exist() {
    let fixtures = [
        "tests/fixtures/minimal.docx",
        "tests/fixtures/tables-heavy.docx",
        "tests/fixtures/headings-hierarchy.docx",
        "tests/fixtures/formatting-showcase.docx",
        "tests/fixtures/lists-comprehensive.docx",
        "tests/fixtures/unicode-special.docx",
        "tests/fixtures/business-report.docx",
        "tests/fixtures/export-test.docx",
    ];

    for fixture in &fixtures {
        assert!(
            Path::new(fixture).exists(),
            "Test fixture {fixture} should exist"
        );
    }
}
