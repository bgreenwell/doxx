use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};
use ratatui_image::protocol::StatefulProtocol;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::document::*;

/// Custom widget for rendering document content with proper text wrapping and inline images.
///
/// This widget handles the complete layout of document elements including:
/// - Text with unicode-aware wrapping
/// - Inline images with correct positioning
/// - Tables, lists, and other formatted content
/// - Search result highlighting
///
/// Unlike using the Paragraph widget with image overlays, this widget renders everything
/// in a single pass with full control over layout, ensuring images are positioned correctly
/// even when text wraps across multiple lines.
pub struct DocumentWidget<'a> {
    elements: &'a [DocumentElement],
    scroll_offset: usize,
    color_enabled: bool,
    search_results: &'a [SearchResult],
    image_protocols: Option<&'a mut Vec<Box<dyn StatefulProtocol>>>,
    current_search_index: usize,
}

impl<'a> DocumentWidget<'a> {
    /// Create a new DocumentWidget with the given document elements
    pub fn new(elements: &'a [DocumentElement]) -> Self {
        Self {
            elements,
            scroll_offset: 0,
            color_enabled: false,
            search_results: &[],
            image_protocols: None,
            current_search_index: 0,
        }
    }

    /// Set the scroll offset (number of elements to skip from the top)
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Enable or disable color rendering
    pub fn color_enabled(mut self, enabled: bool) -> Self {
        self.color_enabled = enabled;
        self
    }

    /// Set search results for highlighting
    pub fn search_results(mut self, results: &'a [SearchResult]) -> Self {
        self.search_results = results;
        self
    }

    /// Set the current search result index for highlighting
    pub fn current_search_index(mut self, index: usize) -> Self {
        self.current_search_index = index;
        self
    }

    /// Set image protocols for rendering images
    pub fn image_protocols(mut self, protocols: Option<&'a mut Vec<Box<dyn StatefulProtocol>>>) -> Self {
        self.image_protocols = protocols;
        self
    }

    /// Wrap formatted text runs into lines that fit within the given width.
    ///
    /// This function properly handles:
    /// - Unicode grapheme clusters (emoji, combining characters)
    /// - Preserving text formatting (bold, italic, colors) across wrapped lines
    /// - Calculating visual width correctly for all unicode characters
    fn wrap_formatted_runs(runs: &[FormattedRun], max_width: usize, color_enabled: bool) -> Vec<Line> {
        if max_width == 0 {
            return vec![];
        }

        let mut lines = Vec::new();
        let mut current_line: Vec<Span> = Vec::new();
        let mut current_width = 0;

        for run in runs {
            let mut style = Style::default();

            // Apply formatting
            if run.formatting.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if run.formatting.italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            if run.formatting.underline {
                style = style.add_modifier(Modifier::UNDERLINED);
            }

            // Apply color if enabled
            if color_enabled {
                if let Some(color_hex) = &run.formatting.color {
                    if let Some(color) = hex_to_color(color_hex) {
                        style = style.fg(color);
                    }
                }
            }

            // Split text into graphemes for proper unicode handling
            for grapheme in run.text.graphemes(true) {
                let g_width = grapheme.width();

                // Check if adding this grapheme would exceed max width
                if current_width + g_width > max_width && current_width > 0 {
                    // Finish current line and start a new one
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                        current_width = 0;
                    }
                }

                // Add grapheme to current line
                current_line.push(Span::styled(grapheme.to_string(), style));
                current_width += g_width;
            }
        }

        // Add remaining content
        if !current_line.is_empty() {
            lines.push(Line::from(current_line));
        }

        // Return at least one empty line if no content
        if lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines
    }

    /// Render a heading element at the current position
    fn render_heading(
        heading: &str,
        level: u8,
        number: Option<&str>,
        area: Rect,
        buf: &mut Buffer,
        current_y: &mut u16,
        color_enabled: bool,
    ) {
        if *current_y >= area.y + area.height {
            return; // Off screen
        }

        // Determine styling based on heading level
        let (style, prefix) = match level {
            1 => (
                if color_enabled {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                },
                "■ ",
            ),
            2 => (
                if color_enabled {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                },
                "  ▶ ",
            ),
            _ => (
                if color_enabled {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                },
                "    ◦ ",
            ),
        };

        // Build heading text with optional numbering
        let text = if let Some(num) = number {
            format!("{}{} {}", prefix, num, heading)
        } else {
            format!("{}{}", prefix, heading)
        };

        buf.set_string(area.x, *current_y, &text, style);
        *current_y += 2; // Heading + blank line
    }

    /// Render a paragraph element at the current position
    fn render_paragraph(
        runs: &[FormattedRun],
        area: Rect,
        buf: &mut Buffer,
        current_y: &mut u16,
        color_enabled: bool,
    ) {
        if *current_y >= area.y + area.height {
            return; // Off screen
        }

        // Wrap the formatted runs into lines
        let wrapped_lines = Self::wrap_formatted_runs(runs, area.width as usize, color_enabled);

        // Render each line
        for line in wrapped_lines {
            if *current_y >= area.y + area.height {
                break; // Stop if we reach bottom of area
            }

            buf.set_line(area.x, *current_y, &line, area.width);
            *current_y += 1;
        }

        *current_y += 1; // Blank line after paragraph
    }

    /// Render a list element at the current position
    fn render_list(
        items: &[ListItem],
        ordered: bool,
        area: Rect,
        buf: &mut Buffer,
        current_y: &mut u16,
        color_enabled: bool,
    ) {
        for (idx, item) in items.iter().enumerate() {
            if *current_y >= area.y + area.height {
                break; // Off screen
            }

            // Determine bullet/number prefix
            let bullet_str = if ordered {
                format!("{}. ", idx + 1)
            } else {
                "• ".to_string()
            };

            let bullet_width = bullet_str.len();
            let indent = " ".repeat(bullet_width);

            // Render bullet/number
            let bullet_style = if color_enabled {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            buf.set_string(area.x, *current_y, &bullet_str, bullet_style);

            // Wrap the item text to fit after the bullet
            let text_width = (area.width as usize).saturating_sub(bullet_width);
            let wrapped_lines = Self::wrap_formatted_runs(&item.runs, text_width, color_enabled);

            // Render first line (on same line as bullet)
            if let Some(first_line) = wrapped_lines.first() {
                buf.set_line(
                    area.x + bullet_width as u16,
                    *current_y,
                    first_line,
                    (area.width as usize - bullet_width) as u16,
                );
                *current_y += 1;
            }

            // Render remaining lines with indent
            for line in wrapped_lines.iter().skip(1) {
                if *current_y >= area.y + area.height {
                    break;
                }
                buf.set_string(area.x, *current_y, &indent, Style::default());
                buf.set_line(
                    area.x + bullet_width as u16,
                    *current_y,
                    line,
                    (area.width as usize - bullet_width) as u16,
                );
                *current_y += 1;
            }
        }

        *current_y += 1; // Blank line after list
    }
}

/// Convert hex color code to ratatui Color
fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

impl<'a> Widget for DocumentWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: Implement rendering logic
        // For now, just render a placeholder
        buf.set_string(
            area.x,
            area.y,
            "DocumentWidget rendering (in progress...)",
            Style::default(),
        );
    }
}
