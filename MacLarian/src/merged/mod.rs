//! Merged LSX asset database
//!
//! This module provides tools for extracting and caching GR2-to-texture mappings
//! from BG3's `_merged.lsf` files.
//!
//! # Overview
//!
//! BG3 stores asset metadata in `_merged.lsf` files which contain mappings between:
//! - **VisualBank**: GR2 mesh files with MaterialID references
//! - **MaterialBank**: Materials with Texture2DParameters referencing textures
//! - **TextureBank**: DDS texture file paths
//! - **VirtualTextureBank**: GTex streaming texture references
//!
//! # Usage
//!
//! ```no_run
//! use MacLarian::merged::embedded_database_cached;
//!
//! // Use the embedded production database (recommended)
//! let db = embedded_database_cached();
//! if let Some(asset) = db.get_by_visual_name("HUM_M_ARM_Leather_A_Body") {
//!     println!("GR2: {}", asset.gr2_path);
//!     println!("Textures: {:?}", asset.textures);
//! }
//! ```

mod embedded;
mod parser;
mod paths;
mod resolver;
mod types;

// Re-export types
pub use types::*;

// Re-export resolver
pub use resolver::MergedResolver;

// Re-export embedded database functions
pub use embedded::{embedded_database, embedded_database_cached};

// Re-export path helpers
pub use paths::{
    bg3_data_path, expand_tilde, path_with_tilde, virtual_textures_pak_path, BG3_DATA_PATH_MACOS,
};

// Re-export parser functions (for advanced use)
pub use parser::{merge_databases, resolve_references};
