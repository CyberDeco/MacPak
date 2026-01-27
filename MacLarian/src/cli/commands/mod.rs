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
                "Invalid layer '{}'. Valid values: 0/BaseMap/BM/Base, 1/NormalMap/NM/Normal, 2/PhysicalMap/PM/Physical",
                s
            )),
        }
    }
}
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

        /// Compression method: raw, lz4, fastlz, best (default: best)
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

impl Commands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Commands::Extract {
                source,
                destination,
                filter,
                file,
                quiet,
                bundle,
                convert_gr2,
                extract_textures,
                extract_virtual_textures,
                game_data,
                virtual_textures,
                delete_gr2,
                png,
                keep_dds,
            } => extract::execute(
                source,
                destination,
                filter.as_deref(),
                file.as_deref(),
                !*quiet,
                extract::Gr2CliOptions {
                    bundle: *bundle,
                    convert_gr2: *convert_gr2,
                    extract_textures: *extract_textures,
                    extract_virtual_textures: *extract_virtual_textures,
                    game_data: game_data.clone(),
                    virtual_textures: virtual_textures.clone(),
                    delete_gr2: *delete_gr2,
                    convert_textures_to_png: *png,
                    keep_original_dds: *keep_dds,
                },
            ),
            Commands::Convert {
                source,
                destination,
                input_format,
                output_format,
            } => convert::execute(
                source,
                destination,
                input_format.as_deref(),
                output_format.as_deref(),
            ),
            Commands::Create { source, destination, compression } => {
                create::execute(source, destination, compression)
            }
            Commands::List {
                source,
                detailed,
                filter,
                count,
            } => list::execute(source, *detailed, filter.as_deref(), *count),
            Commands::Gr2 { command } => command.execute(),
            Commands::VirtualTexture { command } => command.execute(),
            Commands::Mod { command } => command.execute(),
            Commands::Search { command } => command.execute(),
            Commands::Index { command } => command.execute(),
            Commands::Pak { command } => command.execute(),
            Commands::Loca { command } => command.execute(),
            Commands::Texture { command } => command.execute(),
        }
    }
}

impl Gr2Commands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Gr2Commands::Inspect { path } => {
                gr2::inspect(path)
            }
            Gr2Commands::Extract { path, output } => {
                gr2::extract_json(path, output)
            }
            Gr2Commands::Decompress { path, output } => {
                gr2::decompress(path, output.as_deref())
            }
            Gr2Commands::ToGlb { path, output } => {
                gr2::convert_to_glb(path, output.as_deref())
            }
            Gr2Commands::FromGltf { path, output } => {
                gr2::convert_to_gr2(path, output.as_deref())
            }
            Gr2Commands::Bundle { path, output, game_data, virtual_textures, no_glb, no_textures, gltf, png, keep_dds } => {
                gr2::bundle(
                    path,
                    output.as_deref(),
                    game_data.as_deref(),
                    virtual_textures.as_deref(),
                    *no_glb,
                    *no_textures,
                    *gltf,
                    *png,
                    *keep_dds,
                )
            }
            Gr2Commands::ToGlbTextured { path, textures_pak, output } => {
                gr2::convert_to_glb_textured(path, textures_pak, output.as_deref())
            }
        }
    }
}

impl VirtualTextureCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            VirtualTextureCommands::List { path } => {
                virtual_texture::list(path)
            }
            VirtualTextureCommands::Extract { path, output, texture, layer, all_layers } => {
                virtual_texture::extract(
                    path,
                    output,
                    texture.as_deref(),
                    layer.iter().map(|l| l.0).collect(),
                    *all_layers,
                )
            }
            VirtualTextureCommands::Create { name, base, normal, physical, output, compression, no_embed_mip } => {
                virtual_texture::create(
                    name,
                    base.as_deref(),
                    normal.as_deref(),
                    physical.as_deref(),
                    output,
                    compression.as_deref(),
                    *no_embed_mip,
                )
            }
            VirtualTextureCommands::Batch { input, output, layer, recursive } => {
                virtual_texture::batch(input, output, layer.iter().map(|l| l.0).collect(), *recursive)
            }
            VirtualTextureCommands::GtpInfo { path, gts } => {
                virtual_texture::gtp_info(path, gts.as_deref())
            }
        }
    }
}

impl ModCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            ModCommands::Validate { source } => mod_cmd::validate(source),
            ModCommands::InfoJson {
                pak,
                extracted,
                output,
            } => mod_cmd::info_json(pak, extracted, output.as_deref()),
        }
    }
}

impl SearchCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            SearchCommands::FileName {
                pak,
                query,
                type_filter,
            } => search::search_filename(pak, query, type_filter.as_deref()),
            SearchCommands::Path {
                pak,
                query,
                type_filter,
            } => search::search_path(pak, query, type_filter.as_deref()),
            SearchCommands::Uuid { pak, uuid } => search::search_uuid(pak, uuid),
            SearchCommands::Content { pak, query, limit } => {
                search::search_content(pak, query, *limit)
            }
            SearchCommands::FromIndex {
                index_dir,
                query,
                limit,
            } => search::search_index(index_dir, query, *limit),
        }
    }
}

impl IndexCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            IndexCommands::Build {
                paks,
                output,
                fulltext,
            } => {
                // Build index from multiple PAKs
                for pak in paks {
                    search::build_index(pak, Some(output), *fulltext)?;
                }
                Ok(())
            }
            IndexCommands::Stats { index_dir } => search::index_stats(index_dir),
        }
    }
}

impl PakCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            PakCommands::Info { pak } => pak::info(pak),
            PakCommands::Find { dir } => pak::find(dir),
            PakCommands::BatchExtract { source, dest } => pak::batch_extract_cmd(source, dest),
            PakCommands::BatchCreate { source, dest } => pak::batch_create_cmd(source, dest),
        }
    }
}

impl LocaCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            LocaCommands::List { path, limit } => loca::list(path, *limit),
            LocaCommands::Get { path, handle } => loca::get(path, handle),
            LocaCommands::Search { path, query, limit } => loca::search(path, query, *limit),
            LocaCommands::Export { path, output } => loca::export_xml(path, output),
        }
    }
}

impl TextureCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            TextureCommands::Info { path } => texture::info(path),
            TextureCommands::Convert { input, output, format } => {
                texture::convert(input, output, format.as_deref())
            }
            TextureCommands::BatchConvert {
                dir,
                output,
                to,
                dds_format,
            } => texture::batch_convert(dir, output, to, dds_format.as_deref()),
        }
    }
}
