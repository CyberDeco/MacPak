//! File format handlers for Larian Studios formats

pub mod common;
pub mod lsf;
pub mod lsx;
pub mod lsj;
pub mod gr2;

// Re-export common types for convenience
pub use common::{TypeId, get_type_name, type_name_to_id};

// Re-export main document types
pub use lsf::{LsfDocument, LsfNode, LsfAttribute};
pub use lsx::{LsxDocument, LsxRegion, LsxNode, LsxAttribute};
pub use lsj::{LsjDocument, LsjNode, LsjAttribute};

// Re-export gr2 handling
pub use gr2::{decompress, GR2File, GR2Section, GR2Decompressor};