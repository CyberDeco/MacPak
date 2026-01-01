//! MacPak CLI - Command-line interface for BG3 modding toolkit

pub mod commands;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "macpak-cli")]
#[command(about = "MacPak: a BG3 modding toolkit for macOS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Run the MacPak CLI
pub fn run_cli() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    cli.command.execute()?;

    Ok(())
}
