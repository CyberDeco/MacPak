use anyhow::{Context, Result};
use clap::Subcommand;
use glob::glob;
use std::path::PathBuf;
use std::str::FromStr;

/// Expand glob patterns in paths (cross-platform)
///
/// If a path contains glob characters (*, ?, [), expands it.
/// Otherwise returns the path as-is (with tilde expansion).
pub fn expand_globs(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut expanded = Vec::new();

    for path in paths {
        let path_str = path.to_string_lossy();

        // Expand tilde first
        let path_str = shellexpand::tilde(&path_str);

        // Check if path contains glob characters
        if path_str.contains('*') || path_str.contains('?') || path_str.contains('[') {
            let matches: Vec<_> = glob(&path_str)
                .with_context(|| format!("Invalid glob pattern: {path_str}"))?
                .filter_map(|r| r.ok())
                .collect();

            if matches.is_empty() {
                anyhow::bail!("No files matched pattern: {path_str}");
            }

            expanded.extend(matches);
        } else {
            expanded.push(PathBuf::from(path_str.as_ref()));
        }
    }

    Ok(expanded)
}

/// Layer specification for virtual texture extraction
#[derive(Debug, Clone, Copy)]
pub struct LayerArg(pub usize);

impl FromStr for LayerArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        match lower.as_str() {
            "0" | "basemap" | "bm" | "base" => Ok(LayerArg(0)),
            "1" | "normalmap" | "nm" | "normal" => Ok(LayerArg(1)),
            "2" | "physicalmap" | "pm" | "physical" => Ok(LayerArg(2)),
            _ => Err(format!(
                "Invalid layer '{s}'. Valid values: 0/BaseMap/BM/Base, 1/NormalMap/NM/Normal, 2/PhysicalMap/PM/Physical"
            )),
        }
    }
}

// Command implementation modules
pub mod convert;
pub mod gr2;
pub mod loca;
pub mod mod_cmd;
pub mod pak;
pub mod texture;
pub mod virtual_texture;

// Command definitions and execution
mod definitions;
mod execute;

// Re-export subcommand enums
pub use definitions::{
    Gr2Commands, LocaCommands, ModCommands, PakCommands, TextureCommands,
    VirtualTextureCommands,
};

#[derive(Subcommand)]
pub enum Commands {
    /// PAK archive operations (extract, create, list)
    Pak {
        #[command(subcommand)]
        command: PakCommands,
    },

    /// Convert file formats (LSF/LSX/LSJ, GR2/GLB, LOCA/XML, DDS/PNG)
    Convert {
        /// Source file(s) or wildcard pattern
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Output file (single source) or directory (multiple sources)
        destination: PathBuf,

        /// Override output format (auto-detected from extension if not specified)
        #[arg(short = 'o', long)]
        output_format: Option<String>,

        /// DDS compression format when converting to DDS (bc1, bc2, bc3, rgba)
        #[arg(long, default_value = "bc3")]
        texture_format: String,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },

    /// GR2 mesh file operations
    Gr2 {
        #[command(subcommand)]
        command: Gr2Commands,
    },

    /// Virtual texture operations (GTS/GTP files)
    #[command(name = "vt")]
    VirtualTexture {
        #[command(subcommand)]
        command: VirtualTextureCommands,
    },

    /// Mod utilities (validation, info.json generation)
    #[command(name = "mods")]
    Mods {
        #[command(subcommand)]
        command: ModCommands,
    },

    /// LOCA localization file operations
    Loca {
        #[command(subcommand)]
        command: LocaCommands,
    },

    /// Texture operations
    Texture {
        #[command(subcommand)]
        command: TextureCommands,
    },
}
