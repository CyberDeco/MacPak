use clap::Subcommand;
use std::path::PathBuf;
pub mod extract;
pub mod convert;
pub mod create;
pub mod list;
pub mod index;
pub mod validate;
pub mod gr2;

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
        }
    }
}