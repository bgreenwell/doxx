use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use doxx::{ColorDepth, ExportFormat};

mod ansi;
mod config;
mod document;
mod export;
pub mod image_extractor;
mod keymap;
mod state;
pub mod terminal_image;
mod ui;
mod widgets;

#[derive(Parser)]
#[command(
    name = "doxx",
    version,
    about = "Terminal document viewer for .docx files",
    long_about = "Beautiful .docx viewing in your terminal"
)]
struct Cli {
    /// Input document file (.docx)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Start with outline view
    #[arg(short, long)]
    outline: bool,

    /// Jump to specific page
    #[arg(short, long)]
    page: Option<usize>,

    /// Search and highlight term
    #[arg(short, long)]
    search: Option<String>,

    /// Export format
    #[arg(long, value_enum)]
    export: Option<ExportFormat>,

    /// Terminal width for ANSI export (default: $COLUMNS or 80)
    #[arg(short = 'w', long, value_name = "COLS")]
    terminal_width: Option<usize>,

    /// Color depth for ANSI export
    #[arg(long, value_enum, default_value = "auto")]
    color_depth: ColorDepth,

    /// Force interactive UI mode (bypass TTY detection)
    #[arg(long)]
    force_ui: bool,

    /// Enable color support for text rendering
    #[arg(long)]
    color: bool,

    /// Restore last saved scroll position for this document
    #[arg(short = 'r', long)]
    restore_position: bool,

    /// Display images inline in terminal (auto-detect capabilities)
    #[arg(long)]
    images: bool,

    /// Force text-only mode for images (no inline display)
    #[arg(long)]
    no_images: bool,

    /// Extract images to a directory
    #[arg(long)]
    extract_images: Option<PathBuf>,

    /// Maximum image width in terminal columns (default: auto-detect)
    #[arg(long, value_name = "COLS")]
    image_width: Option<u32>,

    /// Maximum image height in terminal rows (default: auto-detect)  
    #[arg(long, value_name = "ROWS")]
    image_height: Option<u32>,

    /// Image scaling factor (0.1 to 2.0, default: 1.0)
    #[arg(long, value_name = "SCALE")]
    image_scale: Option<f32>,

    /// Test terminal image capabilities
    #[arg(long)]
    debug_terminal: bool,

    /// Configuration commands
    #[command(subcommand)]
    config: Option<ConfigCommands>,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set configuration value
    Set { key: String, value: String },
    /// Get configuration value
    Get { key: String },
    /// Initialize configuration
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle debug terminal command
    if cli.debug_terminal {
        use terminal_image::TerminalImageRenderer;
        let renderer = TerminalImageRenderer::new();
        renderer.print_capabilities();
        return Ok(());
    }

    match &cli.config {
        Some(ConfigCommands::Init) => {
            let cfg = config::Config::default();
            cfg.save()?;
            println!(
                "Configuration initialized at: {}",
                config::Config::config_file_path()?.display()
            );
            println!("Edit that file to customize keybindings. Valid presets: default, vim, less");
            return Ok(());
        }
        Some(ConfigCommands::Set { key, value }) => {
            let mut cfg = config::Config::load()?;
            cfg.set_value(key, value)?;
            cfg.save()?;
            println!("{key} = {value}");
            return Ok(());
        }
        Some(ConfigCommands::Get { key }) => {
            let cfg = config::Config::load()?;
            match cfg.get_value(key) {
                Some(v) => println!("{v}"),
                None => anyhow::bail!("Unknown config key: {key}"),
            }
            return Ok(());
        }
        None => {}
    }

    let cfg = config::Config::load()?;

    let file_path = cli
        .file
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Please provide a document file to view"))?;

    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let image_options = document::ImageOptions {
        enabled: cli.images,
        max_width: cli.image_width,
        max_height: cli.image_height,
        scale: cli.image_scale,
    };

    // Run CPU-intensive document loading on a blocking thread
    let file_path_clone = file_path.clone();
    let document = tokio::task::spawn_blocking(move || {
        document::load_document(&file_path_clone, image_options)
    })
    .await??;

    // Handle image extraction flag
    if let Some(extract_dir) = &cli.extract_images {
        use image_extractor::ImageExtractor;

        let mut extractor = ImageExtractor::new()?;
        extractor.extract_images_from_docx(&file_path)?;

        // Copy extracted images to the specified directory
        std::fs::create_dir_all(extract_dir)?;
        for (rel_id, temp_path) in extractor.list_images() {
            let target_path = extract_dir.join(rel_id);
            std::fs::copy(temp_path, &target_path)?;
            println!("Extracted: {}", target_path.display());
        }

        println!(
            "Successfully extracted {} images to {}",
            extractor.list_images().len(),
            extract_dir.display()
        );
        return Ok(());
    }

    if let Some(export_format) = &cli.export {
        match export_format {
            ExportFormat::Ansi => {
                export::export_to_ansi_with_cli_options(
                    &document,
                    cli.terminal_width,
                    &cli.color_depth,
                )?;
            }
            _ => {
                export::export_document(&document, export_format)?;
            }
        }
        return Ok(());
    }

    // Start terminal UI
    ui::run_viewer(document, &cli, &cfg).await?;

    Ok(())
}
