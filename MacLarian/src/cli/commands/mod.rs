use clap::Subcommand;
use std::path::PathBuf;
use std::str::FromStr;

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
pub mod create;
pub mod extract;
pub mod gr2;
pub mod list;
pub mod loca;
pub mod mod_cmd;
pub mod pak;
pub mod search;
pub mod texture;
pub mod virtual_texture;

// Command definitions and execution
mod definitions;
mod execute;

// Re-export subcommand enums
pub use definitions::{
    Gr2Commands, IndexCommands, LocaCommands, ModCommands, PakCommands, SearchCommands,
    TextureCommands, VirtualTextureCommands,
};

#[derive(Subcommand)]
pub enum Commands {
    /// Extract a PAK file
    Extract {
        /// Source PAK file
        #[arg(short, long)]
        source: PathBuf,

        /// Output directory
        #[arg(short, long)]
        destination: PathBuf,

        /// Only extract files matching glob pattern (e.g., "*.lsf", "*_merged.lsf")
        #[arg(long, conflicts_with = "file")]
        filter: Option<String>,

        /// Extract a single file by internal path
        #[arg(long, conflicts_with = "filter")]
        file: Option<String>,

        /// Suppress progress bar
        #[arg(short, long)]
        quiet: bool,

        // GR2 processing options

        /// Enable all GR2 processing (convert to GLB, extract textures and virtual textures)
        #[arg(long)]
        bundle: bool,

        /// Convert extracted GR2 files to GLB format
        #[arg(long)]
        convert_gr2: bool,

        /// Extract DDS textures associated with GR2 files
        #[arg(long)]
        extract_textures: bool,

        /// Extract virtual textures associated with GR2 files
        #[arg(long)]
        extract_virtual_textures: bool,

        /// Path to BG3 install folder (containing Textures.pak, Shared.pak, etc.)
        #[arg(long = "bg3-path")]
        game_data: Option<PathBuf>,

        /// Path to pre-extracted virtual textures (GTP/GTS files)
        #[arg(long)]
        virtual_textures: Option<PathBuf>,

        /// Delete original GR2 files after GLB conversion (keeps by default)
        #[arg(long)]
        delete_gr2: bool,

        /// Convert extracted DDS textures to PNG format
        #[arg(long)]
        png: bool,

        /// Keep original DDS files after PNG conversion
        #[arg(long)]
        keep_dds: bool,
    },

    /// Convert file formats
    Convert {
        /// Source file
        #[arg(short, long)]
        source: PathBuf,

        /// Destination file
        #[arg(short, long)]
        destination: PathBuf,

        /// Input format (auto-detected from extension if not specified)
        #[arg(short = 'i', long)]
        input_format: Option<String>,

        /// Output format (auto-detected from extension if not specified)
        #[arg(short = 'o', long)]
        output_format: Option<String>,
    },

    /// Create a PAK file
    Create {
        /// Source directory
        #[arg(short, long)]
        source: PathBuf,

        /// Output PAK file
        #[arg(short, long)]
        destination: PathBuf,

        /// Compression method (lz4, zlib, none). Default: lz4
        #[arg(short, long, default_value = "lz4")]
        compression: String,
    },

    /// List PAK contents
    List {
        /// PAK file
        #[arg(short, long)]
        source: PathBuf,

        /// Show detailed info (sizes, compression ratio)
        #[arg(short, long)]
        detailed: bool,

        /// Only list files matching glob pattern (e.g., "*.gr2")
        #[arg(long)]
        filter: Option<String>,

        /// Only show count of matching files
        #[arg(short, long)]
        count: bool,
    },

    /// GR2 file operations
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
    Mod {
        #[command(subcommand)]
        command: ModCommands,
    },

    /// Search PAK file contents
    Search {
        #[command(subcommand)]
        command: SearchCommands,
    },

    /// Build and manage search indexes
    Index {
        #[command(subcommand)]
        command: IndexCommands,
    },

    /// PAK file utilities (batch operations, info)
    Pak {
        #[command(subcommand)]
        command: PakCommands,
    },

    /// LOCA localization file operations
    Loca {
        #[command(subcommand)]
        command: LocaCommands,
    },

    /// Texture operations (DDS/PNG conversion)
    Texture {
        #[command(subcommand)]
        command: TextureCommands,
    },
}
