//! PAK archive operations module

mod batch;
mod creator;
mod extractor;
mod lister;
pub mod lspk;
pub mod pak_tools;
mod smart_extract;

// Primary public API
pub use pak_tools::{PakOperations, PakReaderCache, ProgressCallback};

// Internal API (used by search module)
pub use creator::create_pak;
pub use lspk::CompressionMethod;

// Re-export for convenience
pub use extractor::extract_pak;
pub use lister::list_pak_contents;

// Re-export public LSPK types (not internal reader/writer)
pub use lspk::{PakContents, PakFile, PakPhase, PakProgress};

// Re-export batch operations
pub use batch::{
    BatchPakResult, batch_create, batch_extract, find_packable_folders, find_pak_files,
};

// Re-export smart extraction
pub use smart_extract::{SmartExtractionResult, extract_files_smart, extract_pak_smart};

// Re-export Gr2ExtractionOptions from gr2_extraction for convenience
pub use crate::gr2_extraction::Gr2ExtractionOptions;
