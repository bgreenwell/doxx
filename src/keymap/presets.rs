use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};

use super::{actions::Action, bindings::KeyBinding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeymapPreset {
    Default,
    Vim,
    Less,
}

pub fn create_keymap(preset: KeymapPreset) -> HashMap<KeyBinding, Action> {
    match preset {
        KeymapPreset::Default => create_default(),
        KeymapPreset::Vim => create_vim(),
        KeymapPreset::Less => create_less(),
    }
}

fn create_default() -> HashMap<KeyBinding, Action> {
    let mut m = HashMap::new();

    // Navigation
    m.insert(KeyBinding::char('k'), Action::ScrollUp);
    m.insert(
        KeyBinding::new(KeyCode::Up, KeyModifiers::NONE),
        Action::ScrollUp,
    );
    m.insert(KeyBinding::char('j'), Action::ScrollDown);
    m.insert(
        KeyBinding::new(KeyCode::Down, KeyModifiers::NONE),
        Action::ScrollDown,
    );
    m.insert(
        KeyBinding::new(KeyCode::PageUp, KeyModifiers::NONE),
        Action::PageUp,
    );
    m.insert(
        KeyBinding::new(KeyCode::PageDown, KeyModifiers::NONE),
        Action::PageDown,
    );
    m.insert(
        KeyBinding::new(KeyCode::Home, KeyModifiers::NONE),
        Action::GotoStart,
    );
    m.insert(
        KeyBinding::new(KeyCode::End, KeyModifiers::NONE),
        Action::GotoEnd,
    );

    // View switching
    m.insert(KeyBinding::char('o'), Action::ToggleOutline);
    m.insert(KeyBinding::char('s'), Action::EnterSearch);
    m.insert(KeyBinding::char('S'), Action::ToggleSearchState);
    m.insert(KeyBinding::char('h'), Action::ToggleHelp);
    m.insert(
        KeyBinding::new(KeyCode::F(1), KeyModifiers::NONE),
        Action::ToggleHelp,
    );

    // Search navigation
    m.insert(KeyBinding::char('n'), Action::SearchNext);
    m.insert(KeyBinding::char('p'), Action::SearchPrevious);

    // Actions
    m.insert(KeyBinding::char('c'), Action::Copy);
    m.insert(KeyBinding::char('q'), Action::Quit);
    m.insert(
        KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE),
        Action::Escape,
    );
    m.insert(
        KeyBinding::new(KeyCode::Enter, KeyModifiers::NONE),
        Action::OutlineSelect,
    );

    // Search-mode input helpers (used in search mode)
    m.insert(
        KeyBinding::new(KeyCode::Backspace, KeyModifiers::NONE),
        Action::SearchDeleteChar,
    );
    m.insert(
        KeyBinding::new(KeyCode::F(2), KeyModifiers::NONE),
        Action::Copy,
    );

    m
}

fn create_vim() -> HashMap<KeyBinding, Action> {
    let mut m = create_default();

    // Half-page navigation
    m.insert(KeyBinding::char('u'), Action::HalfPageUp);
    m.insert(KeyBinding::ctrl('u'), Action::HalfPageUp);
    m.insert(KeyBinding::char('d'), Action::HalfPageDown);
    m.insert(KeyBinding::ctrl('d'), Action::HalfPageDown);

    // Home/End aliases
    m.insert(KeyBinding::char('H'), Action::GotoStart);
    m.insert(KeyBinding::char('L'), Action::GotoEnd);
    m.insert(KeyBinding::char('g'), Action::GotoStart);
    m.insert(KeyBinding::char('G'), Action::GotoEnd);

    // / for search (vim-style), N for previous; remove default 's' binding
    m.insert(KeyBinding::char('/'), Action::EnterSearch);
    m.insert(KeyBinding::char('N'), Action::SearchPrevious);
    m.remove(&KeyBinding::char('s'));

    m
}

fn create_less() -> HashMap<KeyBinding, Action> {
    let mut m = create_default();

    // / for search; remove default 's' binding
    m.insert(KeyBinding::char('/'), Action::EnterSearch);
    m.insert(KeyBinding::char('N'), Action::SearchPrevious);
    m.remove(&KeyBinding::char('s'));

    // Half-page navigation
    m.insert(KeyBinding::char('u'), Action::HalfPageUp);
    m.insert(KeyBinding::ctrl('u'), Action::HalfPageUp);
    m.insert(KeyBinding::char('d'), Action::HalfPageDown);
    m.insert(KeyBinding::ctrl('d'), Action::HalfPageDown);

    // less uses g/G for start/end, space for page down, b for page up
    m.insert(KeyBinding::char('g'), Action::GotoStart);
    m.insert(KeyBinding::char('G'), Action::GotoEnd);
    m.insert(KeyBinding::char('b'), Action::PageUp);
    m.insert(KeyBinding::char(' '), Action::PageDown);

    m
}
