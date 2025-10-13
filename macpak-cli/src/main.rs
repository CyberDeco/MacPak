use clap::{Parser, Subcommand};

mod cli;
mod commands;

use commands::Commands;

#[derive(Parser)]
#[command(name = "macpak")]
#[command(about = "BG3 Modding Toolkit", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    cli.command.execute()?;
    
    Ok(())
}