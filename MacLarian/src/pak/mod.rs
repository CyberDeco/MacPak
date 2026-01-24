//! PAK archive operations module

pub mod pak_tools;
pub mod creator;
pub mod lspk;
pub mod batch;
mod extractor;
mod lister;

pub use pak_tools::{PakOperations, PakReaderCache, ProgressCallback};
pub use lspk::CompressionMethod;
pub use creator::create_pak;

// Re-export for convenience
pub use extractor::extract_pak;
pub use lister::list_pak_contents;

// Re-export LSPK reader types
pub use lspk::{FileTableEntry, LspkReader, PakContents, PakFile, PakPhase, PakProgress};

// Re-export batch operations
pub use batch::{
    find_pak_files, find_packable_folders, batch_extract, batch_create, BatchPakResult,
};
