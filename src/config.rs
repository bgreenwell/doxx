use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::keymap::{Action, KeyBinding, Keymap, KeymapPreset};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub keymap: KeymapConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapConfig {
    /// Preset name: "default", "vim", "less"
    #[serde(default = "default_preset")]
    pub preset: String,

    /// Custom key overrides: key string -> action string
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

fn default_preset() -> String {
    "default".to_string()
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self {
            preset: default_preset(),
            custom: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_file_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_file_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    pub fn config_file_path() -> Result<PathBuf> {
        let dir = dirs::config_dir().context("Failed to determine config directory")?;
        Ok(dir.join("doxx").join("config.toml"))
    }

    /// Build a Keymap from this config (preset + custom overrides).
    pub fn build_keymap(&self) -> Keymap {
        let preset = match self.keymap.preset.as_str() {
            "vim" => KeymapPreset::Vim,
            "less" => KeymapPreset::Less,
            _ => KeymapPreset::Default,
        };

        let mut km = Keymap::from_preset(preset);

        for (key_str, action_str) in &self.keymap.custom {
            match (KeyBinding::parse_key(key_str), action_str.parse::<Action>()) {
                (Ok(key), Ok(action)) => km.bind(key, action),
                (Err(e), _) => eprintln!("doxx config: invalid key {:?}: {e}", key_str),
                (_, Err(e)) => eprintln!("doxx config: invalid action {:?}: {e}", action_str),
            }
        }

        km
    }

    /// Get a dot-path config value as a string (e.g. "keymap.preset").
    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "keymap.preset" => Some(self.keymap.preset.clone()),
            _ => None,
        }
    }

    /// Set a dot-path config value (e.g. "keymap.preset" = "vim").
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "keymap.preset" => {
                match value {
                    "default" | "vim" | "less" => self.keymap.preset = value.to_string(),
                    other => {
                        anyhow::bail!("Unknown keymap preset: {other}. Valid: default, vim, less")
                    }
                }
                Ok(())
            }
            other => anyhow::bail!("Unknown config key: {other}"),
        }
    }
}
