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
