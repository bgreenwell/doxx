//! Document state persistence
//!
//! This module handles saving and loading document state (scroll position, search, view mode)
//! across sessions. State is stored in a platform-specific config directory.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use crate::ui::ViewMode;

/// State for a single document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    /// Last scroll position (element index)
    pub scroll_offset: usize,
    /// Last search query
    pub last_search: String,
    /// Last view mode (Document, Outline, Search)
    #[serde(skip)]
    pub view_mode: ViewMode,
    /// When this document was last accessed
    #[serde(default = "SystemTime::now")]
    pub last_accessed: SystemTime,
}

impl Default for DocumentState {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            last_search: String::new(),
            view_mode: ViewMode::Document,
            last_accessed: SystemTime::now(),
        }
    }
}

/// Global state manager for all documents
#[derive(Debug, Serialize, Deserialize)]
pub struct StateManager {
    /// Map of absolute file paths to their state
    documents: HashMap<String, DocumentState>,
}

impl StateManager {
    /// Create a new empty state manager
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Load state from disk, or create new if doesn't exist
    pub fn load() -> Result<Self> {
        let state_path = Self::state_file_path()?;

        if !state_path.exists() {
            return Ok(Self::new());
        }

        let contents = fs::read_to_string(&state_path).context("Failed to read state file")?;

        let mut manager: StateManager =
            serde_json::from_str(&contents).context("Failed to parse state file")?;

        // Clean up old entries (older than 90 days)
        manager.cleanup_old_entries(Duration::from_secs(90 * 24 * 60 * 60));

        Ok(manager)
    }

    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        let state_path = Self::state_file_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent).context("Failed to create state directory")?;
        }

        let contents = serde_json::to_string_pretty(self).context("Failed to serialize state")?;

        fs::write(&state_path, contents).context("Failed to write state file")?;

        Ok(())
    }

    /// Get state for a document
    pub fn get_state(&self, file_path: &Path) -> Option<DocumentState> {
        let key = file_path.to_string_lossy().to_string();
        self.documents.get(&key).cloned()
    }

    /// Update state for a document
    pub fn set_state(&mut self, file_path: &Path, state: DocumentState) {
        let key = file_path.to_string_lossy().to_string();
        self.documents.insert(key, state);
    }

    /// Remove old entries that haven't been accessed recently
    fn cleanup_old_entries(&mut self, max_age: Duration) {
        let now = SystemTime::now();
        self.documents.retain(|_, state| {
            now.duration_since(state.last_accessed)
                .map(|age| age < max_age)
                .unwrap_or(false)
        });
    }

    /// Get the platform-specific state file path
    ///
    /// Returns:
    /// - macOS: ~/Library/Application Support/doxx/state.json
    /// - Linux: ~/.config/doxx/state.json
    /// - Windows: %APPDATA%\doxx\state.json
    fn state_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to determine config directory")?;

        Ok(config_dir.join("doxx").join("state.json"))
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_new() {
        let manager = StateManager::new();
        assert_eq!(manager.documents.len(), 0);
    }

    #[test]
    fn test_set_and_get_state() {
        let mut manager = StateManager::new();
        let path = PathBuf::from("/test/document.docx");

        let state = DocumentState {
            scroll_offset: 42,
            last_search: "test".to_string(),
            view_mode: ViewMode::Search,
            last_accessed: SystemTime::now(),
        };

        manager.set_state(&path, state.clone());

        let retrieved = manager.get_state(&path).unwrap();
        assert_eq!(retrieved.scroll_offset, 42);
        assert_eq!(retrieved.last_search, "test");
    }

    #[test]
    fn test_cleanup_old_entries() {
        let mut manager = StateManager::new();
        let path = PathBuf::from("/test/old.docx");

        // Create a state with an old timestamp
        let old_time = SystemTime::now() - Duration::from_secs(100 * 24 * 60 * 60); // 100 days ago
        let state = DocumentState {
            scroll_offset: 0,
            last_search: String::new(),
            view_mode: ViewMode::Document,
            last_accessed: old_time,
        };

        manager.set_state(&path, state);
        assert_eq!(manager.documents.len(), 1);

        // Clean up entries older than 90 days
        manager.cleanup_old_entries(Duration::from_secs(90 * 24 * 60 * 60));

        // Old entry should be removed
        assert_eq!(manager.documents.len(), 0);
    }

    #[test]
    fn test_state_file_path_returns_path() {
        let path = StateManager::state_file_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.ends_with("doxx/state.json") || path.ends_with("doxx\\state.json"));
    }
}
