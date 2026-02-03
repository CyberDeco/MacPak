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
                .filter_map(Result::ok)
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
    #[command(long_about = "PAK archive operations (extract, create, list)

Work with BG3's LSPK package format. Supports batch operations with glob patterns.

Examples:
  maclarian pak list Shared.pak
  maclarian pak list Shared.pak -d -f \"*.lsf\"
  maclarian pak extract Shared.pak ./output
  maclarian pak extract \"*.pak\" ./output -f \"Public/**/*.lsf\"
  maclarian pak create ./MyMod MyMod.pak -c lz4")]
    Pak {
        #[command(subcommand)]
        command: PakCommands,
    },

    /// Convert file formats (LSF/LSX/LSJ, GR2/GLB, LOCA/XML, DDS/PNG)
    #[command(long_about = "Convert file formats (LSF/LSX/LSJ, GR2/GLB, LOCA/XML, DDS/PNG)

Auto-detects input/output formats from file extensions. Supports batch conversion
with glob patterns. Output format can be overridden with -o/--output-format.

Supported conversions:
  LSF <-> LSX    Binary to/from XML document format
  LSF <-> LSJ    Binary to/from JSON document format
  LSX <-> LSJ    XML to/from JSON document format
  GR2 <-> GLB    Granny2 mesh to/from glTF binary
  GR2 <-> glTF   Granny2 mesh to/from glTF
  LOCA <-> XML   Localization binary to/from XML
  DDS <-> PNG    DirectDraw Surface to/from PNG image

Examples:
  maclarian convert meta.lsf meta.lsx
  maclarian convert meta.lsx meta.lsj
  maclarian convert \"*.lsf\" ./output/
  maclarian convert texture.dds texture.png
  maclarian convert texture.png texture.dds --texture-format bc3")]
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
    #[command(long_about = "GR2 mesh file operations

Convert between Granny2 (.GR2) and glTF/GLB formats for 3D model editing.
Supports texture extraction and embedding for complete model export.

Examples:
  maclarian gr2 inspect model.GR2
  maclarian gr2 from-gr2 model.GR2 model.glb
  maclarian gr2 from-gr2 model.GR2 model.glb --textures extract
  maclarian gr2 from-gr2 \"*.GR2\" ./output/ -f gltf
  maclarian gr2 to-gr2 model.glb model.GR2")]
    Gr2 {
        #[command(subcommand)]
        command: Gr2Commands,
    },

    /// Virtual texture operations (GTS/GTP files)
    #[command(name = "vt")]
    #[command(long_about = "Virtual texture operations (GTS/GTP files)

Work with BG3's streaming virtual texture system. GTS files contain metadata,
GTP files contain the actual texture data pages.

Note: Creating custom virtual textures requires BG3 Script Extender for injection,
which is Windows-only. macOS users can extract but may have limited use for creation.

Examples:
  maclarian vt list Textures.gts
  maclarian vt list Textures.gts -d -o metadata.json
  maclarian vt extract Textures.gts ./output/
  maclarian vt extract Textures.gts ./output/ -t MyTexture --layer BM,NM
  maclarian vt create ./textures/ ./output/ -t MyTexture")]
    VirtualTexture {
        #[command(subcommand)]
        command: VirtualTextureCommands,
    },

    /// Mod utilities (validation, info.json generation)
    #[command(name = "mods")]
    #[command(long_about = "Mod utilities (validation, info.json generation)

Tools for mod development: validate structure, generate metadata, package for
distribution, and detect conflicts between mods.

Examples:
  maclarian mods validate MyMod.pak
  maclarian mods validate ./MyModFolder/
  maclarian mods meta ./MyMod -n \"My Mod\" -a \"Author\"
  maclarian mods package ./MyMod ./dist/ -c zip
  maclarian mods conflicts Mod1.pak Mod2.pak Mod3.pak")]
    Mods {
        #[command(subcommand)]
        command: ModCommands,
    },

    /// LOCA localization file operations
    #[command(long_about = "LOCA localization file operations

Search and work with BG3's localization files. LOCA files contain text strings
with associated handles (UUIDs) used throughout the game.

Examples:
  maclarian loca search English.loca \"Shadowheart\"
  maclarian loca search English.loca \"h12345\" --handle
  maclarian loca search English.loca \"quest\" -l 100")]
    Loca {
        #[command(subcommand)]
        command: LocaCommands,
    },

    /// Texture operations
    #[command(long_about = "Texture operations

Inspect and analyze DDS texture files.

Examples:
  maclarian texture info albedo.dds")]
    Texture {
        #[command(subcommand)]
        command: TextureCommands,
    },
}
