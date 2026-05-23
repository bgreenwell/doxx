pub mod actions;
pub mod bindings;
pub mod presets;

use std::collections::HashMap;

pub use actions::Action;
pub use bindings::KeyBinding;
pub use presets::KeymapPreset;

pub struct Keymap {
    bindings: HashMap<KeyBinding, Action>,
}

impl Keymap {
    pub fn from_preset(preset: KeymapPreset) -> Self {
        Self {
            bindings: presets::create_keymap(preset),
        }
    }

    pub fn bind(&mut self, key: KeyBinding, action: Action) {
        self.bindings.insert(key, action);
    }

    pub fn get_action(&self, key: &KeyBinding) -> Option<Action> {
        self.bindings.get(key).copied()
    }

    /// Returns all key strings bound to `action`, sorted: plain chars first, then
    /// special keys, then modifier combos.
    pub fn keys_for_action(&self, action: Action) -> Vec<String> {
        let mut bindings: Vec<&KeyBinding> = self
            .bindings
            .iter()
            .filter(|(_, a)| **a == action)
            .map(|(k, _)| k)
            .collect();
        bindings.sort_by_key(|k| (k.sort_priority(), k.display()));
        bindings.iter().map(|k| k.display()).collect()
    }

    /// Returns the single "best" key string for `action` (first after sorting).
    pub fn primary_key_for_action(&self, action: Action) -> String {
        self.keys_for_action(action)
            .into_iter()
            .next()
            .unwrap_or_else(|| "(unbound)".to_string())
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::from_preset(KeymapPreset::Default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keymap() {
        let km = Keymap::default();
        assert_eq!(km.get_action(&KeyBinding::char('q')), Some(Action::Quit));
        assert_eq!(
            km.get_action(&KeyBinding::char('s')),
            Some(Action::EnterSearch)
        );
    }

    #[test]
    fn test_vim_preset() {
        let km = Keymap::from_preset(KeymapPreset::Vim);
        assert_eq!(
            km.get_action(&KeyBinding::char('/')),
            Some(Action::EnterSearch)
        );
        assert_eq!(
            km.get_action(&KeyBinding::ctrl('d')),
            Some(Action::HalfPageDown)
        );
        assert_eq!(
            km.get_action(&KeyBinding::ctrl('u')),
            Some(Action::HalfPageUp)
        );
        assert_eq!(
            km.get_action(&KeyBinding::char('N')),
            Some(Action::SearchPrevious)
        );
    }

    #[test]
    fn test_custom_override() {
        let mut km = Keymap::from_preset(KeymapPreset::Default);
        km.bind(KeyBinding::char('s'), Action::ToggleHelp);
        assert_eq!(
            km.get_action(&KeyBinding::char('s')),
            Some(Action::ToggleHelp)
        );
    }
}
