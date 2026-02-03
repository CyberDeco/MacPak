//! LSF (Larian Story Format) binary format module
//!
//!

mod document;
mod reader;
mod writer;

// Public API
pub use document::{LsfAttribute, LsfDocument, LsfNode};
pub use reader::{parse_lsf_bytes, read_lsf};
pub use writer::{serialize_lsf, serialize_lsf_with_format, write_lsf, write_lsf_with_format};

// Internal API (used by converter module)
pub(crate) use document::LsfMetadataFormat;
