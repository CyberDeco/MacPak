use clap::Parser;

mod cli;
mod commands;

use commands::Commands;

#[derive(Parser)]
#[command(name = "macpak-cli")]
#[command(about = "MacPak: a BG3 modding toolkit for macOS", long_about = None)]
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
