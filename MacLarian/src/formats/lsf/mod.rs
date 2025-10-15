//! LSF (Larian Story Format) binary format module

mod document;
mod reader;
mod writer;
pub mod string_table;  // Public because it's used in utils and tests

// Public API
pub use document::{LsfDocument, LsfNode, LsfAttribute};
pub use reader::{read_lsf, parse_lsf_bytes};
pub use writer::{write_lsf, serialize_lsf};
pub use string_table::StringTable;