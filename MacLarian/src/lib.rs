//! MacLarian - Native Rust implementation of LSLib functionality
//! 
//! This crate provides low-level access to Larian Studios file formats
//! used in Divinity: Original Sin 2 and Baldur's Gate 3.

pub mod error;
pub mod formats;
pub mod pak;
pub mod compression;
pub mod converter;
pub mod utils;

// Re-exports for convenience
pub use error::{Error, Result};

/// Prelude module for common imports
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::formats::common::{AttributeValue, TranslatedString};
    pub use crate::formats::lsf::LsfDocument;
    pub use crate::formats::lsx::LsxDocument;
    pub use crate::pak::PakOperations;
    pub use crate::converter;
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");