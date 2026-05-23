use anyhow::Result;
use arboard::Clipboard;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    tty::IsTty,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame, Terminal,
};
use std::io;

use crate::{
    config::Config,
    document::*,
    keymap::{Action, KeyBinding, Keymap},
    state::StateManager,
    widgets::{DocumentWidget, LayoutCache},
    Cli,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

type ImageProtocols = Vec<StatefulProtocol>;

pub struct App {
    pub document: Document,
    pub current_view: ViewMode,
    pub scroll_offset: usize,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub backup_search_results: Vec<SearchResult>,
    pub current_search_index: usize,
    pub outline_state: ListState,
    pub show_help: bool,
    pub clipboard: Option<Clipboard>,
    pub status_message: Option<String>,
    pub color_enabled: bool,
    pub image_picker: Option<Picker>,
    pub image_protocols: ImageProtocols,
    pub layout_cache: LayoutCache,
    pub keymap: Keymap,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum ViewMode {
    #[default]
    Document,
    Outline,
    Search,
    #[allow(dead_code)]
    Help,
}

impl App {
    pub fn new(document: Document, cli: &Cli, config: &Config) -> Self {
        // Load saved state for this document (only if --restore-position flag is set)
        let saved_state = if cli.restore_position {
            if let Ok(manager) = StateManager::load() {
                use std::path::PathBuf;
                let doc_path = PathBuf::from(&document.metadata.file_path);
                manager.get_state(&doc_path)
            } else {
                None
            }
        } else {
            None // Default: start at top (like less)
        };

        // Initialize with default or saved state
        let (initial_scroll, initial_search, initial_view) = if let Some(state) = &saved_state {
            (
                state.scroll_offset,
                state.last_search.clone(),
                state.view_mode.clone(),
            )
        } else {
            (0, String::new(), ViewMode::Document)
        };

        let mut app = Self {
            document,
            current_view: initial_view,
            scroll_offset: initial_scroll,
            search_query: initial_search.clone(),
            search_results: Vec::new(),
            backup_search_results: Vec::new(),
            current_search_index: 0,
            outline_state: ListState::default(),
            show_help: false,
            clipboard: Clipboard::new().ok(),
            status_message: None,
            color_enabled: cli.color,
            image_picker: None,
            keymap: config.build_keymap(),
            image_protocols: Vec::new(),
            layout_cache: LayoutCache::new(),
        };

        // Restore search results if we had a saved search
        if !initial_search.is_empty() {
            app.search_results = crate::document::search_document(&app.document, &initial_search);
        }

        // CLI options override saved state
        if cli.outline {
            app.current_view = ViewMode::Outline;
        }

        if let Some(search) = &cli.search {
            app.search_query = search.clone();
            app.search_results = crate::document::search_document(&app.document, search);
            app.current_view = ViewMode::Search;
        }

        if let Some(page) = cli.page {
            // Rough estimate of elements per page
            let elements_per_page = 10;
            app.scroll_offset = (page.saturating_sub(1)) * elements_per_page;
        }

        // Initialize image support if images are enabled
        if cli.images {
            app.init_image_support();
        }

        app
    }

    fn init_image_support(&mut self) {
        // Try to initialize picker from terminal query on Unix, use font size on Windows
        #[cfg(unix)]
        let picker = if let Ok(p) = Picker::from_query_stdio() {
            p
        } else {
            // Fallback to manual font size
            Picker::from_fontsize((8, 16))
        };

        #[cfg(not(unix))]
        let picker = Picker::from_fontsize((8, 16));

        // Process all images in the document
        for element in &self.document.elements {
            if let DocumentElement::Image {
                image_path: Some(path),
                ..
            } = element
            {
                // Try to load and create protocol for each image
                if let Ok(img) = image::ImageReader::open(path) {
                    if let Ok(dyn_img) = img.decode() {
                        let protocol = picker.new_resize_protocol(dyn_img);
                        self.image_protocols.push(protocol);
                    }
                }
            }
        }

        self.image_picker = Some(picker);
    }

    pub fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
            if let Some(result) = self.search_results.get(self.current_search_index) {
                self.scroll_offset = result.element_index;
            }
        }
    }

    pub fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = if self.current_search_index == 0 {
                self.search_results.len() - 1
            } else {
                self.current_search_index - 1
            };
            if let Some(result) = self.search_results.get(self.current_search_index) {
                self.scroll_offset = result.element_index;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset + 1 < self.document.elements.len() {
            self.scroll_offset += 1;
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    pub fn page_down(&mut self, page_size: usize) {
        let max_offset = self.document.elements.len().saturating_sub(1);
        self.scroll_offset = std::cmp::min(self.scroll_offset + page_size, max_offset);
    }

    pub fn copy_content(&mut self) {
        if let Some(clipboard) = &mut self.clipboard {
            let content = match self.current_view {
                ViewMode::Document => {
                    // Copy the full document as text
                    crate::export::format_as_text(&self.document)
                }
                ViewMode::Search => {
                    // Copy search results
                    if self.search_results.is_empty() {
                        "No search results to copy.".to_string()
                    } else {
                        let mut content =
                            format!("Search results for '{}':\n\n", self.search_query);
                        for (i, result) in self.search_results.iter().enumerate() {
                            content.push_str(&format!("{}. {}\n", i + 1, result.text.trim()));
                        }
                        content
                    }
                }
                ViewMode::Outline => {
                    // Copy document outline
                    let outline = crate::document::generate_outline(&self.document);
                    let mut content = String::from("Document Outline:\n\n");
                    for item in outline {
                        let indent = "  ".repeat((item.level as usize).saturating_sub(1));
                        content.push_str(&format!("{}{}\n", indent, item.title));
                    }
                    content
                }
                _ => "Content not available for copying in this view.".to_string(),
            };

            match clipboard.set_text(content) {
                Ok(_) => {
                    self.status_message = Some("Copied to clipboard!".to_string());
                }
                Err(_) => {
                    self.status_message = Some("Failed to copy to clipboard.".to_string());
                }
            }
        } else {
            self.status_message = Some("Clipboard not available.".to_string());
        }
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    pub fn toggle_search_state(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        // Toggles search state: clears results if active, restores backup if inactive.
        if !self.search_results.is_empty() {
            self.backup_search_results = self.search_results.clone();
            self.search_results.clear();
        } else if !self.backup_search_results.is_empty() {
            self.search_results = self.backup_search_results.clone();
            self.backup_search_results.clear();
        }
    }
}

async fn run_non_interactive(document: Document, cli: &Cli) -> Result<()> {
    let app = App::new(document, cli, &Config::default());

    match app.current_view {
        ViewMode::Outline => {
            // Show outline
            let outline = crate::document::generate_outline(&app.document);
            println!("Document Outline:");
            println!("================");
            for item in outline {
                let indent = "  ".repeat((item.level.saturating_sub(1)) as usize);
                println!("{}{}", indent, item.title);
            }
        }
        ViewMode::Search => {
            // Show search results
            println!("Search Results for '{}':", app.search_query);
            println!("==============================");
            for (i, result) in app.search_results.iter().enumerate() {
                println!("{}. {}", i + 1, result.text.trim());
                println!();
            }
            if app.search_results.is_empty() {
                println!("No results found.");
            }
        }
        _ => {
            // Default: show basic document info and content preview
            println!("Document: {}", app.document.title);
            println!("Pages: {}", app.document.metadata.page_count);
            println!("Words: {}", app.document.metadata.word_count);
            println!();
            println!("Content Preview:");
            println!("================");

            // Show first few elements with proper formatting
            let preview_count = std::cmp::min(app.document.elements.len(), 20);
            for element in &app.document.elements[0..preview_count] {
                match element {
                    DocumentElement::Heading {
                        level,
                        text,
                        number,
                    } => {
                        let prefix = match level {
                            1 => "# ",
                            2 => "## ",
                            _ => "### ",
                        };
                        let heading_text = if let Some(number) = number {
                            format!("{number} {text}")
                        } else {
                            text.clone()
                        };
                        println!("{prefix}{heading_text}");
                        println!();
                    }
                    DocumentElement::Paragraph { runs } => {
                        let text: String = runs.iter().map(|run| run.text.as_str()).collect();
                        println!("{text}");
                        println!();
                    }
                    DocumentElement::List { items, ordered } => {
                        for (i, item) in items.iter().enumerate() {
                            let bullet = if *ordered {
                                format!("{}. ", i + 1)
                            } else {
                                "• ".to_string()
                            };
                            let indent = "  ".repeat(item.level as usize);
                            let item_text: String =
                                item.runs.iter().map(|run| run.text.as_str()).collect();
                            println!("{indent}{bullet}{item_text}");
                        }
                        println!();
                    }
                    DocumentElement::Table { .. } => {
                        println!("[Table content - use --export csv to view]");
                        println!();
                    }
                    DocumentElement::Image {
                        description,
                        image_path,
                        ..
                    } => {
                        if let Some(path) = image_path {
                            // Try to display the image inline using terminal protocols
                            match crate::terminal_image::TerminalImageRenderer::with_options(
                                app.document.image_options.max_width,
                                app.document.image_options.max_height,
                                app.document.image_options.scale,
                            )
                            .render_image_from_path(path, description)
                            {
                                Ok(_) => {
                                    // Image displayed successfully
                                    println!();
                                }
                                Err(_) => {
                                    // Fallback to text description
                                    println!("📷 [Image: {description}]");
                                    println!();
                                }
                            }
                        } else {
                            println!("📷 [Image: {description}]");
                            println!();
                        }
                    }
                    DocumentElement::Equation { latex, .. } => {
                        println!("📐 Equation: {latex}");
                        println!();
                    }
                    DocumentElement::PageBreak => {
                        println!("---");
                        println!();
                    }
                }
            }

            if app.document.elements.len() > preview_count {
                println!(
                    "... and {} more elements",
                    app.document.elements.len() - preview_count
                );
                println!();
            }

            println!(
                "Use --export to save full content, or run in an interactive terminal for full UI."
            );
        }
    }

    Ok(())
}

/// Save the current app state to disk
fn save_app_state(app: &App) {
    use crate::state::DocumentState;
    use std::path::PathBuf;

    // Load existing state manager
    let mut manager = StateManager::load().unwrap_or_default();

    // Create state for this document
    let doc_path = PathBuf::from(&app.document.metadata.file_path);
    let state = DocumentState {
        scroll_offset: app.scroll_offset,
        last_search: app.search_query.clone(),
        view_mode: app.current_view.clone(),
        last_accessed: std::time::SystemTime::now(),
    };

    // Update and save
    manager.set_state(&doc_path, state);

    // Ignore errors when saving state (don't crash the app on exit)
    let _ = manager.save();
}

pub async fn run_viewer(document: Document, cli: &Cli, config: &Config) -> Result<()> {
    // Check if we're in an interactive terminal or forced to use UI
    if !cli.force_ui && !IsTty::is_tty(&io::stdout()) {
        // Fallback for non-interactive environments
        return run_non_interactive(document, cli).await;
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(document, cli, config);

    // Run the app
    let res = run_app(&mut terminal, &mut app).await;

    // Save state before exiting
    save_app_state(&app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    let binding = KeyBinding::new(key.code, key.modifiers);
                    let action = app.keymap.get_action(&binding);

                    // In search mode, unbound character keys append to the query
                    if matches!(app.current_view, ViewMode::Search) && action.is_none() {
                        if let KeyCode::Char(c) = key.code {
                            app.search_query.push(c);
                            app.search_results =
                                crate::document::search_document(&app.document, &app.search_query);
                            app.current_search_index = 0;
                            continue;
                        }
                    }

                    if let Some(action) = action {
                        // Clear status message on any action except copy
                        if app.status_message.is_some() && action != Action::Copy {
                            app.clear_status_message();
                        }

                        if handle_action(app, action) {
                            break;
                        }
                    }
                }
            }
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        match app.current_view {
                            ViewMode::Document => {
                                // Scroll up 3 lines for smooth mouse wheel experience
                                for _ in 0..3 {
                                    app.scroll_up();
                                }
                            }
                            ViewMode::Outline => {
                                let selected = app.outline_state.selected().unwrap_or(0);
                                if selected > 0 {
                                    app.outline_state.select(Some(selected - 1));
                                }
                            }
                            ViewMode::Search => app.prev_search_result(),
                            _ => {}
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        match app.current_view {
                            ViewMode::Document => {
                                // Scroll down 3 lines for smooth mouse wheel experience
                                for _ in 0..3 {
                                    app.scroll_down();
                                }
                            }
                            ViewMode::Outline => {
                                let selected = app.outline_state.selected().unwrap_or(0);
                                if selected + 1
                                    < crate::document::generate_outline(&app.document).len()
                                {
                                    app.outline_state.select(Some(selected + 1));
                                }
                            }
                            ViewMode::Search => app.next_search_result(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Returns true if the app should quit.
fn handle_action(app: &mut App, action: Action) -> bool {
    match action {
        Action::Quit => return true,
        Action::Escape => match app.current_view {
            ViewMode::Search | ViewMode::Outline => app.current_view = ViewMode::Document,
            ViewMode::Help => {
                app.show_help = false;
                app.current_view = ViewMode::Document;
            }
            ViewMode::Document => {}
        },
        Action::ScrollUp => app.scroll_up(),
        Action::ScrollDown => app.scroll_down(),
        Action::PageUp => app.page_up(10),
        Action::PageDown => app.page_down(10),
        Action::HalfPageUp => app.page_up(5),
        Action::HalfPageDown => app.page_down(5),
        Action::GotoStart => app.scroll_offset = 0,
        Action::GotoEnd => {
            app.scroll_offset = app.document.elements.len().saturating_sub(1);
        }
        Action::ToggleOutline => app.current_view = ViewMode::Outline,
        Action::EnterSearch => app.current_view = ViewMode::Search,
        Action::ToggleHelp => app.show_help = !app.show_help,
        Action::ToggleSearchState => app.toggle_search_state(),
        Action::SearchNext => {
            if !app.search_results.is_empty() {
                app.next_search_result();
            }
        }
        Action::SearchPrevious => {
            if !app.search_results.is_empty() {
                app.prev_search_result();
            }
        }
        Action::Copy => app.copy_content(),
        Action::OutlineSelect => {
            if let Some(selected) = app.outline_state.selected() {
                if let Some(item) = crate::document::generate_outline(&app.document).get(selected) {
                    app.scroll_offset = item.element_index;
                    app.current_view = ViewMode::Document;
                }
            }
        }
        Action::SearchDeleteChar => {
            app.search_query.pop();
            app.search_results = crate::document::search_document(&app.document, &app.search_query);
            app.current_search_index = 0;
        }
        Action::SearchSubmit => {
            if !app.search_results.is_empty() {
                app.next_search_result();
            }
        }
    }
    false
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.area());

    // Main content area
    match app.current_view {
        ViewMode::Document => render_document(f, chunks[0], app),
        ViewMode::Outline => render_outline(f, chunks[0], app),
        ViewMode::Search => render_search(f, chunks[0], app),
        ViewMode::Help => render_help(f, chunks[0], &app.keymap),
    }

    // Status bar
    render_status_bar(f, chunks[1], app);

    // Help overlay
    if app.show_help {
        render_help_overlay(f, app);
    }
}

fn render_document(f: &mut Frame, area: Rect, app: &mut App) {
    let title = format!("📄 doxx - {}", app.document.title);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Use DocumentWidget for unified rendering with proper text wrapping + images
    let mut doc_widget = DocumentWidget::new(&app.document.elements[..])
        .scroll_offset(app.scroll_offset)
        .color_enabled(app.color_enabled)
        .search_results(&app.search_results[..])
        .current_search_index(app.current_search_index);

    // Render the document content (text + images in single pass)
    doc_widget.render(inner, f, &mut app.image_protocols, &mut app.layout_cache);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(app.document.elements.len())
        .position(app.scroll_offset);

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_outline(f: &mut Frame, area: Rect, app: &mut App) {
    let outline = crate::document::generate_outline(&app.document);
    let items: Vec<ListItem> = outline
        .iter()
        .map(|item| {
            let indent = "  ".repeat((item.level.saturating_sub(1)) as usize);
            let text = format!("{}{}", indent, item.title);
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("📋 Document Outline")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("➤ ");

    f.render_stateful_widget(list, area, &mut app.outline_state);
}

fn render_search(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    // Search input
    let input = Paragraph::new(app.search_query.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔍 Search")
                .border_style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(input, chunks[0]);

    // Search results
    let results: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let prefix = "📄"; // Simplified for now

            let style = if i == app.current_search_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            // Truncate long results and add context (Unicode-safe)
            let display_text = if result.text.len() > 80 {
                // Safe truncation: find the largest valid UTF-8 boundary <= 77 bytes
                let max_bytes = 77;
                let safe_boundary = if result.text.len() <= max_bytes {
                    result.text.len()
                } else {
                    let mut boundary = max_bytes;
                    while boundary > 0 && !result.text.is_char_boundary(boundary) {
                        boundary -= 1;
                    }
                    boundary
                };
                format!("{}...", &result.text[..safe_boundary])
            } else {
                result.text.clone()
            };

            ListItem::new(format!("{} {} [{}]", prefix, display_text, i + 1)).style(style)
        })
        .collect();

    let results_list = List::new(results).block(
        Block::default()
            .title(format!(
                "Results ({}/{})",
                if app.search_results.is_empty() {
                    0
                } else {
                    app.current_search_index + 1
                },
                app.search_results.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(results_list, chunks[1]);
}

fn render_help(f: &mut Frame, area: Rect, keymap: &Keymap) {
    let mut lines: Vec<String> = vec![
        "doxx - Help".to_string(),
        "".to_string(),
        "Navigation:".to_string(),
    ];

    let nav_actions: &[(Action, &str)] = &[
        (Action::ScrollUp, "Scroll up"),
        (Action::ScrollDown, "Scroll down"),
        (Action::PageUp, "Page up"),
        (Action::PageDown, "Page down"),
        (Action::HalfPageUp, "Half-page up"),
        (Action::HalfPageDown, "Half-page down"),
        (Action::GotoStart, "Go to start"),
        (Action::GotoEnd, "Go to end"),
    ];
    for (action, desc) in nav_actions {
        let keys = keymap.keys_for_action(*action);
        if !keys.is_empty() {
            lines.push(format!("  {:<16} {}", keys.join("/"), desc));
        }
    }

    lines.push("".to_string());
    lines.push("Search:".to_string());
    let search_actions: &[(Action, &str)] = &[
        (Action::EnterSearch, "Open search"),
        (Action::SearchNext, "Next result"),
        (Action::SearchPrevious, "Previous result"),
        (Action::ToggleSearchState, "Deselect/Reselect selection"),
    ];
    for (action, desc) in search_actions {
        let keys = keymap.keys_for_action(*action);
        if !keys.is_empty() {
            lines.push(format!("  {:<16} {}", keys.join("/"), desc));
        }
    }

    lines.push("".to_string());
    lines.push("Other:".to_string());
    let other_actions: &[(Action, &str)] = &[
        (Action::ToggleOutline, "Show outline"),
        (Action::Copy, "Copy content to clipboard"),
        (Action::ToggleHelp, "Toggle help"),
        (Action::Quit, "Quit"),
    ];
    for (action, desc) in other_actions {
        let keys = keymap.keys_for_action(*action);
        if !keys.is_empty() {
            lines.push(format!("  {:<16} {}", keys.join("/"), desc));
        }
    }

    lines.push("".to_string());
    lines.push("Copy modes:".to_string());
    lines.push("  Document:  copies full document as text".to_string());
    lines.push("  Outline:   copies document structure".to_string());
    lines.push("  Search:    copies search results (F2)".to_string());
    lines.push("".to_string());
    lines.push("Press any key to close...".to_string());

    let help = Paragraph::new(lines.join("\n"))
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

fn render_help_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);
    render_help(f, area, &app.keymap);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let metadata = &app.document.metadata;
    let view_indicator = match app.current_view {
        ViewMode::Document => "📄 Document",
        ViewMode::Outline => "📋 Outline",
        ViewMode::Search => "🔍 Search",
        ViewMode::Help => "❓ Help",
    };

    let search_info = if !app.search_results.is_empty() {
        format!(
            " • 🔍 {}/{} matches",
            app.current_search_index + 1,
            app.search_results.len()
        )
    } else if !app.search_query.is_empty() {
        " • 🔍 No matches".to_string()
    } else {
        String::new()
    };

    let status_text = if let Some(status_msg) = &app.status_message {
        // Show status message (like copy confirmation) with higher priority
        status_msg.clone()
    } else {
        format!(
            "{} • 📄 {} • {} pages • {} words • {}/{}{}",
            view_indicator,
            metadata
                .file_path
                .split('/')
                .next_back()
                .unwrap_or("Unknown"),
            metadata.page_count,
            metadata.word_count,
            app.scroll_offset + 1,
            app.document.elements.len(),
            search_info
        )
    };

    let status_style = if app.status_message.is_some() {
        // Highlight status messages
        Style::default()
            .fg(Color::Green)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default());

    f.render_widget(status, area);

    // Navigation help (keys pulled from active keymap)
    let km = &app.keymap;
    let help_text = format!(
        "[↕] Scroll [{}] Outline [{}] Search [{}] Copy [{}] Help [{}] Quit",
        km.primary_key_for_action(Action::ToggleOutline),
        km.primary_key_for_action(Action::EnterSearch),
        km.primary_key_for_action(Action::Copy),
        km.primary_key_for_action(Action::ToggleHelp),
        km.primary_key_for_action(Action::Quit),
    );
    let help_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: 1,
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default());

    f.render_widget(help, help_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
