//! Subcommand enum definitions for CLI

use clap::Subcommand;
use std::path::PathBuf;

use super::LayerArg;

/// Mod utility commands
#[derive(Subcommand)]
pub enum ModCommands {
    /// Validate mod directory structure
    Validate {
        /// Path to mod directory (extracted PAK contents)
        #[arg(short, long)]
        source: PathBuf,
    },

    /// Generate info.json for BaldursModManager
    InfoJson {
        /// Path to PAK file (for MD5 calculation)
        #[arg(long)]
        pak: PathBuf,

        /// Path to extracted mod directory (for meta.lsx)
        #[arg(long)]
        extracted: PathBuf,

        /// Output file (prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum Gr2Commands {
    /// Inspect a GR2 file and display its structure
    Inspect {
        /// GR2 file to inspect
        path: PathBuf,
    },

    /// Extract mesh information to JSON
    Extract {
        /// Source GR2 file
        path: PathBuf,

        /// Output JSON file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Decompress a GR2 file (all BitKnit sections)
    Decompress {
        /// Source GR2 file
        path: PathBuf,

        /// Output file (defaults to same directory with _decompressed suffix)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert GR2 to GLB (binary glTF) format
    ToGlb {
        /// Source GR2 file
        path: PathBuf,

        /// Output GLB file (defaults to same name with .glb extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert GLB/glTF to GR2 format
    FromGltf {
        /// Source glTF/GLB file
        path: PathBuf,

        /// Output GR2 file (defaults to same name with .gr2 extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert GR2 to GLB/glTF and extract associated textures
    Bundle {
        /// Source GR2 file
        path: PathBuf,

        /// Output directory (defaults to same directory as GR2)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to BG3 install folder (containing Textures.pak)
        #[arg(long = "bg3-path")]
        game_data: Option<PathBuf>,

        /// Path to pre-extracted virtual textures (GTP/GTS files)
        #[arg(long)]
        virtual_textures: Option<PathBuf>,

        /// Skip GLB/glTF conversion (only extract textures)
        #[arg(long)]
        no_glb: bool,

        /// Skip texture extraction (only convert to GLB/glTF)
        #[arg(long)]
        no_textures: bool,

        /// Output as glTF instead of GLB (outputs .gltf + .bin files)
        #[arg(long)]
        gltf: bool,

        /// Convert extracted DDS textures to PNG format
        #[arg(long)]
        png: bool,

        /// Keep original DDS files after PNG conversion
        #[arg(long)]
        keep_dds: bool,
    },

    /// Convert GR2 to GLB with embedded textures
    ToGlbTextured {
        /// Source GR2 file
        path: PathBuf,

        /// Path to Textures.pak
        #[arg(long)]
        textures_pak: PathBuf,

        /// Output GLB file (defaults to same name with .textured.glb extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Virtual Texture (GTS/GTP) commands
#[derive(Subcommand)]
pub enum VirtualTextureCommands {
    /// List textures in a GTS file
    List {
        /// Path to .gts file
        path: PathBuf,
    },

    /// Extract textures from GTS/GTP files to DDS
    Extract {
        /// Path to .gts or .gtp file
        path: PathBuf,

        /// Output directory for DDS files
        #[arg(short, long)]
        output: PathBuf,

        /// Extract only this texture (by name)
        #[arg(short, long)]
        texture: Option<String>,

        /// Layer(s) to extract: 0/BaseMap/BM/Base, 1/NormalMap/NM/Normal, 2/PhysicalMap/PM/Physical
        /// Can be specified multiple times (--layer BM --layer NM) or comma-separated (--layer BM,NM)
        #[arg(short, long, value_delimiter = ',')]
        layer: Vec<LayerArg>,

        /// Extract all layers (creates _0, _1, _2 files per texture)
        #[arg(short, long)]
        all_layers: bool,
    },

    /// Create a virtual texture set from DDS source textures
    Create {
        /// Name for the virtual texture
        #[arg(short, long)]
        name: String,

        /// Path to base map DDS (color/albedo)
        #[arg(long)]
        base: Option<PathBuf>,

        /// Path to normal map DDS
        #[arg(long)]
        normal: Option<PathBuf>,

        /// Path to physical map DDS (roughness/metallic)
        #[arg(long)]
        physical: Option<PathBuf>,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Compression method: raw, fastlz (default: fastlz)
        #[arg(short, long)]
        compression: Option<String>,

        /// Disable embedding mip levels in tiles (use for DDS without mips)
        #[arg(long)]
        no_embed_mip: bool,
    },

    /// Batch extract multiple GTS files in parallel
    Batch {
        /// Input directory containing GTS files
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for extracted textures
        #[arg(short, long)]
        output: PathBuf,

        /// Layer(s) to extract: 0/BaseMap/BM/Base, 1/NormalMap/NM/Normal, 2/PhysicalMap/PM/Physical (default: all)
        /// Can be specified multiple times (--layer BM --layer NM) or comma-separated (--layer BM,NM)
        #[arg(short, long, value_delimiter = ',')]
        layer: Vec<LayerArg>,

        /// Search subdirectories recursively
        #[arg(short, long)]
        recursive: bool,
    },

    /// Show info about a GTP page file
    GtpInfo {
        /// Path to .gtp file
        path: PathBuf,

        /// Path to .gts file (auto-detected if not specified)
        #[arg(long)]
        gts: Option<PathBuf>,
    },
}

/// Search commands
#[derive(Subcommand)]
pub enum SearchCommands {
    /// Search for files by filename (case-insensitive)
    #[command(name = "filename")]
    FileName {
        /// PAK file to search
        pak: PathBuf,

        /// Search term
        query: String,

        /// Filter by file type (lsx, lsf, gr2, dds, etc.)
        #[arg(short = 't', long = "type")]
        type_filter: Option<String>,
    },

    /// Search for files by path (case-insensitive substring match)
    Path {
        /// PAK file to search
        pak: PathBuf,

        /// Search term
        query: String,

        /// Filter by file type (lsx, lsf, gr2, dds, etc.)
        #[arg(short = 't', long = "type")]
        type_filter: Option<String>,
    },

    /// Search for files by UUID (handles various formats)
    #[command(name = "uuid")]
    Uuid {
        /// PAK file to search
        pak: PathBuf,

        /// UUID to search for
        uuid: String,
    },

    /// Full-text content search (slower, searches file contents)
    Content {
        /// PAK file to search
        pak: PathBuf,

        /// Search query
        query: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Search using a pre-built index (faster for repeated searches)
    #[command(name = "index")]
    FromIndex {
        /// Directory containing the exported index
        index_dir: PathBuf,

        /// Search query
        query: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
}

/// Index commands
#[derive(Subcommand)]
pub enum IndexCommands {
    /// Build a search index from PAK files
    Build {
        /// PAK file(s) to index
        #[arg(required = true)]
        paks: Vec<PathBuf>,

        /// Output directory for the index
        #[arg(short, long)]
        output: PathBuf,

        /// Build full-text index (slower, enables content search)
        #[arg(long)]
        fulltext: bool,
    },

    /// Show statistics about an index
    Stats {
        /// Directory containing the index
        index_dir: PathBuf,
    },
}

/// PAK utility commands
#[derive(Subcommand)]
pub enum PakCommands {
    /// Show detailed info about a PAK file (file counts, compression stats)
    Info {
        /// PAK file to analyze
        pak: PathBuf,
    },

    /// Find all PAK files in a directory
    Find {
        /// Directory to search
        dir: PathBuf,
    },

    /// Batch extract multiple PAK files
    BatchExtract {
        /// Source directory containing PAK files
        #[arg(short, long)]
        source: PathBuf,

        /// Destination directory for extracted files
        #[arg(short, long)]
        dest: PathBuf,
    },

    /// Batch create PAK files from folders
    BatchCreate {
        /// Source directory containing folders to pack
        #[arg(short, long)]
        source: PathBuf,

        /// Destination directory for PAK files
        #[arg(short, long)]
        dest: PathBuf,
    },
}

/// LOCA localization file commands
#[derive(Subcommand)]
pub enum LocaCommands {
    /// List entries in a LOCA file
    List {
        /// LOCA file to read
        path: PathBuf,

        /// Maximum entries to display
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Get a specific entry by handle/key
    Get {
        /// LOCA file to read
        path: PathBuf,

        /// Handle or partial key to search for
        handle: String,
    },

    /// Search for entries containing text
    Search {
        /// LOCA file to read
        path: PathBuf,

        /// Text to search for
        query: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Export LOCA file to XML format
    Export {
        /// Source LOCA file
        path: PathBuf,

        /// Output XML file
        #[arg(short, long)]
        output: PathBuf,
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

    /// Convert a texture file (DDS<->PNG)
    Convert {
        /// Input file (DDS or PNG)
        input: PathBuf,

        /// Output file (PNG or DDS)
        output: PathBuf,

        /// DDS format when converting to DDS (bc1, bc2, bc3, rgba)
        #[arg(short, long, default_value = "bc3")]
        format: Option<String>,
    },

    /// Batch convert textures in a directory
    BatchConvert {
        /// Directory containing textures
        #[arg(short, long)]
        dir: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Target format (png or dds)
        #[arg(short, long)]
        to: String,

        /// DDS format when converting to DDS (bc1, bc2, bc3, rgba)
        #[arg(long)]
        dds_format: Option<String>,
    },
}
