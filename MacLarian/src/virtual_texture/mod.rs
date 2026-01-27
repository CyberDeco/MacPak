//! Virtual texture format handlers (GTS/GTP)
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! This module provides support for BG3's virtual texture system:
//! - GTS (Game Texture Set) - metadata files describing texture layouts
//! - GTP (Game Texture Page) - tile data files with compressed textures
//!
//! # Extraction Example
//!
//! ```no_run
//! use maclarian::virtual_texture::VirtualTextureExtractor;
//!
//! // Extract a GTP file to DDS textures
//! VirtualTextureExtractor::extract(
//!     "path/to/texture.gtp",
//!     "output/directory",
//! ).unwrap();
//! ```
//!
//! # Creation Example
//!
//! ```no_run
//! use maclarian::virtual_texture::builder::{VirtualTextureBuilder, SourceTexture};
//!
//! // Create a virtual texture from source DDS files
//! let result = VirtualTextureBuilder::new()
//!     .add_texture(
//!         SourceTexture::new("MyTexture")
//!             .with_base_map("base.dds")
//!             .with_normal_map("normal.dds")
//!     )
//!     .build("output/")?;
//! # Ok::<(), maclarian::error::Error>(())
//! ```

mod types;
mod gts;
mod gtp;
mod extractor;
mod utils;
mod batch;
mod mod_config;
pub mod builder;
pub mod writer;

// Re-exports - public types
pub use types::*;
pub use gts::GtsFile;
pub use gtp::GtpFile;

// Public extractor
pub use extractor::{VirtualTextureExtractor, ExtractOptions};

// Utility functions
pub use utils::{
    list_gts, gtp_info, get_subfolder_name, find_base_name, find_gts_path, extract_all,
    GtsInfo, PageFileInfo, GtpInfo, ExtractResult,
};

// Batch operations
pub use batch::{
    extract_gts_file, extract_batch, GtsExtractResult, BatchExtractResult,
};
