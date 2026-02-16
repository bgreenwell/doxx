use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};
use ratatui_image::{protocol::StatefulProtocol, StatefulImage};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::LayoutCache;
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

    /// Wrap formatted text runs into lines that fit within the given width.
    ///
    /// This function properly handles:
    /// - Unicode grapheme clusters (emoji, combining characters)
    /// - Preserving text formatting (bold, italic, colors) across wrapped lines
    /// - Calculating visual width correctly for all unicode characters
    /// - Search result highlighting (current match and other matches)
    fn wrap_formatted_runs(
        runs: &[FormattedRun],
        max_width: usize,
        color_enabled: bool,
        search_matches: &[(usize, usize)], // List of (start_pos, end_pos) for matches in this element
        is_current_match: bool,            // True if this element contains the current search match
    ) -> Vec<Line<'static>> {
        if max_width == 0 {
            return vec![];
        }

        let mut lines = Vec::new();
        let mut current_line: Vec<Span> = Vec::new();
        let mut current_width = 0;
        let mut char_position = 0; // Track absolute character position across all runs

        for run in runs {
            let mut base_style = Style::default();

            // Apply formatting
            if run.formatting.bold {
                base_style = base_style.add_modifier(Modifier::BOLD);
            }
            if run.formatting.italic {
                base_style = base_style.add_modifier(Modifier::ITALIC);
            }
            if run.formatting.underline {
                base_style = base_style.add_modifier(Modifier::UNDERLINED);
            }
            if run.formatting.strikethrough {
                base_style = base_style.add_modifier(Modifier::CROSSED_OUT);
            }

            // Apply color if enabled
            if color_enabled {
                if let Some(color_hex) = &run.formatting.color {
                    if let Some(color) = hex_to_color(color_hex) {
                        base_style = base_style.fg(color);
                    }
                }
            }

            // Split text into graphemes for proper unicode handling
            for grapheme in run.text.graphemes(true) {
                let g_width = grapheme.width();

                // Determine if this character is within a search match
                let mut style = base_style;
                for &(start_pos, end_pos) in search_matches {
                    if char_position >= start_pos && char_position < end_pos {
                        // Apply search highlight
                        if is_current_match {
                            // Current match: bright yellow background
                            style = style.bg(Color::Yellow).fg(Color::Black);
                        } else {
                            // Other matches: darker highlight
                            style = style.bg(Color::Indexed(240)); // Dark gray
                        }
                        break;
                    }
                }

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
                char_position += grapheme.chars().count(); // Advance character position
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
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                },
                "■ ",
            ),
            2 => (
                if color_enabled {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                },
                "  ▶ ",
            ),
            _ => (
                if color_enabled {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                },
                "    ◦ ",
            ),
        };

        // Build heading text with optional numbering
        let text = if let Some(num) = number {
            format!("{prefix}{num} {heading}")
        } else {
            format!("{prefix}{heading}")
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
        search_matches: &[(usize, usize)],
        is_current_match: bool,
        element_index: usize,
        layout_cache: &mut LayoutCache,
    ) {
        if *current_y >= area.y + area.height {
            return; // Off screen
        }

        // Try to get cached lines, or wrap if not cached
        let wrapped_lines = if search_matches.is_empty() {
            // Only use cache if there are no search matches (search highlighting changes the output)
            if let Some(cached) = layout_cache.get(element_index, area.width) {
                cached.clone()
            } else {
                let lines = Self::wrap_formatted_runs(
                    runs,
                    area.width as usize,
                    color_enabled,
                    search_matches,
                    is_current_match,
                );
                layout_cache.insert(element_index, area.width, lines.clone());
                lines
            }
        } else {
            // Don't cache when search highlighting is active
            Self::wrap_formatted_runs(
                runs,
                area.width as usize,
                color_enabled,
                search_matches,
                is_current_match,
            )
        };

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
        search_matches: &[(usize, usize)],
        is_current_match: bool,
        _element_index: usize,
        _layout_cache: &mut LayoutCache,
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
            let wrapped_lines = Self::wrap_formatted_runs(
                &item.runs,
                text_width,
                color_enabled,
                search_matches,
                is_current_match,
            );

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

    /// Render a table element at the current position
    fn render_table(
        table: &TableData,
        area: Rect,
        buf: &mut Buffer,
        current_y: &mut u16,
        color_enabled: bool,
    ) {
        if *current_y >= area.y + area.height {
            return; // Off screen
        }

        let available_width = area.width as usize;

        // Calculate column widths based on metadata
        let col_widths = &table.metadata.column_widths;
        let total_width: usize = col_widths.iter().sum();

        // Scale widths to fit available space
        let scaled_widths: Vec<usize> = if total_width > available_width {
            col_widths
                .iter()
                .map(|w| (w * available_width) / total_width.max(1))
                .collect()
        } else {
            col_widths.clone()
        };

        // Render title if present
        if let Some(title) = &table.metadata.title {
            let title_style = if color_enabled {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
            buf.set_string(area.x, *current_y, title, title_style);
            *current_y += 1;
        }

        // Render headers if present
        if table.metadata.has_headers && !table.headers.is_empty() {
            Self::render_table_row(
                &table.headers,
                &scaled_widths,
                area,
                buf,
                current_y,
                color_enabled,
                true,
            );

            // Header separator line
            if *current_y < area.y + area.height {
                let separator = "─".repeat(available_width.min(scaled_widths.iter().sum()));
                buf.set_string(area.x, *current_y, &separator, Style::default());
                *current_y += 1;
            }
        }

        // Render rows
        for row in &table.rows {
            if *current_y >= area.y + area.height {
                break;
            }
            Self::render_table_row(
                row,
                &scaled_widths,
                area,
                buf,
                current_y,
                color_enabled,
                false,
            );
        }

        *current_y += 1; // Blank line after table
    }

    /// Render a single table row
    fn render_table_row(
        cells: &[TableCell],
        col_widths: &[usize],
        area: Rect,
        buf: &mut Buffer,
        current_y: &mut u16,
        color_enabled: bool,
        is_header: bool,
    ) {
        if *current_y >= area.y + area.height {
            return;
        }

        let mut x_offset = 0;

        for (i, cell) in cells.iter().enumerate() {
            let width = col_widths.get(i).copied().unwrap_or(10);

            // Apply cell styling
            let mut style = Style::default();
            if is_header {
                style = style.add_modifier(Modifier::BOLD);
                if color_enabled {
                    style = style.fg(Color::Yellow);
                }
            } else if color_enabled {
                if let Some(color_hex) = &cell.formatting.color {
                    if let Some(color) = hex_to_color(color_hex) {
                        style = style.fg(color);
                    }
                }
            }

            // Apply cell formatting
            if cell.formatting.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if cell.formatting.italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            if cell.formatting.underline {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            if cell.formatting.strikethrough {
                style = style.add_modifier(Modifier::CROSSED_OUT);
            }

            // Truncate content to fit width
            let content = if cell.content.len() > width {
                format!("{}…", &cell.content[..width.saturating_sub(1)])
            } else {
                cell.content.clone()
            };

            // Apply alignment
            let aligned_content = match cell.alignment {
                TextAlignment::Left => format!("{content:<width$}"),
                TextAlignment::Right => format!("{content:>width$}"),
                TextAlignment::Center => {
                    let padding = width.saturating_sub(content.len());
                    let left_pad = padding / 2;
                    let right_pad = padding - left_pad;
                    format!(
                        "{}{}{}",
                        " ".repeat(left_pad),
                        content,
                        " ".repeat(right_pad)
                    )
                }
                TextAlignment::Justify => format!("{content:<width$}"),
            };

            buf.set_string(
                area.x + x_offset as u16,
                *current_y,
                &aligned_content,
                style,
            );

            x_offset += width + 1; // +1 for column separator

            // Render column separator
            if i < cells.len() - 1 && x_offset < area.width as usize {
                buf.set_string(
                    area.x + x_offset as u16 - 1,
                    *current_y,
                    "│",
                    Style::default(),
                );
            }
        }

        *current_y += 1;
    }

    /// Render an image placeholder (actual image rendering happens in main render loop)
    fn render_image_placeholder(
        description: &str,
        area: Rect,
        buf: &mut Buffer,
        current_y: &mut u16,
        color_enabled: bool,
        image_height: u16,
    ) {
        if *current_y >= area.y + area.height {
            return;
        }

        // Reserve space for the image
        *current_y += image_height;

        // Render description below the image space
        if *current_y < area.y + area.height {
            let desc_style = if color_enabled {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default()
            };
            let desc_text = format!("🖼️  {description}");
            buf.set_string(area.x, *current_y, &desc_text, desc_style);
            *current_y += 2; // Description + blank line
        }
    }

    /// Render a page break element
    fn render_page_break(area: Rect, buf: &mut Buffer, current_y: &mut u16, color_enabled: bool) {
        if *current_y >= area.y + area.height {
            return;
        }

        let style = if color_enabled {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let separator = "─".repeat(area.width as usize);
        buf.set_string(area.x, *current_y, &separator, style);
        *current_y += 2; // Page break + blank line
    }

    /// Custom render method that has access to Frame for complete rendering.
    ///
    /// This method renders all document elements including text (with wrapping) and images.
    /// Unlike the Widget trait's render method, this has access to Frame which is required
    /// for rendering StatefulImage widgets.
    pub fn render(
        &mut self,
        area: Rect,
        frame: &mut Frame,
        image_protocols: &mut [StatefulProtocol],
        layout_cache: &mut LayoutCache,
    ) {
        let buf = frame.buffer_mut();

        // Check if terminal width changed and invalidate cache if needed
        layout_cache.check_width(area.width);

        // Start rendering from the top of the area
        let mut current_y = area.y;

        // Skip elements based on scroll offset
        let visible_elements = self.elements.iter().enumerate().skip(self.scroll_offset);

        // Track image positions and protocol indices for rendering
        let mut images_to_render: Vec<(u16, usize)> = Vec::new(); // (y_position, protocol_index)
        let mut protocol_idx = 0;

        // Render each visible element
        for (element_index, element) in visible_elements {
            // Stop if we've reached the bottom of the area
            if current_y >= area.y + area.height {
                break;
            }

            // Extract search matches for this element
            let search_matches: Vec<(usize, usize)> = self
                .search_results
                .iter()
                .filter(|result| result.element_index == element_index)
                .map(|result| (result.start_pos, result.end_pos))
                .collect();

            // Check if this element contains the current search match
            let is_current_match = !self.search_results.is_empty()
                && self.current_search_index < self.search_results.len()
                && self.search_results[self.current_search_index].element_index == element_index;

            match element {
                DocumentElement::Heading {
                    level,
                    text,
                    number,
                } => {
                    Self::render_heading(
                        text,
                        *level,
                        number.as_deref(),
                        area,
                        buf,
                        &mut current_y,
                        self.color_enabled,
                    );
                }

                DocumentElement::Paragraph { runs } => {
                    Self::render_paragraph(
                        runs,
                        area,
                        buf,
                        &mut current_y,
                        self.color_enabled,
                        &search_matches,
                        is_current_match,
                        element_index,
                        layout_cache,
                    );
                }

                DocumentElement::List { items, ordered } => {
                    Self::render_list(
                        items,
                        *ordered,
                        area,
                        buf,
                        &mut current_y,
                        self.color_enabled,
                        &search_matches,
                        is_current_match,
                        element_index,
                        layout_cache,
                    );
                }

                DocumentElement::Table { table } => {
                    Self::render_table(table, area, buf, &mut current_y, self.color_enabled);
                }

                DocumentElement::Image {
                    description,
                    image_path,
                    ..
                } => {
                    // Check if we can render this image
                    if image_path.is_some() && protocol_idx < image_protocols.len() {
                        // Store image position for rendering after text
                        let image_y = current_y;
                        images_to_render.push((image_y, protocol_idx));

                        // Reserve space for the image
                        Self::render_image_placeholder(
                            description,
                            area,
                            buf,
                            &mut current_y,
                            self.color_enabled,
                            15, // Standard image height
                        );

                        protocol_idx += 1;
                    } else {
                        // Render text-only placeholder
                        let status = if image_path.is_some() {
                            " [Image available - use --images flag]"
                        } else {
                            " [Image not extracted]"
                        };
                        let desc_text = format!("🖼️  {description}{status}");
                        buf.set_string(area.x, current_y, &desc_text, Style::default());
                        current_y += 2;
                    }
                }

                DocumentElement::Equation { latex, .. } => {
                    if current_y >= area.y + area.height {
                        continue;
                    }

                    // Render equation as formatted text with icon
                    let icon_style = if self.color_enabled {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    };

                    let latex_style = if self.color_enabled {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().add_modifier(Modifier::BOLD)
                    };

                    // Build the equation line
                    let line = Line::from(vec![
                        Span::styled("📐 ", icon_style),
                        Span::styled(latex, latex_style),
                    ]);

                    buf.set_line(area.x, current_y, &line, area.width);
                    current_y += 2; // Equation + blank line
                }

                DocumentElement::PageBreak => {
                    Self::render_page_break(area, buf, &mut current_y, self.color_enabled);
                }
            }
        }

        // Now render all images using Frame (after text has been rendered to buffer)
        for (y_pos, proto_idx) in images_to_render {
            if let Some(protocol) = image_protocols.get_mut(proto_idx) {
                // Ensure image is within visible area
                if y_pos < area.y + area.height {
                    let img_rect = Rect {
                        x: area.x,
                        y: y_pos,
                        width: area.width.min(80),
                        height: 15.min(area.y + area.height - y_pos),
                    };

                    let image_widget = StatefulImage::new();
                    frame.render_stateful_widget(image_widget, img_rect, protocol);
                }
            }
        }
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
