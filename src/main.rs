use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod document;
mod export;
mod ui;

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

    /// Force interactive UI mode (bypass TTY detection)
    #[arg(long)]
    force_ui: bool,

    /// Enable color support for text rendering
    #[arg(long)]
    color: bool,

    /// Configuration commands
    #[command(subcommand)]
    config: Option<ConfigCommands>,
}

#[derive(clap::ValueEnum, Clone)]
enum ExportFormat {
    Markdown,
    Text,
    Csv,
    Json,
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

    match &cli.config {
        Some(ConfigCommands::Init) => {
            println!("Initializing doxx configuration...");
            // TODO: Initialize config file
            return Ok(());
        }
        Some(ConfigCommands::Set { key, value }) => {
            println!("Setting {key} = {value}");
            // TODO: Set config value
            return Ok(());
        }
        Some(ConfigCommands::Get { key }) => {
            println!("Getting {key}");
            // TODO: Get config value
            return Ok(());
        }
        None => {}
    }

    let file_path = cli
        .file
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Please provide a document file to view"))?;

    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let document = document::load_document(&file_path).await?;

    if let Some(export_format) = &cli.export {
        export::export_document(&document, export_format)?;
        return Ok(());
    }

    // Start terminal UI
    ui::run_viewer(document, &cli).await?;

    Ok(())
}
