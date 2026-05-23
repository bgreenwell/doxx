use std::path::Path;
use std::process::Command;

#[test]
fn test_minimal_document_parsing() {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/minimal.docx"])
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
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx", "--export", "csv"])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export tables to CSV"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Product") && stdout.contains("Quantity"),
        "Should contain CSV headers"
    );
}

#[test]
fn test_headings_outline() {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx", "--outline"])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully generate outline"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Text Formatting"),
        "Should contain heading from document"
    );
}

#[test]
fn test_formatting_markdown_export() {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx", "--export", "markdown"])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully export to markdown"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("**") && stdout.contains("*"),
        "Should contain markdown formatting (bold and italic)"
    );
}

#[test]
fn test_unicode_document() {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx", "--export", "text"])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully parse document with unicode"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("你好世界"),
        "Should contain CJK unicode content"
    );
}

#[test]
fn test_document_parsing() {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx"])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should successfully parse comprehensive document"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Text Formatting"),
        "Should contain document content"
    );
}

#[test]
fn test_export_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx", "--export", "json"])
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
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx", "--search", "revenue"])
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
fn test_empty_search_functionality() {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx", "--search", ""])
        .output()
        .expect("Failed to execute doxx");

    assert!(
        output.status.success(),
        "doxx should handle empty search gracefully"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No results found"),
        "Should show no results for empty search"
    );
}

#[test]
fn test_help_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["--help"])
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
        "tests/fixtures/comprehensive.docx",
        "tests/fixtures/colors.docx",
        "tests/fixtures/equations.docx",
        "tests/fixtures/images.docx",
    ];

    for fixture in &fixtures {
        assert!(
            Path::new(fixture).exists(),
            "Test fixture {fixture} should exist"
        );
    }
}
