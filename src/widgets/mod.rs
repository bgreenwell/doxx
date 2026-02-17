mod document;

use ratatui::text::Line;
use std::collections::HashMap;

pub use document::DocumentWidget;

/// Cache for wrapped text lines to avoid re-wrapping on every frame
#[derive(Debug, Default)]
pub struct LayoutCache {
    /// Cached wrapped lines: (element_index, terminal_width) -> Vec<Line>
    cache: HashMap<(usize, u16), Vec<Line<'static>>>,
    /// Last known terminal width for invalidation
    last_width: u16,
}

impl LayoutCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            last_width: 0,
        }
    }

    /// Get cached lines for an element, if available
    pub fn get(&self, element_index: usize, width: u16) -> Option<&Vec<Line<'static>>> {
        self.cache.get(&(element_index, width))
    }

    /// Store wrapped lines for an element
    pub fn insert(&mut self, element_index: usize, width: u16, lines: Vec<Line<'static>>) {
        self.cache.insert((element_index, width), lines);
    }

    /// Invalidate cache if terminal width changed
    pub fn check_width(&mut self, width: u16) {
        if width != self.last_width {
            self.cache.clear();
            self.last_width = width;
        }
    }
}
