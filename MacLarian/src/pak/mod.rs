//! PAK archive operations module

pub mod pak_tools;
mod creator;
pub(crate) mod lspk;
mod batch;
mod smart_extract;
mod extractor;
mod lister;

// Primary public API
pub use pak_tools::{PakOperations, ProgressCallback};

// Internal API (used by search module)
pub(crate) use pak_tools::PakReaderCache;
pub use lspk::CompressionMethod;
pub use creator::create_pak;

// Re-export for convenience
pub use extractor::extract_pak;
pub use lister::list_pak_contents;

// Re-export public LSPK types (not internal reader/writer)
pub use lspk::{PakContents, PakFile};

// Re-export batch operations
pub use batch::{
    find_pak_files, find_packable_folders, batch_extract, batch_create, BatchPakResult,
};

// Re-export smart extraction
pub use smart_extract::{extract_files_smart, extract_pak_smart, SmartExtractionResult};

// Re-export Gr2ExtractionOptions from gr2_extraction for convenience
pub use crate::gr2_extraction::Gr2ExtractionOptions;
