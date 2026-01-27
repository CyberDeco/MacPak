//! MacLarian CLI - Command-line interface for Larian file format tools

pub mod commands;
pub mod progress;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "maclarian")]
#[command(about = "MacLarian: Larian file format tools for BG3", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Run the MacLarian CLI
pub fn run_cli() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    cli.command.execute()?;

    Ok(())
}
