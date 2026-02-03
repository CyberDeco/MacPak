//! `MacLarian` CLI - Command-line interface for Larian file format tools

pub mod commands;
pub mod progress;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "maclarian")]
#[command(about = "MacLarian: Larian file format tools for BG3")]
#[command(long_about = "MacLarian: Larian file format tools for BG3

A pure-Rust toolkit for working with Baldur's Gate 3 file formats on macOS.
Supports PAK archives, document formats (LSF/LSX/LSJ), 3D models (GR2/glTF),
virtual textures (GTS/GTP), localization (LOCA), and mod utilities.

Examples:
  maclarian pak list Shared.pak
  maclarian pak extract Shared.pak ./output
  maclarian convert meta.lsf meta.lsx
  maclarian gr2 from-gr2 model.GR2 model.glb

Documentation: https://github.com/CyberDeco/MacPak/wiki/MacLarian-CLI-Commands")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Run the `MacLarian` CLI
pub fn run_cli() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    cli.command.execute()?;

    Ok(())
}
