#![allow(non_snake_case)]
//! MacLarian - Larian Studios file format library for macOS
//! 
//! This library provides tools for working with Baldur's Gate 3 and other
//! Larian Studios game file formats, including:
//! - LSF (binary format)
//! - LSX (XML format)
//! - LSJ (JSON format)
//! - LSBC, LSBX, LSBS (legacy binary formats)

pub mod error;
pub mod formats;
pub mod pak;
pub mod compression;
pub mod converter;
pub mod utils;
pub mod merged;
pub mod gr2_extraction;

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
    pub use crate::formats::virtual_texture::{VirtualTextureExtractor, GtsFile, GtpFile};
    pub use crate::formats::dialog::{
        Dialog, DialogNode, NodeConstructor, LocalizationCache,
        parse_dialog, parse_dialog_bytes, parse_dialog_file,
    };
    pub use crate::pak::PakOperations;
    pub use crate::converter;
    pub use crate::merged::{MergedResolver, MergedDatabase, VisualAsset};
    pub use crate::gr2_extraction::{
        Gr2ExtractionOptions, Gr2ExtractionResult,
        process_extracted_gr2, extract_gr2_with_textures,
    };
    pub use crate::converter::gr2_to_gltf::{
        convert_gr2_bytes_to_glb_with_textures, TexturedGlbResult,
    };
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");