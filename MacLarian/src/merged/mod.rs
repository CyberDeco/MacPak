//! Merged LSX asset database
//!
//! This module provides tools for extracting and caching GR2-to-texture mappings
//! from BG3's `_merged.lsf` files.
//!
//! # Overview
//!
//! BG3 stores asset metadata in `_merged.lsf` files which contain mappings between:
//! - **`VisualBank`**: GR2 mesh files with `MaterialID` references
//! - **`MaterialBank`**: Materials with `Texture2DParameters` referencing textures
//! - **`TextureBank`**: DDS texture file paths
//! - **`VirtualTextureBank`**: `GTex` streaming texture references
//!
//! # Usage
//!
//! ```no_run
//! use maclarian::merged::GameDataResolver;
//!
//! // Auto-detect game installation and query the database
//! let resolver = GameDataResolver::auto_detect()?;
//! if let Some(asset) = resolver.get_by_visual_name("HUM_M_ARM_Leather_A_Body") {
//!     println!("GR2: {}", asset.gr2_path);
//!     println!("Textures: {:?}", asset.textures);
//! }
//! # Ok::<(), maclarian::error::Error>(())
//! ```

mod game_data;
mod parser;
mod paths;
mod resolver;
mod types;

// Re-export types
pub use types::*;

// Re-export resolver
pub use resolver::MergedResolver;

// Re-export game data resolver (primary API)
pub use game_data::{GameDataResolver, BG3_DATA_PATH_WINDOWS};

// Re-export path helpers
pub use paths::{
    bg3_data_path, expand_tilde, path_with_tilde, virtual_textures_pak_path, BG3_DATA_PATH_MACOS,
};

