#![allow(non_snake_case)]
//! # MacLarian
//!
//! A pure-Rust library for working with Baldur's Gate 3 and Larian Studios file formats.
//!
//! ## Supported Formats
//!
//! - **PAK archives** - Extract, create, and list game asset packages
//! - **LSF/LSX/LSJ** - Binary, XML, and JSON document formats
//! - **GR2** - Granny2 mesh files with BitKnit decompression
//! - **Virtual Textures** - GTS/GTP streaming texture extraction
//! - **LOCA** - Localization files
//! - **DDS/PNG** - Texture conversion
//!
//! ## Quick Start
//!
//! ### Working with PAK Archives
//!
//! ```no_run
//! use maclarian::pak::PakOperations;
//!
//! // List contents of a PAK file
//! let files = PakOperations::list("Shared.pak")?;
//! println!("Found {} files", files.len());
//!
//! // Extract a PAK file
//! PakOperations::extract("Shared.pak", "output/")?;
//!
//! // Read a specific file without extracting
//! let data = PakOperations::read_file_bytes("Shared.pak", "Public/Shared/meta.lsx")?;
//! # Ok::<(), maclarian::Error>(())
//! ```
//!
//! ### Converting Document Formats
//!
//! ```no_run
//! use maclarian::converter::convert_lsf_to_lsx;
//!
//! // Convert LSF (binary) to LSX (XML) file
//! convert_lsf_to_lsx("meta.lsf", "meta.lsx")?;
//! # Ok::<(), maclarian::Error>(())
//! ```
//!
//! ### Using the Prelude
//!
//! The prelude provides convenient access to commonly used types:
//!
//! ```
//! use maclarian::prelude::*;
//!
//! // Now you have access to:
//! // - PakOperations, SearchIndex, FileType
//! // - LsfDocument, LsxDocument, LsjDocument
//! // - VirtualTextureExtractor, GtsFile, GtpFile
//! // - Error, Result, and more
//! ```
//!
//! ## Feature Flags
//!
//! - `cli` - Enables the `maclarian` command-line binary

pub mod error;
pub mod formats;
pub mod pak;
pub mod compression;
pub mod converter;
pub mod utils;
pub mod merged;
pub mod gr2_extraction;
pub mod mods;
pub mod search;

// Top-level domain modules (promoted from formats/)
pub mod virtual_texture;

// Re-exports for convenience
pub use error::{Error, Result};

/// Prelude module for common imports
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::formats::common::{TypeId, get_type_name, type_name_to_id};
    pub use crate::formats::lsf::{LsfDocument, LsfNode, LsfAttribute};
    pub use crate::formats::lsx::{LsxDocument, LsxRegion, LsxNode, LsxAttribute};
    pub use crate::formats::lsj::{LsjDocument, LsjNode, LsjAttribute};
    pub use crate::formats::gr2::decompress_gr2;

    // Virtual texture exports (from top-level module)
    pub use crate::virtual_texture::{
        VirtualTextureExtractor, GtsFile, GtpFile,
        extract_gts_file, extract_batch as extract_vt_batch,
        GtsExtractResult, BatchExtractResult as VtBatchResult,
    };

    // PAK operations
    pub use crate::pak::{
        PakOperations,
        find_pak_files, find_packable_folders, batch_extract, batch_create, BatchPakResult,
    };

    pub use crate::converter;
    pub use crate::merged::{MergedResolver, MergedDatabase, VisualAsset};
    pub use crate::gr2_extraction::{
        Gr2ExtractionOptions, Gr2ExtractionResult,
        process_extracted_gr2, process_extracted_gr2_to_dir, extract_gr2_with_textures,
    };
    pub use crate::converter::gr2_gltf::{
        convert_gr2_bytes_to_glb_with_textures, TexturedGlbResult,
    };

    // Mods module exports
    pub use crate::mods::{
        generate_info_json, InfoJsonResult,
        validate_mod_structure, ModValidationResult,
    };

    // Search module exports (public types only)
    pub use crate::search::{
        SearchIndex, IndexedFile, FileType, FullTextResult,
    };
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// CLI module (feature-gated)
#[cfg(feature = "cli")]
pub mod cli;