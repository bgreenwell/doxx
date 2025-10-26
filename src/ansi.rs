use anyhow::Result;
use crossterm::style::{
    Attribute, Color as CrosstermColor, ResetColor, SetAttribute, SetForegroundColor,
};
use std::fmt::Write;

use crate::{document::*, ColorDepth};

pub struct AnsiOptions {
    pub terminal_width: usize,
    pub color_depth: ColorDepth,
}

impl Default for AnsiOptions {
    fn default() -> Self {
        Self {
            terminal_width: std::env::var("COLUMNS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(80),
            color_depth: ColorDepth::Auto,
        }
    }
}

pub fn export_to_ansi_with_options(document: &Document, options: &AnsiOptions) -> Result<String> {
    let mut output = String::new();

    // Add document title
    write_ansi_heading(&mut output, &document.title, 1, options)?;
    output.push('\n');

    // Add metadata
    writeln!(
        output,
        "{}Document Information{}",
        format_ansi_text("", true, false, false, false, None, options),
        format_ansi_reset()
    )?;
    writeln!(output, "- File: {}", document.metadata.file_path)?;
    writeln!(output, "- Pages: {}", document.metadata.page_count)?;
    writeln!(output, "- Words: {}", document.metadata.word_count)?;
    if let Some(author) = &document.metadata.author {
        writeln!(output, "- Author: {author}")?;
    }
    output.push('\n');

    // Separator
    let separator = "=".repeat(std::cmp::min(50, options.terminal_width));
    writeln!(output, "{separator}")?;
    output.push('\n');

    // Convert document content
    for element in &document.elements {
        match element {
            DocumentElement::Heading {
                level,
                text,
                number,
            } => {
                let heading_text = if let Some(number) = number {
                    format!("{number} {text}")
                } else {
                    text.clone()
                };
                write_ansi_heading(&mut output, &heading_text, *level, options)?;
                output.push('\n');
            }
            DocumentElement::Paragraph { runs } => {
                if runs.is_empty() || runs.iter().all(|run| run.text.trim().is_empty()) {
                    continue;
                }
                write_ansi_paragraph(&mut output, runs, options)?;
                output.push('\n');
            }
            DocumentElement::List { items, ordered } => {
                write_ansi_list(&mut output, items, *ordered, options)?;
                output.push('\n');
            }
            DocumentElement::Table { table } => {
                write_ansi_table(&mut output, table, options)?;
                output.push('\n');
            }
            DocumentElement::Image { description, .. } => {
                writeln!(
                    output,
                    "{}🖼️  [Image: {}]{}",
                    format_ansi_color(Some("#FF00FF"), options), // Magenta
                    description,
                    format_ansi_reset()
                )?;
                output.push('\n');
            }
            DocumentElement::Equation { latex, .. } => {
                writeln!(
                    output,
                    "{}📐 {}{}",
                    format_ansi_color(Some("#00AAFF"), options), // Cyan
                    latex,
                    format_ansi_reset()
                )?;
                output.push('\n');
            }
            DocumentElement::PageBreak => {
                let separator = "─".repeat(std::cmp::min(60, options.terminal_width));
                writeln!(
                    output,
                    "{}{}{}",
                    format_ansi_color(Some("#666666"), options), // Dark gray
                    separator,
                    format_ansi_reset()
                )?;
                output.push('\n');
            }
        }
    }

    Ok(output)
}

fn write_ansi_heading(
    output: &mut String,
    text: &str,
    level: u8,
    options: &AnsiOptions,
) -> Result<()> {
    let color = match level {
        1 => Some("#FFFF00"), // Yellow
        2 => Some("#00FF00"), // Green
        _ => Some("#00FFFF"), // Cyan
    };

    let prefix = match level {
        1 => "■ ",
        2 => "  ▶ ",
        3 => "    ◦ ",
        _ => "      • ",
    };

    let formatted_text = format_ansi_text(
        &format!("{prefix}{text}"),
        true,
        false,
        false,
        false,
        color,
        options,
    );

    writeln!(output, "{}{}", formatted_text, format_ansi_reset())?;
    Ok(())
}

fn write_ansi_paragraph(
    output: &mut String,
    runs: &[FormattedRun],
    options: &AnsiOptions,
) -> Result<()> {
    for run in runs {
        let formatted_text = format_ansi_text(
            &run.text,
            run.formatting.bold,
            run.formatting.italic,
            run.formatting.underline,
            run.formatting.strikethrough,
            run.formatting.color.as_deref(),
            options,
        );
        write!(output, "{formatted_text}")?;
    }
    write!(output, "{}", format_ansi_reset())?;
    writeln!(output)?;
    Ok(())
}

fn write_ansi_list(
    output: &mut String,
    items: &[ListItem],
    ordered: bool,
    options: &AnsiOptions,
) -> Result<()> {
    for (i, item) in items.iter().enumerate() {
        let bullet = if ordered {
            format!("{}. ", i + 1)
        } else {
            "• ".to_string()
        };

        let indent = "  ".repeat(item.level as usize);
        let bullet_color = format_ansi_color(Some("#0066FF"), options); // Blue

        write!(
            output,
            "{}{}{}{}",
            bullet_color,
            indent,
            bullet,
            format_ansi_reset()
        )?;

        for run in &item.runs {
            let formatted_text = format_ansi_text(
                &run.text,
                run.formatting.bold,
                run.formatting.italic,
                run.formatting.underline,
                run.formatting.strikethrough,
                run.formatting.color.as_deref(),
                options,
            );
            write!(output, "{formatted_text}")?;
        }
        write!(output, "{}", format_ansi_reset())?;
        writeln!(output)?;
    }
    Ok(())
}

fn write_ansi_table(output: &mut String, table: &TableData, options: &AnsiOptions) -> Result<()> {
    // Add table title if present
    if let Some(title) = &table.metadata.title {
        let formatted_title = format_ansi_text(
            &format!("📊 {title}"),
            true,
            false,
            false,
            false,
            Some("#0066FF"), // Blue
            options,
        );
        writeln!(output, "{}{}", formatted_title, format_ansi_reset())?;
        output.push('\n');
    }

    // Simple table rendering for ANSI
    if !table.headers.is_empty() {
        // Headers
        write!(output, "│")?;
        for header in &table.headers {
            write!(
                output,
                " {}{}{} │",
                format_ansi_text("", true, false, false, false, None, options),
                header.content,
                format_ansi_reset()
            )?;
        }
        writeln!(output)?;

        // Separator
        write!(output, "├")?;
        for _ in &table.headers {
            write!(output, "─────┼")?;
        }
        writeln!(output, "┤")?;

        // Rows
        for row in &table.rows {
            write!(output, "│")?;
            for cell in row {
                write!(output, " {} │", cell.content)?;
            }
            writeln!(output)?;
        }
    }

    Ok(())
}

fn format_ansi_text(
    text: &str,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    color: Option<&str>,
    options: &AnsiOptions,
) -> String {
    let mut result = String::new();

    // Apply formatting attributes
    if bold {
        result.push_str(&format!("{}", SetAttribute(Attribute::Bold)));
    }
    if italic {
        result.push_str(&format!("{}", SetAttribute(Attribute::Italic)));
    }
    if underline {
        result.push_str(&format!("{}", SetAttribute(Attribute::Underlined)));
    }
    if strikethrough {
        result.push_str(&format!("{}", SetAttribute(Attribute::CrossedOut)));
    }

    // Apply color
    if let Some(color_hex) = color {
        result.push_str(&format_ansi_color(Some(color_hex), options));
    }

    result.push_str(text);

    // Reset formatting after this run to prevent bleeding into subsequent runs
    result.push_str(&format_ansi_reset());

    result
}

fn format_ansi_color(color_hex: Option<&str>, options: &AnsiOptions) -> String {
    let Some(hex) = color_hex else {
        return String::new();
    };

    match convert_hex_to_crossterm_color(hex, &options.color_depth) {
        Some(color) => format!("{}", SetForegroundColor(color)),
        None => String::new(),
    }
}

fn format_ansi_reset() -> String {
    format!("{ResetColor}")
}

fn convert_hex_to_crossterm_color(hex: &str, color_depth: &ColorDepth) -> Option<CrosstermColor> {
    // Remove # if present and ensure we have 6 characters
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    // Parse RGB components
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    match color_depth {
        ColorDepth::Monochrome => None,
        ColorDepth::Standard => {
            // Convert to 16 colors (approximation)
            let color_index = rgb_to_ansi_16(r, g, b);
            Some(CrosstermColor::AnsiValue(color_index))
        }
        ColorDepth::Extended => {
            // Convert to 256 colors
            let color_index = rgb_to_ansi_256(r, g, b);
            Some(CrosstermColor::AnsiValue(color_index))
        }
        ColorDepth::TrueColor | ColorDepth::Auto => {
            // Use full RGB
            Some(CrosstermColor::Rgb { r, g, b })
        }
    }
}

fn rgb_to_ansi_16(r: u8, g: u8, b: u8) -> u8 {
    // Simple mapping to 16 colors
    let r_bright = r > 127;
    let g_bright = g > 127;
    let b_bright = b > 127;

    let base = match (r > 64, g > 64, b > 64) {
        (false, false, false) => 0, // Black
        (false, false, true) => 4,  // Blue
        (false, true, false) => 2,  // Green
        (false, true, true) => 6,   // Cyan
        (true, false, false) => 1,  // Red
        (true, false, true) => 5,   // Magenta
        (true, true, false) => 3,   // Yellow
        (true, true, true) => 7,    // White
    };

    // Add 8 for bright colors if any component is very bright
    if r_bright || g_bright || b_bright {
        base + 8
    } else {
        base
    }
}

fn rgb_to_ansi_256(r: u8, g: u8, b: u8) -> u8 {
    // 256-color conversion
    if r == g && g == b {
        // Grayscale
        if r < 8 {
            16
        } else if r > 247 {
            231
        } else {
            232 + (r - 8) / 10
        }
    } else {
        // Color cube: 16 + 36*r + 6*g + b
        let r_index = (r as f32 / 255.0 * 5.0) as u8;
        let g_index = (g as f32 / 255.0 * 5.0) as u8;
        let b_index = (b as f32 / 255.0 * 5.0) as u8;
        16 + 36 * r_index + 6 * g_index + b_index
    }
}
