use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
    Frame,
};
use ratatui_image::{protocol::StatefulProtocol, StatefulImage};

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
