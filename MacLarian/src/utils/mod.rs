//! Utility functions

pub mod path;

// Re-export moved modules for backwards compatibility
pub use crate::formats::lsf::StringTable;
pub use crate::formats::common::hash::{hash_string, hash_string_djb2, hash_string_lslib};
pub use crate::formats::common as lsf_types;

pub use path::normalize_path;