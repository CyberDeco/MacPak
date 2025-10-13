//! Utility functions

pub mod string_table;
pub mod hash;
pub mod path;

pub use string_table::StringTable;
pub use hash::{hash_string, hash_string_djb2};
pub use path::normalize_path;
