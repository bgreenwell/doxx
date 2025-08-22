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
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame, Terminal,
};
use std::io;

use crate::{document::*, Cli};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

type ImageProtocols = Vec<Box<dyn StatefulProtocol>>;

pub struct App {
    pub document: Document,
    pub current_view: ViewMode,
    pub scroll_offset: usize,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub current_search_index: usize,
    pub outline_state: ListState,
    pub show_help: bool,
    pub clipboard: Option<Clipboard>,
    pub status_message: Option<String>,
    pub color_enabled: bool,
    pub image_picker: Option<Picker>,
    pub image_protocols: ImageProtocols,
}

#[derive(Debug, Clone)]
pub enum ViewMode {
    Document,
    Outline,
    Search,
    #[allow(dead_code)]
    Help,
}

impl App {
    pub fn new(document: Document, cli: &Cli) -> Self {
        let mut app = Self {
            document,
            current_view: ViewMode::Document,
            scroll_offset: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            outline_state: ListState::default(),
            show_help: false,
            clipboard: Clipboard::new().ok(),
            status_message: None,
            color_enabled: cli.color,
            image_picker: None,
            image_protocols: Vec::new(),
        };

        // Apply CLI options
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
        // Try to initialize picker from termios on Unix, use default on Windows
        #[cfg(unix)]
        let mut picker = if let Ok(p) = Picker::from_termios() {
            p
        } else {
            // Fallback to manual font size
            Picker::new((8, 16))
        };

        #[cfg(not(unix))]
        let mut picker = Picker::new((8, 16));

        picker.guess_protocol();

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
}

async fn run_non_interactive(document: Document, cli: &Cli) -> Result<()> {
    let app = App::new(document, cli);

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
                    DocumentElement::Paragraph { text, .. } => {
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
                            println!("{}{}{}", indent, bullet, item.text);
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

pub async fn run_viewer(document: Document, cli: &Cli) -> Result<()> {
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
    let mut app = App::new(document, cli);

    // Run the app
    let res = run_app(&mut terminal, &mut app).await;

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
                    // Clear status message on any key press (except the copy key)
                    if app.status_message.is_some()
                        && key.code != KeyCode::Char('c')
                        && key.code != KeyCode::F(2)
                    {
                        app.clear_status_message();
                    }
                    match app.current_view {
                        ViewMode::Document => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('o') => app.current_view = ViewMode::Outline,
                            KeyCode::Char('s') => app.current_view = ViewMode::Search,
                            KeyCode::Char('h') | KeyCode::F(1) => app.show_help = !app.show_help,
                            KeyCode::Char('c') => app.copy_content(),
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                            KeyCode::PageUp => app.page_up(10),
                            KeyCode::PageDown => app.page_down(10),
                            KeyCode::Home => app.scroll_offset = 0,
                            KeyCode::End => {
                                app.scroll_offset = app.document.elements.len().saturating_sub(1)
                            }
                            KeyCode::Char('n') if !app.search_results.is_empty() => {
                                app.next_search_result()
                            }
                            KeyCode::Char('p') if !app.search_results.is_empty() => {
                                app.prev_search_result()
                            }
                            _ => {}
                        },
                        ViewMode::Outline => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.current_view = ViewMode::Document
                            }
                            KeyCode::Char('c') => app.copy_content(),
                            KeyCode::Up | KeyCode::Char('k') => {
                                let selected = app.outline_state.selected().unwrap_or(0);
                                if selected > 0 {
                                    app.outline_state.select(Some(selected - 1));
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let selected = app.outline_state.selected().unwrap_or(0);
                                if selected + 1
                                    < crate::document::generate_outline(&app.document).len()
                                {
                                    app.outline_state.select(Some(selected + 1));
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(selected) = app.outline_state.selected() {
                                    if let Some(outline_item) =
                                        crate::document::generate_outline(&app.document)
                                            .get(selected)
                                    {
                                        app.scroll_offset = outline_item.element_index;
                                        app.current_view = ViewMode::Document;
                                    }
                                }
                            }
                            _ => {}
                        },
                        ViewMode::Search => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.current_view = ViewMode::Document
                            }
                            KeyCode::F(2) => app.copy_content(), // Use F2 for copy in search mode to avoid conflicts
                            KeyCode::Char(c) => {
                                app.search_query.push(c);
                                app.search_results = crate::document::search_document(
                                    &app.document,
                                    &app.search_query,
                                );
                                app.current_search_index = 0;
                            }
                            KeyCode::Backspace => {
                                app.search_query.pop();
                                app.search_results = crate::document::search_document(
                                    &app.document,
                                    &app.search_query,
                                );
                                app.current_search_index = 0;
                            }
                            KeyCode::Enter | KeyCode::Down => app.next_search_result(),
                            KeyCode::Up => app.prev_search_result(),
                            _ => {}
                        },
                        ViewMode::Help => match key.code {
                            KeyCode::Char('q')
                            | KeyCode::Esc
                            | KeyCode::Char('h')
                            | KeyCode::F(1) => {
                                app.show_help = false;
                                app.current_view = ViewMode::Document;
                            }
                            _ => {}
                        },
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
        ViewMode::Help => render_help(f, chunks[0]),
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

    let visible_height = inner.height as usize;
    let end_index = std::cmp::min(
        app.scroll_offset + visible_height,
        app.document.elements.len(),
    );

    let mut text = Text::default();

    for (index, element) in app.document.elements[app.scroll_offset..end_index]
        .iter()
        .enumerate()
    {
        let actual_index = app.scroll_offset + index;
        let is_search_match = app
            .search_results
            .iter()
            .any(|r| r.element_index == actual_index);

        match element {
            DocumentElement::Heading {
                level,
                text: heading_text,
                number,
            } => {
                let style = match level {
                    1 => Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    2 => Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                };

                let prefix = match level {
                    1 => "■ ".to_string(),
                    2 => "  ▶ ".to_string(),
                    3 => "    ◦ ".to_string(),
                    _ => "      • ".to_string(),
                };

                let display_text = if let Some(number) = number {
                    format!("{number} {heading_text}")
                } else {
                    heading_text.clone()
                };

                let line = if is_search_match {
                    Line::from(vec![
                        Span::styled(prefix.clone(), style),
                        Span::styled(display_text, style.bg(Color::Yellow).fg(Color::Black)),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(display_text, style),
                    ])
                };
                text.lines.push(line);
                text.lines.push(Line::from(""));
            }
            DocumentElement::Paragraph {
                text: para_text,
                formatting,
            } => {
                let mut style = Style::default();
                if formatting.bold {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if formatting.italic {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if formatting.underline {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }

                // Apply text color from document formatting (only if color is enabled)
                if app.color_enabled {
                    if let Some(color_hex) = &formatting.color {
                        if let Some(color) = hex_to_color(color_hex) {
                            style = style.fg(color);
                        }
                    }
                }

                // Add visual indication for different types of content
                let display_text = if para_text.trim().is_empty() {
                    // Skip empty paragraphs
                    continue;
                } else if para_text.len() > 100 {
                    // Long paragraphs get some indentation
                    format!("  {para_text}")
                } else {
                    para_text.clone()
                };

                if is_search_match {
                    style = style.bg(Color::Yellow).fg(Color::Black);
                }

                text.lines
                    .push(Line::from(Span::styled(display_text, style)));
                text.lines.push(Line::from(""));
            }
            DocumentElement::List { items, ordered } => {
                for (i, item) in items.iter().enumerate() {
                    let bullet = if *ordered {
                        format!("{}. ", i + 1)
                    } else {
                        "• ".to_string()
                    };

                    let indent = "  ".repeat(item.level as usize);

                    // Combine indent and bullet to ensure proper spacing
                    let prefixed_bullet = format!("{indent}{bullet}");
                    let line = Line::from(vec![
                        Span::styled(prefixed_bullet, Style::default().fg(Color::Blue)),
                        Span::raw(&item.text),
                    ]);
                    text.lines.push(line);
                }
                text.lines.push(Line::from(""));
            }
            DocumentElement::Table { table } => {
                render_table_enhanced(table, &mut text);
            }
            DocumentElement::Image {
                description,
                width,
                height,
                image_path,
                ..
            } => {
                let dimensions = match (width, height) {
                    (Some(w), Some(h)) => format!(" ({w}x{h})"),
                    _ => String::new(),
                };

                let status = if image_path.is_some() && !app.image_protocols.is_empty() {
                    " [TUI placeholder - use --export text to view images]"
                } else if image_path.is_some() {
                    " [Image available - use --export text to view]"
                } else {
                    " [Image not extracted]"
                };

                let line = Line::from(vec![
                    Span::styled("🖼️  ", Style::default().fg(Color::Magenta)),
                    Span::styled(description, Style::default().fg(Color::Gray)),
                    Span::styled(dimensions, Style::default().fg(Color::DarkGray)),
                    Span::styled(status, Style::default().fg(Color::Green)),
                ]);
                text.lines.push(line);
                text.lines.push(Line::from(""));
            }
            DocumentElement::PageBreak => {
                text.lines.push(Line::from(Span::styled(
                    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                    Style::default().fg(Color::DarkGray),
                )));
                text.lines.push(Line::from(""));
            }
        }
    }

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: false }) // Don't trim whitespace to preserve list indentation
        .scroll((0, 0));

    f.render_widget(paragraph, inner);

    // Render scrollbar
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

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        "🆘 doxx - Help",
        "",
        "📖 Document Navigation:",
        "  ↑/k        Scroll up",
        "  ↓/j        Scroll down",
        "  Page Up    Page up",
        "  Page Down  Page down",
        "  Home       Go to start",
        "  End        Go to end",
        "",
        "🔍 Search:",
        "  s          Open search",
        "  n          Next result",
        "  p          Previous result",
        "",
        "📋 Other Features:",
        "  o          Show outline",
        "  c          Copy content to clipboard",
        "  h/F1       Toggle help",
        "  q          Quit",
        "",
        "📄 Copy Functionality:",
        "  Document:  Copies full document as text",
        "  Outline:   Copies document structure",
        "  Search:    Copies search results (use F2)",
        "",
        "Press any key to close help...",
    ];

    let help = Paragraph::new(help_text.join("\n"))
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

fn render_help_overlay(f: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);
    render_help(f, area);
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

    // Navigation help
    let help_text = "[↕] Scroll [o] Outline [s] Search [c] Copy [h] Help [q] Quit";
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

fn render_table_enhanced(table: &TableData, text: &mut Text) {
    let metadata = &table.metadata;

    // Add table title if present
    if let Some(title) = &metadata.title {
        text.lines.push(Line::from(Span::styled(
            format!("📊 {title}"),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )));
        text.lines.push(Line::from(""));
    }

    // Generate table with proper alignment and borders
    if !table.headers.is_empty() {
        // Top border
        let top_border = generate_table_border(&metadata.column_widths, BorderType::Top);
        text.lines.push(Line::from(Span::styled(
            top_border,
            Style::default().fg(Color::Gray),
        )));

        // Header row
        let header_line = render_table_row(&table.headers, &metadata.column_widths, true);
        text.lines.push(Line::from(Span::styled(
            header_line,
            Style::default().add_modifier(Modifier::BOLD),
        )));

        // Header separator
        let separator = generate_table_border(&metadata.column_widths, BorderType::Separator);
        text.lines.push(Line::from(Span::styled(
            separator,
            Style::default().fg(Color::Gray),
        )));

        // Data rows
        for row in &table.rows {
            let row_line = render_table_row(row, &metadata.column_widths, false);
            text.lines.push(Line::from(Span::raw(row_line)));
        }

        // Bottom border
        let bottom_border = generate_table_border(&metadata.column_widths, BorderType::Bottom);
        text.lines.push(Line::from(Span::styled(
            bottom_border,
            Style::default().fg(Color::Gray),
        )));
    }

    text.lines.push(Line::from(""));
}

#[derive(Clone, Copy)]
enum BorderType {
    Top,
    Separator,
    Bottom,
}

fn generate_table_border(column_widths: &[usize], border_type: BorderType) -> String {
    let (left, middle, right, fill) = match border_type {
        BorderType::Top => ("┌", "┬", "┐", "─"),
        BorderType::Separator => ("├", "┼", "┤", "─"),
        BorderType::Bottom => ("└", "┴", "┘", "─"),
    };

    let mut border = String::new();
    border.push_str(left);

    for (i, &width) in column_widths.iter().enumerate() {
        border.push_str(&fill.repeat(width + 2)); // +2 for padding
        if i < column_widths.len() - 1 {
            border.push_str(middle);
        }
    }

    border.push_str(right);
    border
}

fn render_table_row(cells: &[TableCell], column_widths: &[usize], is_header: bool) -> String {
    let mut row = String::new();
    row.push('│');

    for (i, cell) in cells.iter().enumerate() {
        let width = column_widths.get(i).copied().unwrap_or(10);
        let aligned_content = align_cell_content(&cell.content, cell.alignment, width);
        let formatted_content = if is_header {
            aligned_content
        } else {
            apply_cell_formatting(&aligned_content, &cell.formatting)
        };

        row.push(' ');
        row.push_str(&formatted_content);
        row.push(' ');
        row.push('│');
    }

    row
}

fn align_cell_content(content: &str, alignment: TextAlignment, width: usize) -> String {
    let trimmed = content.trim();

    match alignment {
        TextAlignment::Left => format!("{trimmed:<width$}"),
        TextAlignment::Right => format!("{trimmed:>width$}"),
        TextAlignment::Center => {
            let padding = width.saturating_sub(trimmed.len());
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            format!(
                "{}{}{}",
                " ".repeat(left_pad),
                trimmed,
                " ".repeat(right_pad)
            )
        }
        TextAlignment::Justify => {
            // For terminal output, treat justify as left-aligned
            format!("{trimmed:<width$}")
        }
    }
}

fn apply_cell_formatting(content: &str, _formatting: &TextFormatting) -> String {
    // For terminal output, we'll keep formatting simple
    // Advanced formatting could use ANSI codes here
    content.to_string()
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

/// Convert hex color code to ratatui Color
fn hex_to_color(hex: &str) -> Option<Color> {
    // Remove # if present and ensure we have 6 characters
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    // Parse RGB components
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}
