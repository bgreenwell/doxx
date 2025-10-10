use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Represents a mathematical equation extracted from a DOCX file
#[derive(Debug, Clone)]
pub struct Equation {
    /// Raw OMML (Office Math Markup Language) XML
    pub omml: String,
    /// Parsed and rendered Unicode representation
    pub unicode: String,
    /// Plain text fallback
    pub fallback: String,
}

/// OMML element types we need to parse
#[derive(Debug, Clone)]
enum OmmlElement {
    /// Superscript: base^exponent
    Superscript {
        base: Box<OmmlElement>,
        sup: Box<OmmlElement>,
    },
    /// Subscript: base_sub
    Subscript {
        base: Box<OmmlElement>,
        sub: Box<OmmlElement>,
    },
    /// Fraction: numerator/denominator
    Fraction {
        num: Box<OmmlElement>,
        den: Box<OmmlElement>,
    },
    /// N-ary operator (sum, integral, etc.)
    Nary {
        operator: String,
        sub: Option<Box<OmmlElement>>,
        sup: Option<Box<OmmlElement>>,
        base: Box<OmmlElement>,
    },
    /// Delimiter (parentheses, brackets, etc.)
    Delimiter { content: Box<OmmlElement> },
    /// Text run
    Text(String),
    /// Sequence of elements
    Sequence(Vec<OmmlElement>),
}

impl Equation {
    /// Parse OMML XML and convert to Unicode representation
    pub fn from_omml(omml: String) -> Result<Self> {
        let unicode = parse_omml_to_unicode(&omml)?;
        let fallback = extract_text_from_omml(&omml);

        Ok(Equation {
            omml,
            unicode,
            fallback,
        })
    }
}

/// Extract plain text from OMML (for fallback display)
fn extract_text_from_omml(omml: &str) -> String {
    let mut reader = Reader::from_str(omml);
    reader.config_mut().trim_text(true);

    let mut text = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"m:t" => {
                // Inside <m:t> tag, capture text
                if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                    text.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error parsing OMML: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    text
}

/// Convert OMML XML to Unicode mathematical representation
fn parse_omml_to_unicode(omml: &str) -> Result<String> {
    let element = parse_omml_element(omml)?;
    Ok(render_to_unicode(&element))
}

/// Parse OMML XML into structured elements
fn parse_omml_element(xml: &str) -> Result<OmmlElement> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut elements = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_ref = e.name();
                let tag_name = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");

                match tag_name {
                    "m:sSup" => {
                        // Parse superscript structure
                        elements.push(parse_superscript(&mut reader)?);
                    }
                    "m:sSub" => {
                        elements.push(parse_subscript(&mut reader)?);
                    }
                    "m:f" => {
                        elements.push(parse_fraction(&mut reader)?);
                    }
                    "m:nary" => {
                        elements.push(parse_nary(&mut reader)?);
                    }
                    "m:d" => {
                        elements.push(parse_delimiter(&mut reader)?);
                    }
                    "m:r" => {
                        elements.push(parse_run(&mut reader)?);
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("XML parsing error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    if elements.len() == 1 {
        Ok(elements.into_iter().next().unwrap())
    } else {
        Ok(OmmlElement::Sequence(elements))
    }
}

/// Parse superscript element
fn parse_superscript(reader: &mut Reader<&[u8]>) -> Result<OmmlElement> {
    let mut base = None;
    let mut sup = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_ref = e.name();
                let tag = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                match tag {
                    "m:e" => {
                        let content = read_element_content(reader, "m:e")?;
                        if base.is_none() {
                            base = Some(parse_omml_element(&content)?);
                        }
                    }
                    "m:sup" => {
                        let content = read_element_content(reader, "m:sup")?;
                        sup = Some(parse_omml_element(&content)?);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:sSup" => break,
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Superscript parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(OmmlElement::Superscript {
        base: Box::new(base.unwrap_or(OmmlElement::Text(String::new()))),
        sup: Box::new(sup.unwrap_or(OmmlElement::Text(String::new()))),
    })
}

/// Parse subscript element
fn parse_subscript(reader: &mut Reader<&[u8]>) -> Result<OmmlElement> {
    let mut base = None;
    let mut sub = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_ref = e.name();
                let tag = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                match tag {
                    "m:e" => {
                        let content = read_element_content(reader, "m:e")?;
                        if base.is_none() {
                            base = Some(parse_omml_element(&content)?);
                        }
                    }
                    "m:sub" => {
                        let content = read_element_content(reader, "m:sub")?;
                        sub = Some(parse_omml_element(&content)?);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:sSub" => break,
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Subscript parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(OmmlElement::Subscript {
        base: Box::new(base.unwrap_or(OmmlElement::Text(String::new()))),
        sub: Box::new(sub.unwrap_or(OmmlElement::Text(String::new()))),
    })
}

/// Parse fraction element
fn parse_fraction(reader: &mut Reader<&[u8]>) -> Result<OmmlElement> {
    let mut num = None;
    let mut den = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_ref = e.name();
                let tag = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                match tag {
                    "m:num" => {
                        let content = read_element_content(reader, "m:num")?;
                        num = Some(parse_omml_element(&content)?);
                    }
                    "m:den" => {
                        let content = read_element_content(reader, "m:den")?;
                        den = Some(parse_omml_element(&content)?);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:f" => break,
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Fraction parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(OmmlElement::Fraction {
        num: Box::new(num.unwrap_or(OmmlElement::Text(String::new()))),
        den: Box::new(den.unwrap_or(OmmlElement::Text(String::new()))),
    })
}

/// Parse n-ary operator (sum, integral, product, etc.)
fn parse_nary(reader: &mut Reader<&[u8]>) -> Result<OmmlElement> {
    let mut operator = String::from("∑"); // Default to summation
    let mut sub = None;
    let mut sup = None;
    let mut base = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_ref = e.name();
                let tag = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                match tag {
                    "m:chr" => {
                        // Extract operator character from attribute
                        if let Some(a) = e
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == b"m:val")
                        {
                            operator = String::from_utf8_lossy(&a.value).to_string();
                        }
                    }
                    "m:sub" => {
                        let content = read_element_content(reader, "m:sub")?;
                        sub = Some(parse_omml_element(&content)?);
                    }
                    "m:sup" => {
                        let content = read_element_content(reader, "m:sup")?;
                        sup = Some(parse_omml_element(&content)?);
                    }
                    "m:e" => {
                        let content = read_element_content(reader, "m:e")?;
                        base = Some(parse_omml_element(&content)?);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:nary" => break,
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Nary parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(OmmlElement::Nary {
        operator,
        sub: sub.map(Box::new),
        sup: sup.map(Box::new),
        base: Box::new(base.unwrap_or(OmmlElement::Text(String::new()))),
    })
}

/// Parse delimiter (parentheses, brackets, etc.)
fn parse_delimiter(reader: &mut Reader<&[u8]>) -> Result<OmmlElement> {
    let mut content = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_ref = e.name();
                let tag = std::str::from_utf8(name_ref.as_ref()).unwrap_or("");
                if tag == "m:e" {
                    let xml_content = read_element_content(reader, "m:e")?;
                    content = Some(parse_omml_element(&xml_content)?);
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:d" => break,
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Delimiter parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(OmmlElement::Delimiter {
        content: Box::new(content.unwrap_or(OmmlElement::Text(String::new()))),
    })
}

/// Parse text run
fn parse_run(reader: &mut Reader<&[u8]>) -> Result<OmmlElement> {
    let mut text = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"m:t" => {
                if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                    text.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"m:r" => break,
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Run parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(OmmlElement::Text(text))
}

/// Read content of an XML element as a string
fn read_element_content(reader: &mut Reader<&[u8]>, end_tag: &str) -> Result<String> {
    let mut content = String::new();
    let mut buf = Vec::new();
    let mut depth = 1;
    let end_tag_bytes = end_tag.as_bytes();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                content.push('<');
                content.push_str(std::str::from_utf8(e.name().as_ref()).unwrap_or(""));
                for a in e.attributes().flatten() {
                    content.push(' ');
                    content.push_str(std::str::from_utf8(a.key.as_ref()).unwrap_or(""));
                    content.push_str("=\"");
                    content.push_str(&String::from_utf8_lossy(&a.value));
                    content.push('"');
                }
                content.push('>');
                depth += 1;
            }
            Ok(Event::End(ref e)) => {
                depth -= 1;
                if depth == 0 && e.name().as_ref() == end_tag_bytes {
                    break;
                }
                content.push_str("</");
                content.push_str(std::str::from_utf8(e.name().as_ref()).unwrap_or(""));
                content.push('>');
            }
            Ok(Event::Text(ref e)) => {
                content.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Element content read error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(content)
}

/// Render parsed OMML element to Unicode string
fn render_to_unicode(element: &OmmlElement) -> String {
    match element {
        OmmlElement::Text(s) => s.clone(),
        OmmlElement::Sequence(elements) => elements.iter().map(render_to_unicode).collect(),
        OmmlElement::Superscript { base, sup } => {
            let base_str = render_to_unicode(base);
            let sup_str = render_to_unicode(sup);
            format!("{}{}", base_str, to_superscript(&sup_str))
        }
        OmmlElement::Subscript { base, sub } => {
            let base_str = render_to_unicode(base);
            let sub_str = render_to_unicode(sub);
            format!("{}{}", base_str, to_subscript(&sub_str))
        }
        OmmlElement::Fraction { num, den } => {
            let num_str = render_to_unicode(num);
            let den_str = render_to_unicode(den);
            // For simple single-char fractions, use Unicode fractions
            match (num_str.as_str(), den_str.as_str()) {
                ("1", "2") => "½".to_string(),
                ("1", "4") => "¼".to_string(),
                ("3", "4") => "¾".to_string(),
                ("1", "3") => "⅓".to_string(),
                ("2", "3") => "⅔".to_string(),
                ("1", "5") => "⅕".to_string(),
                ("1", "8") => "⅛".to_string(),
                _ => format!("({num_str}⁄{den_str})"),
            }
        }
        OmmlElement::Nary {
            operator,
            sub,
            sup,
            base,
        } => {
            let mut result = operator.clone();
            if let Some(s) = sub {
                result.push_str(&to_subscript(&render_to_unicode(s)));
            }
            if let Some(s) = sup {
                result.push_str(&to_superscript(&render_to_unicode(s)));
            }
            result.push_str(&render_to_unicode(base));
            result
        }
        OmmlElement::Delimiter { content } => {
            format!("({})", render_to_unicode(content))
        }
    }
}

/// Convert ASCII text to Unicode superscript
fn to_superscript(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            '+' => '⁺',
            '-' => '⁻',
            '=' => '⁼',
            '(' => '⁽',
            ')' => '⁾',
            'n' => 'ⁿ',
            'i' => 'ⁱ',
            _ => c, // Keep other characters as-is
        })
        .collect()
}

/// Convert ASCII text to Unicode subscript
fn to_subscript(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            '+' => '₊',
            '-' => '₋',
            '=' => '₌',
            '(' => '₍',
            ')' => '₎',
            'a' => 'ₐ',
            'e' => 'ₑ',
            'h' => 'ₕ',
            'i' => 'ᵢ',
            'j' => 'ⱼ',
            'k' => 'ₖ',
            'l' => 'ₗ',
            'm' => 'ₘ',
            'n' => 'ₙ',
            'o' => 'ₒ',
            'p' => 'ₚ',
            'r' => 'ᵣ',
            's' => 'ₛ',
            't' => 'ₜ',
            'u' => 'ᵤ',
            'v' => 'ᵥ',
            'x' => 'ₓ',
            _ => c, // Keep other characters as-is
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superscript_conversion() {
        assert_eq!(to_superscript("2"), "²");
        assert_eq!(to_superscript("n"), "ⁿ");
        assert_eq!(to_superscript("10"), "¹⁰");
    }

    #[test]
    fn test_subscript_conversion() {
        assert_eq!(to_subscript("0"), "₀");
        assert_eq!(to_subscript("k"), "ₖ");
        assert_eq!(to_subscript("n-k"), "ₙ₋ₖ");
    }

    #[test]
    fn test_simple_fraction() {
        let omml = r#"<m:f><m:num><m:r><m:t>1</m:t></m:r></m:num><m:den><m:r><m:t>2</m:t></m:r></m:den></m:f>"#;
        let eq = Equation::from_omml(omml.to_string()).unwrap();
        assert_eq!(eq.unicode, "½");
    }
}
