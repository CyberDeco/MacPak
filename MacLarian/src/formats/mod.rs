//! File format handlers for Larian Studios formats
//!
//! Note: `virtual_texture` has been promoted to a top-level module.
//! It is re-exported here for backwards compatibility.
//!
//! Note: `dialog`, `dyes`, `wem`, and `voice_meta` have been moved to MacPak
//! as they are GUI-specific features.

pub mod common;
pub mod lsf;
pub mod lsx;
pub mod lsj;
pub mod loca;
pub mod gr2;
pub mod meta;

// Re-export common types for convenience
pub use common::{TypeId, get_type_name, type_name_to_id};

// Re-export main document types
pub use lsf::{LsfDocument, LsfNode, LsfAttribute};
pub use lsx::{LsxDocument, LsxRegion, LsxNode, LsxAttribute};
pub use lsj::{LsjDocument, LsjNode, LsjAttribute};
pub use loca::{LocaResource, LocalizedText, read_loca, write_loca};
pub use meta::{ModMetadata, parse_meta_lsx};

// Re-export GR2 decompression utilities
pub use gr2::decompress_gr2;

// Re-export virtual texture types (from top-level module for backwards compatibility)
pub use crate::virtual_texture::{VirtualTextureExtractor, GtsFile, GtpFile};
