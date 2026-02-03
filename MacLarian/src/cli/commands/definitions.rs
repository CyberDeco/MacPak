//! Subcommand enum definitions for CLI

use clap::Subcommand;
use std::path::PathBuf;

use super::LayerArg;

/// PAK archive commands
#[derive(Subcommand)]
pub enum PakCommands {
    /// Extract files from PAK archive(s)
    #[command(long_about = "Extract files from PAK archive(s)

Extracts files from one or more PAK archives. Supports glob patterns for batch
extraction and filtering by internal file paths.

Examples:
  maclarian pak extract Shared.pak ./output/
  maclarian pak extract \"*.pak\" ./output/
  maclarian pak extract Shared.pak ./output/ -f \"*.lsf\"
  maclarian pak extract Shared.pak ./output/ --file \"Public/Shared/meta.lsx\"")]
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
    #[command(long_about = "Create PAK file(s) from directory(ies)

Packages a directory into a PAK archive. Supports batch creation with glob patterns
and configurable compression.

Compression methods:
  lz4   - Fast compression, good ratio (default)
  zlib  - Better ratio, slower
  none  - No compression

Examples:
  maclarian pak create ./MyMod MyMod.pak
  maclarian pak create ./MyMod MyMod.pak -c lz4
  maclarian pak create \"./Mods/*\" ./output/")]
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
    #[command(long_about = "List contents of a PAK file

Shows files contained in a PAK archive. Use -d for detailed size and compression
info, -f to filter by glob pattern, -c for count only.

Examples:
  maclarian pak list Shared.pak
  maclarian pak list Shared.pak -d
  maclarian pak list Shared.pak -f \"*.lsf\"
  maclarian pak list Shared.pak -f \"Public/**/*\" -c")]
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
    #[command(long_about = "Inspect a GR2 file and display its structure

Displays metadata about a GR2 mesh file including meshes, bones, materials,
and texture references. Output can be saved to JSON for further processing.

Examples:
  maclarian gr2 inspect model.GR2
  maclarian gr2 inspect model.GR2 -o info.json")]
    Inspect {
        /// GR2 file to inspect
        path: PathBuf,

        /// Output to JSON file (prints to CLI if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert GR2 to glTF/GLB format
    #[command(name = "from-gr2")]
    #[command(long_about = "Convert GR2 to glTF/GLB format

Converts Granny2 mesh files to glTF/GLB for editing in Blender or other 3D tools.
Optionally extracts or embeds textures from the game files.

Texture modes:
  extract   - Save textures as separate PNG files alongside the model
  embedded  - Embed textures directly in GLB (GLB format only)

Examples:
  maclarian gr2 from-gr2 model.GR2 model.glb
  maclarian gr2 from-gr2 model.GR2 model.gltf -f gltf
  maclarian gr2 from-gr2 model.GR2 ./output/ --textures extract
  maclarian gr2 from-gr2 \"*.GR2\" ./output/
  maclarian gr2 from-gr2 model.GR2 model.glb --bg3-path /path/to/BG3/Data")]
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
    #[command(long_about = "Convert glTF/GLB to GR2 format

Converts glTF/GLB models back to Granny2 format for use in BG3 mods.
Note: Output is currently uncompressed (Oodle compression not yet implemented).

Examples:
  maclarian gr2 to-gr2 model.glb model.GR2
  maclarian gr2 to-gr2 model.gltf model.GR2
  maclarian gr2 to-gr2 \"*.glb\" ./output/")]
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
    #[command(long_about = "List metadata from a GTS file

Displays information about a virtual texture set including texture names,
dimensions, layer count, and page file references.

Examples:
  maclarian vt list Textures.gts
  maclarian vt list Textures.gts -d
  maclarian vt list Textures.gts -o metadata.json")]
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
    #[command(long_about = "Extract textures from GTS/GTP files to DDS

Extracts virtual textures to DDS files. Can filter by texture name and layer.

Layer names (case-insensitive):
  0, BaseMap, BM, Base       - Albedo/diffuse texture
  1, NormalMap, NM, Normal   - Normal map
  2, PhysicalMap, PM, Physical - PBR physical properties

Examples:
  maclarian vt extract Textures.gts ./output/
  maclarian vt extract Textures.gts ./output/ -t MyTexture
  maclarian vt extract Textures.gts ./output/ --layer BM
  maclarian vt extract Textures.gts ./output/ --layer BM,NM,PM
  maclarian vt extract \"*.gts\" ./output/")]
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
    #[command(long_about = "Create a virtual texture set from DDS source textures

Creates GTS/GTP virtual texture files from DDS source textures. Auto-detects
layer types from common suffixes (_BM, _NM, _PM) or specify paths manually.

Note: Virtual texture injection requires BG3 Script Extender (Windows-only).
macOS users should be aware of this limitation before creating custom textures.

Examples:
  maclarian vt create ./textures/ ./output/
  maclarian vt create ./textures/ ./output/ -t MyTexture
  maclarian vt create ./textures/ ./output/ --base albedo.dds --normal normal.dds
  maclarian vt create ./textures/ ./output/ -c raw")]
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
    #[command(long_about = "Search for entries in a LOCA file

Searches localization files for text content or handles. LOCA files contain
game text strings indexed by UUID handles.

Examples:
  maclarian loca search English.loca \"Shadowheart\"
  maclarian loca search English.loca \"hello\" -l 100
  maclarian loca search English.loca \"h7a8b9c0\" --handle")]
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
    #[command(long_about = "Show info about a DDS texture file

Displays metadata about a DDS texture including dimensions, format,
mip levels, and compression type.

Examples:
  maclarian texture info albedo.dds")]
    Info {
        /// DDS file to analyze
        path: PathBuf,
    },
}

/// Mod utility commands
#[derive(Subcommand)]
pub enum ModCommands {
    /// Validate mod structure and PAK integrity
    #[command(long_about = "Validate mod structure and PAK integrity

Checks that a mod has the correct directory structure, valid meta.lsx,
and that PAK files are not corrupted. Supports glob patterns for batch validation.

Examples:
  maclarian mods validate MyMod.pak
  maclarian mods validate ./MyModFolder/
  maclarian mods validate \"*.pak\"")]
    Validate {
        /// Path(s) to mod directory or PAK file(s) - supports glob patterns
        #[arg(required = true)]
        source: Vec<PathBuf>,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Package mod for `BaldursModManager` (generates info.json alongside PAK)
    #[command(
        long_about = "Package mod for BaldursModManager (generates info.json alongside PAK)

Creates a distribution-ready mod package with proper folder structure and
info.json for mod managers. Optionally compresses to zip or 7z archive.

Output structure:
  <destination>/ModName/
    ├── ModName.pak
    └── info.json

Examples:
  maclarian mods package ./MyMod ./dist/
  maclarian mods package MyMod.pak ./dist/ -c zip
  maclarian mods package ./MyMod ./dist/ -c 7z"
    )]
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
    #[command(long_about = "Generate meta.lsx metadata file for a mod

Creates the required meta.lsx file for a BG3 mod with proper UUID, version,
and mod info. The file is placed in the correct location within the mod structure.

Examples:
  maclarian mods meta ./MyMod -n \"My Cool Mod\" -a \"Author Name\"
  maclarian mods meta ./MyMod -n \"My Mod\" -a \"Author\" -d \"A description\"
  maclarian mods meta ./MyMod -n \"My Mod\" -a \"Author\" -v 1.2.0.0")]
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

    /// Find files modified by multiple mods (potential conflicts)
    #[command(
        long_about = "Find files modified by multiple mods (potential conflicts)

Compares two or more mods and identifies files that are modified by multiple mods,
which may cause conflicts when loaded together. Useful for troubleshooting load order.

Examples:
  maclarian mods conflicts Mod1.pak Mod2.pak
  maclarian mods conflicts Mod1.pak Mod2.pak Mod3.pak
  maclarian mods conflicts ./Mod1/ ./Mod2/"
    )]
    Conflicts {
        /// PAK files or mod directories to compare (2 or more)
        #[arg(required = true, num_args = 2..)]
        sources: Vec<PathBuf>,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },
}
