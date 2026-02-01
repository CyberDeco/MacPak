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

mod batch;
pub mod builder;
mod extractor;
mod gtp;
mod gts;
pub mod mod_config;
mod types;
mod utils;
pub(crate) mod writer;

// Re-exports - public types
pub use gtp::GtpFile;
pub use gts::GtsFile;

// Re-export only public types from types module (not internal format structs)
pub use types::{
    GtsCodec, TileCompression, VTexPhase, VTexProgress, VTexProgressCallback, VirtualTextureLayer,
    VirtualTextureOutput,
};

// Public extractor
pub use extractor::{ExtractOptions, VirtualTextureExtractor};

// Utility functions
pub use utils::{
    ExtractResult, GtpInfo, GtsInfo, PageFileInfo, extract_all, find_base_name, find_gts_path,
    get_subfolder_name, gtp_info, list_gts,
};

// Batch operations
pub use batch::{BatchExtractResult, GtsExtractResult, extract_batch, extract_gts_file};

// Mod config discovery and lookup
pub use mod_config::{
    DiscoveredVirtualTexture, DiscoverySource, discover_mod_virtual_textures,
    discover_virtual_textures, extract_by_gtex, find_gts_for_gtex, find_virtual_texture,
};
