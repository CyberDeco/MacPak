//! File format handlers for Larian Studios formats
//!
//! Note: `virtual_texture` has been promoted to a top-level module.
//! It is re-exported here for backwards compatibility.

pub mod common;
pub mod gr2;
pub mod loca;
pub mod lsf;
pub mod lsj;
pub mod lsx;
pub mod meta;

// Re-export common types for convenience
pub use common::{TypeId, get_type_name, type_name_to_id};

// Re-export main document types
pub use loca::{LocaResource, LocalizedText, read_loca, write_loca};
pub use lsf::{LsfAttribute, LsfDocument, LsfNode};
pub use lsj::{LsjAttribute, LsjDocument, LsjNode};
pub use lsx::{LsxAttribute, LsxDocument, LsxNode, LsxRegion};
pub use meta::{ModMetadata, parse_meta_lsx};

// Re-export GR2 decompression utilities
pub use gr2::decompress_gr2;

// Re-export virtual texture types (from top-level module for backwards compatibility)
pub use crate::virtual_texture::{GtpFile, GtsFile, VirtualTextureExtractor};
