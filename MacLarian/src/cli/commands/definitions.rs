//! Subcommand enum definitions for CLI

use clap::Subcommand;
use std::path::PathBuf;

use super::LayerArg;

/// PAK archive commands
#[derive(Subcommand)]
pub enum PakCommands {
    /// Extract files from PAK archive(s)
    Extract {
        /// Source PAK file(s) or wildcard pattern
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Output directory
        destination: PathBuf,

        /// Only extract files matching glob pattern (e.g., "*.lsf")
        #[arg(short = 'f', long, conflicts_with = "file")]
        filter: Option<String>,

        /// Extract specific file(s) by internal path (comma-separated)
        #[arg(long, conflicts_with = "filter")]
        file: Option<String>,

        /// Suppress progress bar
        #[arg(short, long)]
        quiet: bool,
    },

    /// Create PAK file(s) from directory(ies)
    Create {
        /// Source directory(ies) to pack (supports wildcards)
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Output PAK file (single source) or directory (multiple sources)
        destination: PathBuf,

        /// Compression method (lz4, zlib, none)
        #[arg(short, long, default_value = "lz4")]
        compression: String,

        /// Suppress progress bar
        #[arg(short, long)]
        quiet: bool,
    },

    /// List contents of a PAK file
    List {
        /// PAK file
        source: PathBuf,

        /// Show detailed info (sizes, compression ratio)
        #[arg(short, long)]
        detailed: bool,

        /// Filter by glob pattern (auto-detects and normalizes UUIDs)
        #[arg(short = 'f', long)]
        filter: Option<String>,

        /// Only show count of matching files
        #[arg(short, long)]
        count: bool,

        /// Suppress extra output
        #[arg(short, long)]
        quiet: bool,
    },
}

/// GR2 mesh file commands
#[derive(Subcommand)]
pub enum Gr2Commands {
    /// Inspect a GR2 file and display its structure
    Inspect {
        /// GR2 file to inspect
        path: PathBuf,

        /// Output to JSON file (prints to CLI if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert GR2 to glTF/GLB format
    #[command(name = "from-gr2")]
    FromGr2 {
        /// Source GR2 file(s) or wildcard pattern
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Output file (single source) or directory (multiple sources)
        destination: PathBuf,

        /// Output format (glb or gltf)
        #[arg(short = 'f', long, default_value = "glb")]
        format: String,

        /// Texture handling: "extract" (separate files) or "embedded" (in GLB)
        #[arg(long)]
        textures: Option<String>,

        /// Path to BG3 install folder (required for --textures if not auto-detected)
        #[arg(long = "bg3-path")]
        bg3_path: Option<PathBuf>,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Convert glTF/GLB to GR2 format
    #[command(name = "to-gr2")]
    ToGr2 {
        /// Source GLB or glTF file(s) or wildcard pattern
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Output GR2 file (single source) or directory (multiple sources)
        destination: PathBuf,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },
}

/// Virtual Texture (GTS/GTP) commands
#[derive(Subcommand)]
pub enum VirtualTextureCommands {
    /// List metadata from a GTS file
    List {
        /// Path to .gts file
        path: PathBuf,

        /// Show full page file list (summary only by default)
        #[arg(short, long)]
        detailed: bool,

        /// Output to JSON file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Extract textures from GTS/GTP files to DDS
    Extract {
        /// Source GTS/GTP file(s) or wildcard pattern
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Output directory for DDS files
        destination: PathBuf,

        /// Extract only this texture by name
        #[arg(short = 't', long = "gtex")]
        gtex: Option<String>,

        /// Layer(s) to extract: 0/BaseMap/BM/Base, 1/NormalMap/NM/Normal, 2/PhysicalMap/PM/Physical
        /// Can be specified multiple times (--layer BM --layer NM) or comma-separated (--layer BM,NM)
        #[arg(short, long, value_delimiter = ',')]
        layer: Vec<LayerArg>,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Create a virtual texture set from DDS source textures
    Create {
        /// Source directory containing DDS files
        source: PathBuf,

        /// Output directory (for .gts and .gtp files)
        destination: PathBuf,

        /// Output texture name(s), comma-separated (defaults to GTP filename)
        #[arg(short = 't', long = "gtex")]
        gtex: Option<String>,

        /// Path to base map DDS (if not auto-detected)
        #[arg(long)]
        base: Option<PathBuf>,

        /// Path to normal map DDS (if not auto-detected)
        #[arg(long)]
        normal: Option<PathBuf>,

        /// Path to physical map DDS (if not auto-detected)
        #[arg(long)]
        physical: Option<PathBuf>,

        /// Compression method: raw, fastlz (default: fastlz)
        #[arg(short, long, default_value = "fastlz")]
        compression: String,

        /// Disable embedding mip levels in tiles (use for DDS without mips)
        #[arg(long)]
        no_embed_mip: bool,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },
}

/// LOCA localization file commands
#[derive(Subcommand)]
pub enum LocaCommands {
    /// Search for entries in a LOCA file
    Search {
        /// LOCA file to search
        path: PathBuf,

        /// Search term
        query: String,

        /// Search handles instead of text content
        #[arg(long)]
        handle: bool,

        /// Maximum results to return
        #[arg(short, long, default_value = "50")]
        limit: usize,

        /// Suppress extra output
        #[arg(short, long)]
        quiet: bool,
    },
}

/// Texture operation commands
#[derive(Subcommand)]
pub enum TextureCommands {
    /// Show info about a DDS texture file
    Info {
        /// DDS file to analyze
        path: PathBuf,
    },
}

/// Mod utility commands
#[derive(Subcommand)]
pub enum ModCommands {
    /// Validate mod structure, PAK integrity, or directory for PAK creation
    Validate {
        /// Path(s) to mod directory, PAK file(s), or folder containing mods
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Recursively scan directory for all mods
        #[arg(short, long)]
        recursive: bool,

        /// Check PAK file integrity (verify all files can be read/decompressed)
        #[arg(short = 'i', long)]
        check_integrity: bool,

        /// Validate directory can be packed into a PAK
        #[arg(short = 'd', long)]
        dry_run: bool,

        /// Only check PAK files (skip directories)
        #[arg(long)]
        paks_only: bool,

        /// Only check directories (skip PAK files)
        #[arg(long)]
        dirs_only: bool,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Package mod for BaldursModManager (generates info.json alongside PAK)
    Package {
        /// Path to .pak file or mod directory
        source: PathBuf,

        /// Output directory (creates `<destination>/ModName/ModName.pak` + `info.json`)
        destination: PathBuf,

        /// Compress output as zip or 7z
        #[arg(short, long, value_parser = ["zip", "7z"])]
        compress: Option<String>,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Generate meta.lsx metadata file for a mod
    Meta {
        /// Mod source directory (creates `<source>/Mods/<Folder>/meta.lsx`)
        source: PathBuf,

        /// Mod display name
        #[arg(short, long)]
        name: String,

        /// Author name
        #[arg(short, long)]
        author: String,

        /// Mod description
        #[arg(short, long, default_value = "")]
        description: String,

        /// Folder name (defaults to sanitized mod name)
        #[arg(short, long)]
        folder: Option<String>,

        /// UUID (auto-generated if not provided)
        #[arg(short, long)]
        uuid: Option<String>,

        /// Version in format "major.minor.patch.build" (default: 1.0.0.0)
        #[arg(short, long, default_value = "1.0.0.0")]
        version: String,
    },

    /// Discover virtual textures defined in mod config files
    Vtex {
        /// Mod directory(ies) to scan (can be mod roots or folders containing mods)
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Output to JSON file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Suppress extra output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Find files modified by multiple mods (potential conflicts)
    Conflicts {
        /// PAK files or mod directories to compare (2 or more)
        #[arg(required = true, num_args = 2..)]
        sources: Vec<PathBuf>,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },
}
