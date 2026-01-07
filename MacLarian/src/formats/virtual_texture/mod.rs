//! Virtual texture format handlers (GTS/GTP)
//!
//! This module provides support for BG3's virtual texture system:
//! - GTS (Game Texture Set) - metadata files describing texture layouts
//! - GTP (Game Texture Page) - tile data files with compressed textures
//!
//! # Usage
//!
//! ```no_run
//! use maclarian::formats::virtual_textures::VirtualTextureExtractor;
//!
//! // Extract a GTP file to DDS textures
//! VirtualTextureExtractor::extract(
//!     "path/to/texture.gtp",
//!     "output/directory",
//!     None,
//! ).unwrap();
//! ```

pub mod types;
pub mod gts;
pub mod gtp;
pub mod extractor;
pub mod utils;

// Re-exports
pub use types::*;
pub use gts::GtsFile;
pub use gtp::GtpFile;
pub use extractor::{VirtualTextureExtractor, DdsWriter};
pub use utils::{
    list_gts, gtp_info, get_subfolder_name, find_base_name, find_gts_path, extract_all,
    GtsInfo, PageFileInfo, GtpInfo, ExtractResult,
};
