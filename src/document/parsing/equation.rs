//! Equation extraction and OMML to LaTeX conversion
//!
//! This module handles extraction of mathematical equations from Word documents
//! and conversion from OMML (Office Math Markup Language) to LaTeX format.

use anyhow::Result;
use std::path::Path;

/// Equation type and context information
#[derive(Debug, Clone)]
pub(crate) struct EquationInfo {
    pub(crate) latex: String,
    pub(crate) fallback: String,
    pub(crate) is_inline: bool,
    pub(crate) paragraph_index: usize,
}

/// Represents content within a paragraph (text or inline equation)
#[derive(Debug, Clone)]
pub(crate) enum ParagraphContent {
    Text(String),
    #[allow(dead_code)] // fallback may be used for UI display in future
    InlineEquation {
        latex: String,
        fallback: String,
    },
}

/// Parse paragraphs with inline equations directly from XML
/// Returns a map of paragraph index to ordered content (text and inline equations)
pub(crate) fn extract_inline_equation_positions(
    file_path: &Path,
) -> Result<std::collections::HashMap<usize, Vec<ParagraphContent>>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use std::fs::File;
    use std::io::Read;
    use zip::ZipArchive;

    let file = File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read word/document.xml
    let mut document_xml = String::new();
    let mut xml_file = archive.by_name("word/document.xml")?;
    xml_file.read_to_string(&mut document_xml)?;

    let mut paragraphs: std::collections::HashMap<usize, Vec<ParagraphContent>> =
        std::collections::HashMap::new();
    let mut reader = Reader::from_str(&document_xml);
    reader.config_mut().trim_text(false); // Don't trim to preserve spacing

    let mut buf = Vec::new();
    let mut in_paragraph = false;
    let mut in_math = false;
    let mut in_math_para = false; // Track if we're in a display equation
    let mut in_text_run = false;
    let mut current_paragraph_index = 0;
    let mut current_paragraph_content: Vec<ParagraphContent> = Vec::new();
    let mut current_text = String::new();
    let mut current_omml = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                in_paragraph = true;
                current_paragraph_index += 1;
                current_paragraph_content.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:p" => {
                in_paragraph = false;
                if !current_paragraph_content.is_empty() {
                    paragraphs.insert(current_paragraph_index, current_paragraph_content.clone());
                }
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"m:oMathPara" => {
                in_math_para = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:oMathPara" => {
                in_math_para = false;
            }
            Ok(Event::Start(ref e))
                if e.name().as_ref() == b"m:oMath" && in_paragraph && !in_math_para =>
            {
                // Inline equation (not wrapped in oMathPara)
                in_math = true;
                current_omml.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:oMath" && in_math => {
                in_math = false;
                let (latex, fallback) = parse_simple_omml(&current_omml);
                current_paragraph_content
                    .push(ParagraphContent::InlineEquation { latex, fallback });
                current_omml.clear();
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" && in_paragraph && !in_math => {
                in_text_run = true;
                current_text.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" && in_text_run => {
                in_text_run = false;
                if !current_text.is_empty() {
                    current_paragraph_content.push(ParagraphContent::Text(current_text.clone()));
                }
            }
            Ok(Event::Text(ref e)) if in_text_run => {
                current_text.push_str(&e.unescape().unwrap_or_default());
            }
            // Capture OMML content for inline equations
            Ok(Event::Start(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push('<');
                current_omml.push_str(tag_name);
                for a in e.attributes().flatten() {
                    let key = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                    let value = String::from_utf8_lossy(&a.value);
                    current_omml.push(' ');
                    current_omml.push_str(key);
                    current_omml.push_str("=\"");
                    current_omml.push_str(&value);
                    current_omml.push('"');
                }
                current_omml.push('>');
            }
            Ok(Event::End(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push_str("</");
                current_omml.push_str(tag_name);
                current_omml.push('>');
            }
            Ok(Event::Empty(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push('<');
                current_omml.push_str(tag_name);
                for a in e.attributes().flatten() {
                    let key = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                    let value = String::from_utf8_lossy(&a.value);
                    current_omml.push(' ');
                    current_omml.push_str(key);
                    current_omml.push_str("=\"");
                    current_omml.push_str(&value);
                    current_omml.push('"');
                }
                current_omml.push_str("/>");
            }
            Ok(Event::Text(ref e)) if in_math => {
                current_omml.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error reading XML for inline equations: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(paragraphs)
}

/// Extract equations from .docx file by reading raw XML
/// Since docx-rs doesn't expose OMML (Office Math Markup Language), we parse the ZIP directly
pub(crate) fn extract_equations_from_docx(file_path: &Path) -> Result<Vec<EquationInfo>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use std::fs::File;
    use std::io::Read;
    use zip::ZipArchive;

    let file = File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read word/document.xml
    let mut document_xml = String::new();
    let mut xml_file = archive.by_name("word/document.xml")?;
    xml_file.read_to_string(&mut document_xml)?;

    let mut equations = Vec::new();
    let mut reader = Reader::from_str(&document_xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut in_math = false;
    let mut in_math_para = false;
    let mut current_omml = String::new();
    let mut current_paragraph_index = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                current_paragraph_index += 1;
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"m:oMathPara" => {
                in_math_para = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:oMathPara" => {
                in_math_para = false;
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"m:oMath" => {
                in_math = true;
                current_omml.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:oMath" => {
                in_math = false;

                // Parse the collected OMML to LaTeX
                let (latex, fallback) = parse_simple_omml(&current_omml);

                // Inline equations are NOT wrapped in <m:oMathPara>
                let is_inline = !in_math_para;

                equations.push(EquationInfo {
                    latex,
                    fallback,
                    is_inline,
                    paragraph_index: current_paragraph_index,
                });
                current_omml.clear();
            }
            Ok(Event::Start(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push('<');
                current_omml.push_str(tag_name);

                // Capture attributes (e.g., m:chr m:val="∑")
                for a in e.attributes().flatten() {
                    let key = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                    let value = String::from_utf8_lossy(&a.value);
                    current_omml.push(' ');
                    current_omml.push_str(key);
                    current_omml.push_str("=\"");
                    current_omml.push_str(&value);
                    current_omml.push('"');
                }
                current_omml.push('>');
            }
            Ok(Event::End(ref e)) if in_math => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push_str("</");
                current_omml.push_str(tag_name);
                current_omml.push('>');
            }
            Ok(Event::Empty(ref e)) if in_math => {
                // Handle self-closing tags like <m:type m:val="noBar"/>
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                current_omml.push('<');
                current_omml.push_str(tag_name);

                // Capture attributes
                for a in e.attributes().flatten() {
                    let key = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                    let value = String::from_utf8_lossy(&a.value);
                    current_omml.push(' ');
                    current_omml.push_str(key);
                    current_omml.push_str("=\"");
                    current_omml.push_str(&value);
                    current_omml.push('"');
                }
                current_omml.push_str("/>");
            }
            Ok(Event::Text(ref e)) if in_math => {
                current_omml.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error reading OMML: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(equations)
}

/// OMML parser that converts to LaTeX format
fn parse_simple_omml(omml: &str) -> (String, String) {
    // Extract plain text for fallback
    let fallback = omml
        .split("<m:t>")
        .skip(1)
        .filter_map(|s| s.split("</m:t>").next())
        .collect::<Vec<_>>()
        .join("");

    let latex = omml_to_latex(omml);

    if latex.is_empty() {
        (fallback.clone(), fallback)
    } else {
        (latex, fallback)
    }
}

/// Convert OMML XML to LaTeX
fn omml_to_latex(omml: &str) -> String {
    let mut result = String::new();
    let mut i = 0;

    while i < omml.len() {
        // Look for OMML structures
        if omml[i..].starts_with("<m:sSup>") {
            // Superscript: ^{...}
            let end = omml[i..].find("</m:sSup>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let (Some(base), Some(sup)) = (
                extract_latex_text(content, "m:e"),
                extract_latex_text(content, "m:sup"),
            ) {
                result.push_str(&base);
                result.push_str("^{");
                result.push_str(&sup);
                result.push('}');
            }
            i += end + 8;
        } else if omml[i..].starts_with("<m:sSub>") {
            // Subscript: _{...}
            let end = omml[i..].find("</m:sSub>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let (Some(base), Some(sub)) = (
                extract_latex_text(content, "m:e"),
                extract_latex_text(content, "m:sub"),
            ) {
                result.push_str(&base);
                result.push_str("_{");
                result.push_str(&sub);
                result.push('}');
            }
            i += end + 8;
        } else if omml[i..].starts_with("<m:sSub") && omml[i..].starts_with("<m:sSubSup>") {
            // Subscript and superscript: _{}^{}
            let end = omml[i..].find("</m:sSubSup>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let (Some(base), Some(sub), Some(sup)) = (
                extract_latex_text(content, "m:e"),
                extract_latex_text(content, "m:sub"),
                extract_latex_text(content, "m:sup"),
            ) {
                result.push_str(&base);
                result.push_str("_{");
                result.push_str(&sub);
                result.push_str("}^{");
                result.push_str(&sup);
                result.push('}');
            }
            i += end + 12;
        } else if omml[i..].starts_with("<m:d>") {
            // Delimiter: \left(...\right)
            let end = omml[i..].find("</m:d>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            result.push_str("\\left(");
            if let Some(inner) = extract_latex_text(content, "m:e") {
                result.push_str(&inner);
            }
            result.push_str("\\right)");
            i += end + 5;
        } else if omml[i..].starts_with("<m:f>") {
            // Fraction: \frac{num}{den} or binomial coefficient: \binom{n}{k}
            let end = omml[i..].find("</m:f>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            // Check if it's a binomial coefficient (noBar type)
            let is_binom = content.contains("m:val=\"noBar\"");

            if let (Some(num), Some(den)) = (
                extract_latex_text(content, "m:num"),
                extract_latex_text(content, "m:den"),
            ) {
                if is_binom {
                    result.push_str("\\binom{");
                    result.push_str(&num);
                    result.push_str("}{");
                    result.push_str(&den);
                    result.push('}');
                } else {
                    result.push_str("\\frac{");
                    result.push_str(&num);
                    result.push_str("}{");
                    result.push_str(&den);
                    result.push('}');
                }
            }
            i += end + 5;
        } else if omml[i..].starts_with("<m:func>") {
            // Function: \sin, \cos, \tan, etc.
            let end = omml[i..].find("</m:func>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let Some(func_name) = extract_latex_text(content, "m:fName") {
                result.push('\\');
                result.push_str(&func_name);
            }
            if let Some(argument) = extract_latex_text(content, "m:e") {
                result.push(' ');
                result.push_str(&argument);
            }
            i += end + 8;
        } else if omml[i..].starts_with("<m:rad>") {
            // Radical (square root): \sqrt{...} or \sqrt[n]{...}
            let end = omml[i..].find("</m:rad>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            result.push_str("\\sqrt");
            // Check for degree (nth root)
            if let Some(deg) = extract_latex_text(content, "m:deg") {
                if deg != "2" && !deg.is_empty() {
                    result.push('[');
                    result.push_str(&deg);
                    result.push(']');
                }
            }
            result.push('{');
            if let Some(base) = extract_latex_text(content, "m:e") {
                result.push_str(&base);
            }
            result.push('}');
            i += end + 7;
        } else if omml[i..].starts_with("<m:nary") {
            // N-ary operator: \sum_{...}^{...}, \int_{...}^{...}, etc.
            let end = omml[i..].find("</m:nary>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            // Extract operator character and convert to LaTeX command
            let operator = if let Some(chr_pos) = content.find("m:val=\"") {
                let start = chr_pos + 7;
                if let Some(end_quote) = content[start..].find('"') {
                    let chr = &content[start..start + end_quote];
                    match chr {
                        "∑" => "\\sum",
                        "∫" => "\\int",
                        "∬" => "\\iint",
                        "∭" => "\\iiint",
                        "∮" => "\\oint",
                        "∏" => "\\prod",
                        "⋃" => "\\bigcup",
                        "⋂" => "\\bigcap",
                        _ => "\\sum",
                    }
                } else {
                    "\\sum"
                }
            } else {
                "\\sum"
            };

            result.push_str(operator);

            // Extract sub and sup
            if let Some(sub) = extract_latex_text(content, "m:sub") {
                result.push_str("_{");
                result.push_str(&sub);
                result.push('}');
            }
            if let Some(sup) = extract_latex_text(content, "m:sup") {
                result.push_str("^{");
                result.push_str(&sup);
                result.push('}');
            }
            if let Some(base) = extract_latex_text(content, "m:e") {
                result.push(' ');
                result.push_str(&base);
            }

            i += end + 9;
        } else if omml[i..].starts_with("<m:r>") {
            // Text run - extract text without processing
            let end = omml[i..].find("</m:r>").unwrap_or(omml.len() - i);
            let content = &omml[i..i + end];

            if let Some(text) = extract_text(content, "m:t") {
                // Convert special characters to LaTeX
                for ch in text.chars() {
                    match ch {
                        'π' => result.push_str("\\pi "),
                        'α' => result.push_str("\\alpha "),
                        'β' => result.push_str("\\beta "),
                        'γ' => result.push_str("\\gamma "),
                        'Γ' => result.push_str("\\Gamma "),
                        'δ' => result.push_str("\\delta "),
                        'Δ' => result.push_str("\\Delta "),
                        'θ' => result.push_str("\\theta "),
                        'λ' => result.push_str("\\lambda "),
                        'μ' => result.push_str("\\mu "),
                        'σ' => result.push_str("\\sigma "),
                        'Σ' => result.push_str("\\Sigma "),
                        'φ' => result.push_str("\\phi "),
                        'ω' => result.push_str("\\omega "),
                        'Ω' => result.push_str("\\Omega "),
                        '∞' => result.push_str("\\infty "),
                        '±' => result.push_str("\\pm "),
                        '×' => result.push_str("\\times "),
                        '÷' => result.push_str("\\div "),
                        '≤' => result.push_str("\\leq "),
                        '≥' => result.push_str("\\geq "),
                        '≠' => result.push_str("\\neq "),
                        '≈' => result.push_str("\\approx "),
                        '∈' => result.push_str("\\in "),
                        '∉' => result.push_str("\\notin "),
                        '⊂' => result.push_str("\\subset "),
                        '⊃' => result.push_str("\\supset "),
                        '∪' => result.push_str("\\cup "),
                        '∩' => result.push_str("\\cap "),
                        '∅' => result.push_str("\\emptyset "),
                        '√' => result.push_str("\\sqrt"),
                        _ => result.push(ch),
                    }
                }
            }
            i += end + 5;
        } else if omml[i..].starts_with("<m:t>") {
            // Text content
            let end = omml[i + 4..].find("</m:t>").unwrap_or(omml.len() - i - 4);
            let text = &omml[i + 4..i + 4 + end];
            // Convert special characters
            for ch in text.chars() {
                match ch {
                    'π' => result.push_str("\\pi"),
                    'α' => result.push_str("\\alpha"),
                    'β' => result.push_str("\\beta"),
                    _ => result.push(ch),
                }
            }
            i += 4 + end + 5;
        } else {
            i += 1;
        }
    }

    result
}

/// Extract text from an OMML tag and recursively convert nested OMML to LaTeX
fn extract_latex_text(omml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    if let Some(start_pos) = omml.find(&start_tag) {
        let content = &omml[start_pos + start_tag.len()..];

        // Find the matching closing tag, accounting for nesting
        let mut depth = 1;
        let mut pos = 0;
        let mut end_pos = None;

        while pos < content.len() && depth > 0 {
            if content[pos..].starts_with(&start_tag) {
                depth += 1;
                pos += start_tag.len();
            } else if content[pos..].starts_with(&end_tag) {
                depth -= 1;
                if depth == 0 {
                    end_pos = Some(pos);
                    break;
                }
                pos += end_tag.len();
            } else {
                // Skip to next character boundary (Unicode-safe)
                let next_char = content[pos..].chars().next();
                if let Some(ch) = next_char {
                    pos += ch.len_utf8();
                } else {
                    break;
                }
            }
        }

        if let Some(end_pos) = end_pos {
            let inner = &content[..end_pos];

            // Check if inner content has OMML structures
            if inner.contains("<m:") {
                // Recursively convert nested OMML to LaTeX
                return Some(omml_to_latex(inner));
            } else {
                // Extract plain text from <m:t> tags
                let text = inner
                    .split("<m:t>")
                    .skip(1)
                    .filter_map(|s| s.split("</m:t>").next())
                    .collect::<Vec<_>>()
                    .join("");

                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

/// Extract text from an OMML tag
fn extract_text(omml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    if let Some(start_pos) = omml.find(&start_tag) {
        let content = &omml[start_pos + start_tag.len()..];
        if let Some(end_pos) = content.find(&end_tag) {
            let inner = &content[..end_pos];

            // Inner is already the text between <tag> and </tag>, just return it
            if !inner.is_empty() {
                return Some(inner.to_string());
            }
        }
    }
    None
}
