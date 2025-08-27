use anyhow::Result;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Theme configuration for doxx
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// UI chrome colors
    pub ui: UiTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTheme {
    /// Main document view
    pub document_border: String,
    pub document_title: String,

    /// Footer area (unified background for entire footer)
    pub footer_bg: String,
    pub status_bar_fg: String,
    pub status_message_fg: String,
    pub help_bar_fg: String,

    /// Search interface
    pub search_border: String,
    pub search_input: String,
    pub search_match_bg: String,
    pub search_match_fg: String,

    /// Outline view
    pub outline_border: String,
    pub outline_item: String,
    pub outline_selected_bg: String,
    pub outline_selected_fg: String,

    /// Help view
    pub help_border: String,
    pub help_text: String,

    /// Scrollbar
    pub scrollbar: String,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            ui: UiTheme {
                document_border: "#6495ED".to_string(), // Cornflower Blue
                document_title: "#FFFFFF".to_string(),  // White

                footer_bg: "#2F2F2F".to_string(),     // Dark Gray
                status_bar_fg: "#FFFFFF".to_string(), // White
                status_message_fg: "#90EE90".to_string(), // Light Green
                help_bar_fg: "#A0A0A0".to_string(),   // Light Gray

                search_border: "#FFD700".to_string(), // Gold/Yellow
                search_input: "#FFD700".to_string(),  // Gold/Yellow
                search_match_bg: "#FFD700".to_string(), // Gold/Yellow
                search_match_fg: "#000000".to_string(), // Black

                outline_border: "#90EE90".to_string(), // Light Green
                outline_item: "#FFFFFF".to_string(),   // White
                outline_selected_bg: "#6495ED".to_string(), // Cornflower Blue
                outline_selected_fg: "#FFFFFF".to_string(), // White

                help_border: "#FFD700".to_string(), // Gold/Yellow
                help_text: "#FFFFFF".to_string(),   // White

                scrollbar: "#808080".to_string(), // Gray
            },
        }
    }
}

impl Theme {
    /// Load theme from config directory
    pub fn load() -> Result<Self> {
        if let Some(config_path) = Self::get_config_path() {
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                let theme: Theme = toml::from_str(&content)?;
                return Ok(theme);
            }
        }

        // Return default theme if no config found
        Ok(Theme::default())
    }

    /// Save theme to config directory
    pub fn save(&self) -> Result<()> {
        if let Some(config_path) = Self::get_config_path() {
            // Create config directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let content = toml::to_string_pretty(self)?;
            fs::write(&config_path, content)?;
        }

        Ok(())
    }

    /// Get the path to the theme config file
    pub fn get_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("doxx").join("theme.toml"))
    }

    /// Initialize default theme file
    pub fn init_default() -> Result<()> {
        let theme = Theme::default();
        theme.save()?;
        Ok(())
    }

    /// Convert hex color string to ratatui Color
    pub fn hex_to_color(hex: &str) -> Option<Color> {
        // Remove # if present
        let hex = hex.trim_start_matches('#');

        // Support both 6-character (RGB) and 8-character (RGBA) hex codes
        // For RGBA, A is ignored
        match hex.len() {
            6 => {
                // RGB format: #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::Rgb(r, g, b))
            }
            8 => {
                // RGBA format: #RRGGBBAA (ignore alpha channel)
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                // Alpha channel at hex[6..8] is ignored
                Some(Color::Rgb(r, g, b))
            }
            _ => None,
        }
    }

    /// Get color with fallback to white
    pub fn get_color(&self, hex: &str) -> Color {
        Self::hex_to_color(hex).unwrap_or(Color::White)
    }
}

/// Theme manager for caching and applying themes
#[derive(Debug)]
pub struct ThemeManager {
    current_theme: Theme,
    color_cache: HashMap<String, Color>,
}

impl ThemeManager {
    pub fn new() -> Result<Self> {
        let theme = Theme::load()?;
        let mut manager = ThemeManager {
            current_theme: theme,
            color_cache: HashMap::new(),
        };
        manager.rebuild_cache();
        Ok(manager)
    }

    pub fn theme(&self) -> &Theme {
        &self.current_theme
    }

    pub fn reload_theme(&mut self) -> Result<()> {
        self.current_theme = Theme::load()?;
        self.rebuild_cache();
        Ok(())
    }

    pub fn get_cached_color(&self, hex: &str) -> Color {
        self.color_cache.get(hex).copied().unwrap_or(Color::White)
    }

    fn rebuild_cache(&mut self) {
        self.color_cache.clear();

        // Cache all theme colors by cloning the theme first to avoid borrowing issues
        let theme = self.current_theme.clone();

        // UI colors only
        self.cache_color(&theme.ui.document_border);
        self.cache_color(&theme.ui.document_title);
        self.cache_color(&theme.ui.footer_bg);
        self.cache_color(&theme.ui.status_bar_fg);
        self.cache_color(&theme.ui.status_message_fg);
        self.cache_color(&theme.ui.help_bar_fg);
        self.cache_color(&theme.ui.search_border);
        self.cache_color(&theme.ui.search_input);
        self.cache_color(&theme.ui.search_match_bg);
        self.cache_color(&theme.ui.search_match_fg);
        self.cache_color(&theme.ui.outline_border);
        self.cache_color(&theme.ui.outline_item);
        self.cache_color(&theme.ui.outline_selected_bg);
        self.cache_color(&theme.ui.outline_selected_fg);
        self.cache_color(&theme.ui.help_border);
        self.cache_color(&theme.ui.help_text);
        self.cache_color(&theme.ui.scrollbar);
    }

    fn cache_color(&mut self, hex: &str) {
        if let Some(color) = Theme::hex_to_color(hex) {
            self.color_cache.insert(hex.to_string(), color);
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            let theme = Theme::default();
            let mut manager = ThemeManager {
                current_theme: theme,
                color_cache: HashMap::new(),
            };
            manager.rebuild_cache();
            manager
        })
    }
}
