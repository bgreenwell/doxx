use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn char(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    pub fn ctrl(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// Parse a key binding from a string like "ctrl-d", "shift-h", "/", "enter", "esc", "space".
    pub fn parse_key(s: &str) -> Result<Self> {
        // A literal " " is meaningful (the spacebar), so check for it before
        // trimming would otherwise reduce it to an empty, unparseable string.
        if s == " " {
            return Ok(Self::char(' '));
        }

        let s = s.trim().to_lowercase();
        let parts: Vec<&str> = s.splitn(2, '-').collect();

        match parts.as_slice() {
            ["ctrl", rest] => {
                let c = parse_single_char(rest)?;
                Ok(Self::ctrl(c))
            }
            ["shift", rest] => {
                let c = parse_single_char(rest)?;
                Ok(Self::new(
                    KeyCode::Char(c.to_ascii_uppercase()),
                    KeyModifiers::NONE,
                ))
            }
            [single] => {
                // Special key names
                match *single {
                    "space" => Ok(Self::char(' ')),
                    "enter" => Ok(Self::new(KeyCode::Enter, KeyModifiers::NONE)),
                    "esc" | "escape" => Ok(Self::new(KeyCode::Esc, KeyModifiers::NONE)),
                    "backspace" => Ok(Self::new(KeyCode::Backspace, KeyModifiers::NONE)),
                    "tab" => Ok(Self::new(KeyCode::Tab, KeyModifiers::NONE)),
                    "up" => Ok(Self::new(KeyCode::Up, KeyModifiers::NONE)),
                    "down" => Ok(Self::new(KeyCode::Down, KeyModifiers::NONE)),
                    "left" => Ok(Self::new(KeyCode::Left, KeyModifiers::NONE)),
                    "right" => Ok(Self::new(KeyCode::Right, KeyModifiers::NONE)),
                    "pageup" | "pgup" => Ok(Self::new(KeyCode::PageUp, KeyModifiers::NONE)),
                    "pagedown" | "pgdn" => Ok(Self::new(KeyCode::PageDown, KeyModifiers::NONE)),
                    "home" => Ok(Self::new(KeyCode::Home, KeyModifiers::NONE)),
                    "end" => Ok(Self::new(KeyCode::End, KeyModifiers::NONE)),
                    "f1" => Ok(Self::new(KeyCode::F(1), KeyModifiers::NONE)),
                    "f2" => Ok(Self::new(KeyCode::F(2), KeyModifiers::NONE)),
                    other => {
                        // Use original case for char lookup (avoid lowercasing 'N' → 'n')
                        let original = s.as_str();
                        let c = parse_single_char(original)?;
                        let _ = other; // suppress unused warning
                        Ok(Self::char(c))
                    }
                }
            }
            _ => bail!("Cannot parse key binding: {s}"),
        }
    }
}

impl KeyBinding {
    pub fn display(&self) -> String {
        let key = match &self.code {
            KeyCode::Char(' ') => "Space".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::PageUp => "PgUp".to_string(),
            KeyCode::PageDown => "PgDn".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::F(n) => format!("F{n}"),
            _ => "?".to_string(),
        };
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            format!("ctrl-{key}")
        } else {
            key
        }
    }

    pub(super) fn sort_priority(&self) -> u8 {
        match &self.code {
            KeyCode::Char(_) if self.modifiers == KeyModifiers::NONE => 0,
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::F(_) => 1,
            _ => 2,
        }
    }
}

fn parse_single_char(s: &str) -> Result<char> {
    let mut chars = s.chars();
    let c = chars
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty key string"))?;
    if chars.next().is_some() {
        bail!("Expected single character, got: {s}");
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    #[test]
    fn test_parse_ctrl() {
        let b = KeyBinding::parse_key("ctrl-d").unwrap();
        assert_eq!(b.code, KeyCode::Char('d'));
        assert_eq!(b.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_parse_char() {
        let b = KeyBinding::parse_key("q").unwrap();
        assert_eq!(b.code, KeyCode::Char('q'));
        assert_eq!(b.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_parse_space() {
        // Literal " " must survive parse_key's trim(), since trimming it
        // naively reduces the spacebar to an empty, unparseable string.
        let b = KeyBinding::parse_key(" ").unwrap();
        assert_eq!(b.code, KeyCode::Char(' '));
        assert_eq!(b.modifiers, KeyModifiers::NONE);

        // "space" is also accepted as a named alias, for readability in config.toml.
        let b = KeyBinding::parse_key("space").unwrap();
        assert_eq!(b.code, KeyCode::Char(' '));
    }

    #[test]
    fn test_display_space() {
        assert_eq!(KeyBinding::char(' ').display(), "Space");
    }

    #[test]
    fn test_parse_special_keys() {
        assert_eq!(KeyBinding::parse_key("enter").unwrap().code, KeyCode::Enter);
        assert_eq!(KeyBinding::parse_key("esc").unwrap().code, KeyCode::Esc);
        assert_eq!(KeyBinding::parse_key("up").unwrap().code, KeyCode::Up);
    }
}
