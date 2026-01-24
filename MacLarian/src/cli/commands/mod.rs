use clap::Subcommand;
use std::path::PathBuf;
pub mod extract;
pub mod convert;
pub mod create;
pub mod list;
pub mod gr2;
pub mod virtual_texture;
#[cfg(feature = "audio")]
pub mod wem;

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
    },

    /// List PAK contents
    List {
        /// PAK file
        #[arg(short, long)]
        source: PathBuf,
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

    /// WEM audio file operations
    #[cfg(feature = "audio")]
    Wem {
        #[command(subcommand)]
        command: WemCommands,
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

    /// Convert GR2 to GLB and extract associated textures
    Bundle {
        /// Source GR2 file
        path: PathBuf,

        /// Output directory (defaults to same directory as GR2)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to BG3 game data folder (for Textures.pak)
        #[arg(long)]
        game_data: Option<PathBuf>,

        /// Path to pre-extracted virtual textures (GTP/GTS files)
        #[arg(long)]
        virtual_textures: Option<PathBuf>,

        /// Skip GLB conversion (only extract textures)
        #[arg(long)]
        no_glb: bool,

        /// Skip texture extraction (only convert to GLB)
        #[arg(long)]
        no_textures: bool,
    },

    /// Convert GR2 to GLB with embedded textures (test command)
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

/// WEM audio commands
#[cfg(feature = "audio")]
#[derive(Subcommand)]
pub enum WemCommands {
    /// Inspect a WEM file header
    Inspect {
        /// WEM file to inspect
        path: PathBuf,
    },

    /// Decode a WEM file to WAV (requires vgmstream-cli)
    Decode {
        /// Source WEM file
        path: PathBuf,

        /// Output WAV file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Use silent fallback if vgmstream unavailable (outputs silence with correct duration)
        #[arg(long)]
        silent: bool,
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
        /// Path to .gts file
        gts: PathBuf,

        /// Output directory for DDS files
        #[arg(short, long)]
        output: PathBuf,

        /// Directory containing GTP files (defaults to GTS directory)
        #[arg(long)]
        gtp_dir: Option<PathBuf>,

        /// Extract only this texture (by name)
        #[arg(short, long)]
        texture: Option<String>,

        /// Layer index to extract (ignored if --all-layers is set)
        #[arg(short, long)]
        layer: Option<usize>,

        /// Extract all layers (creates _0, _1, _2 files per texture)
        #[arg(short, long)]
        all_layers: bool,
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

impl Commands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Commands::Extract { source, destination } => {
                extract::execute(source, destination)
            }
            Commands::Convert { source, destination, input_format, output_format } => {
                convert::execute(
                    source,
                    destination,
                    input_format.as_deref(),
                    output_format.as_deref()
                )
            }
            Commands::Create { source, destination } => {
                create::execute(source, destination)
            }
            Commands::List { source } => {
                list::execute(source)
            }
            Commands::Gr2 { command } => {
                command.execute()
            }
            Commands::VirtualTexture { command } => {
                command.execute()
            }
            #[cfg(feature = "audio")]
            Commands::Wem { command } => {
                command.execute()
            }
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
            Gr2Commands::Bundle { path, output, game_data, virtual_textures, no_glb, no_textures } => {
                gr2::bundle(
                    path,
                    output.as_deref(),
                    game_data.as_deref(),
                    virtual_textures.as_deref(),
                    *no_glb,
                    *no_textures,
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
            VirtualTextureCommands::Extract { gts, output, gtp_dir, texture, layer, all_layers } => {
                virtual_texture::extract(
                    gts,
                    gtp_dir.as_deref(),
                    output,
                    texture.as_deref(),
                    *layer,
                    *all_layers,
                )
            }
            VirtualTextureCommands::GtpInfo { path, gts } => {
                virtual_texture::gtp_info(path, gts.as_deref())
            }
        }
    }
}

#[cfg(feature = "audio")]
impl WemCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            WemCommands::Inspect { path } => {
                wem::inspect(path)
            }
            WemCommands::Decode { path, output, silent } => {
                wem::decode(path, output.as_deref(), *silent)
            }
        }
    }
}
