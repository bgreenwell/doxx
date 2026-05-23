use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // Navigation
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,
    GotoStart,
    GotoEnd,

    // View switching
    ToggleOutline,
    EnterSearch,
    ToggleHelp,
    ToggleSearchState,

    // Search navigation (active when results exist)
    SearchNext,
    SearchPrevious,

    // Actions
    Copy,
    Quit,
    Escape,

    // Outline-specific
    OutlineSelect,

    // Search-mode input
    SearchDeleteChar,
    SearchSubmit,
}

impl FromStr for Action {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "scroll_up" => Ok(Action::ScrollUp),
            "scroll_down" => Ok(Action::ScrollDown),
            "page_up" => Ok(Action::PageUp),
            "page_down" => Ok(Action::PageDown),
            "half_page_up" => Ok(Action::HalfPageUp),
            "half_page_down" => Ok(Action::HalfPageDown),
            "goto_start" => Ok(Action::GotoStart),
            "goto_end" => Ok(Action::GotoEnd),
            "toggle_outline" => Ok(Action::ToggleOutline),
            "search" => Ok(Action::EnterSearch),
            "toggle_help" => Ok(Action::ToggleHelp),
            "toggle_search_state" => Ok(Action::ToggleSearchState),
            "search_next" => Ok(Action::SearchNext),
            "search_previous" => Ok(Action::SearchPrevious),
            "copy" => Ok(Action::Copy),
            "quit" => Ok(Action::Quit),
            "escape" => Ok(Action::Escape),
            "outline_select" => Ok(Action::OutlineSelect),
            "search_delete_char" => Ok(Action::SearchDeleteChar),
            "search_submit" => Ok(Action::SearchSubmit),
            other => anyhow::bail!("Unknown action: {other}"),
        }
    }
}
