use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use std::io;

struct App {
    image_protocol: StatefulProtocol,
}

impl App {
    fn new() -> Result<Self> {
        // Try to initialize picker from terminal query on Unix
        #[cfg(unix)]
        let picker = if let Ok(p) = Picker::from_query_stdio() {
            p
        } else {
            // Fallback to manual font size
            Picker::from_fontsize((8, 16))
        };

        #[cfg(not(unix))]
        let picker = Picker::from_fontsize((8, 16));

        // Load a test image - we'll create a simple one
        let img = image::DynamicImage::ImageRgb8(
            image::RgbImage::from_fn(400, 300, |x, y| {
                // Create a gradient pattern
                let r = ((x as f32 / 400.0) * 255.0) as u8;
                let g = ((y as f32 / 300.0) * 255.0) as u8;
                let b = 128;
                image::Rgb([r, g, b])
            })
        );

        let protocol = picker.new_resize_protocol(img);

        Ok(Self {
            image_protocol: protocol,
        })
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Run the app
    let res = run_app(&mut terminal, &mut app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("ratatui-image Test (Press 'q' to quit)")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Image area
    let image_block = Block::default()
        .title("Test Image")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let image_inner = image_block.inner(chunks[1]);
    f.render_widget(image_block, chunks[1]);

    // Render the image
    let image_widget = StatefulImage::new();
    f.render_stateful_widget(image_widget, image_inner, &mut app.image_protocol);

    // Footer with protocol info
    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Terminal: WezTerm (Kitty protocol)"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
