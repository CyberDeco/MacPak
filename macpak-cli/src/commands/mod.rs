use clap::Subcommand;
use std::path::PathBuf;

pub mod extract;
pub mod convert;
pub mod create;
pub mod list;
pub mod index;
pub mod validate;

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
        
        /// Input format
        #[arg(short = 'i', long, default_value = "lsf")]
        input_format: String,
        
        /// Output format
        #[arg(short = 'o', long, default_value = "lsx")]
        output_format: String,
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
}

impl Commands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Commands::Extract { source, destination } => {
                extract::execute(source, destination)
            }
            Commands::Convert { source, destination, input_format, output_format } => {
                convert::execute(source, destination, input_format, output_format)
            }
            Commands::Create { source, destination } => {
                create::execute(source, destination)
            }
            Commands::List { source } => {
                list::execute(source)
            }
        }
    }
}