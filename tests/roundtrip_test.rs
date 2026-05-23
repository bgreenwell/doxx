/// Round-trip test: comprehensive.md → pandoc → comprehensive.docx → doxx markdown export
///
/// comprehensive.md is the canonical source document. This test verifies that doxx's
/// markdown exporter preserves the content and formatting from the pandoc-generated docx.
/// It serves as a regression guard: changes to the markdown exporter that drop content
/// or break formatting will fail here.
///
/// Notes on known non-round-trippable aspects (not tested here):
/// - Heading auto-numbers: doxx adds section numbers (1, 1.1) when it detects hierarchy
/// - List detection: pandoc list items are parsed as paragraphs by doxx
/// - Underline: not emitted by the markdown exporter
use std::process::Command;

fn export_comprehensive_markdown() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_doxx"))
        .args(["tests/fixtures/comprehensive.docx", "--export", "markdown"])
        .output()
        .expect("Failed to run doxx");

    assert!(
        output.status.success(),
        "doxx exited with error: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn test_headings_preserved() {
    let md = export_comprehensive_markdown();
    // Heading text is preserved (auto-numbers are prepended but text still present)
    assert!(md.contains("Text Formatting"), "H1 heading text missing");
    assert!(md.contains("Heading Level Two"), "H2 heading text missing");
    assert!(
        md.contains("Heading Level Three"),
        "H3 heading text missing"
    );
    assert!(md.contains("Heading Level Four"), "H4 heading text missing");
    assert!(md.contains("Heading Level Five"), "H5 heading text missing");
    assert!(md.contains("Heading Level Six"), "H6 heading text missing");
}

#[test]
fn test_inline_formatting_preserved() {
    let md = export_comprehensive_markdown();
    assert!(md.contains("**bold text**"), "Bold formatting missing");
    assert!(md.contains("*italic text*"), "Italic formatting missing");
    assert!(
        md.contains("~~strikethrough~~"),
        "Strikethrough formatting missing"
    );
    // Combined formatting on the same paragraph
    assert!(
        md.contains("**bold**") && md.contains("*italic*") && md.contains("~~strikethrough~~"),
        "Combined formatting missing"
    );
}

#[test]
fn test_table_preserved() {
    let md = export_comprehensive_markdown();
    assert!(
        md.contains("| Product | Quantity | Price |"),
        "Table header missing"
    );
    assert!(md.contains("| Widget A |"), "Table row missing");
    assert!(md.contains("| Widget B |"), "Table row missing");
    assert!(md.contains("| Widget C |"), "Table row missing");
    // Alignment row must be present
    assert!(md.contains(":---"), "Table alignment row missing");
}

#[test]
fn test_unicode_preserved() {
    let md = export_comprehensive_markdown();
    assert!(md.contains("你好世界"), "CJK characters missing");
    assert!(md.contains("مرحبا"), "Arabic characters missing");
    assert!(md.contains("こんにちは"), "Japanese characters missing");
    assert!(md.contains("🎉"), "Emoji missing");
    assert!(md.contains("∑"), "Math symbol missing");
    assert!(md.contains("€"), "Currency symbol missing");
}

#[test]
fn test_financial_content_preserved() {
    let md = export_comprehensive_markdown();
    // These terms must appear for the search tests to work against comprehensive.docx
    assert!(md.contains("revenue"), "Search term 'revenue' missing");
    assert!(md.contains("Q4"), "Search term 'Q4' missing");
    assert!(md.contains("Executive"), "Search term 'Executive' missing");
    assert!(md.contains("$1,200,000"), "Financial figure missing");
}

#[test]
fn test_plain_paragraphs_preserved() {
    let md = export_comprehensive_markdown();
    assert!(
        md.contains("Normal paragraph with no special formatting."),
        "Plain paragraph missing"
    );
    assert!(
        md.contains("Content under a level two heading."),
        "Paragraph under heading missing"
    );
}
