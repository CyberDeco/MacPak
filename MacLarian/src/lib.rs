#![allow(non_snake_case)]
//! `MacLarian` - Larian Studios file format library for macOS
//!
//! This library provides tools for working with Baldur's Gate 3 and other
//! Larian Studios game file formats, including:
//! - LSF (binary format)
//! - LSX (XML format)
//! - LSJ (JSON format)
//! - LSBC, LSBX, LSBS (legacy binary formats)
//! - Dialog (conversation trees)
//! - Virtual Textures (GTS/GTP)
//! - PAK archives

pub mod error;
pub mod formats;
pub mod pak;
pub mod compression;
pub mod converter;
pub mod utils;
pub mod merged;
pub mod gr2_extraction;
pub mod dyes;
pub mod mods;
pub mod search;

// Top-level domain modules (promoted from formats/)
pub mod dialog;
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

    // Dialog exports (from top-level module)
    pub use crate::dialog::{
        Dialog, DialogNode, NodeConstructor, LocalizationCache,
        parse_dialog, parse_dialog_bytes, parse_dialog_file,
        parse_dialog_lsf, parse_dialog_lsf_bytes,
        load_localization_from_pak_parallel,
    };
    pub use crate::dialog::export::generate_html as generate_dialog_html;

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
        process_extracted_gr2, extract_gr2_with_textures,
    };
    pub use crate::converter::gr2_gltf::{
        convert_gr2_bytes_to_glb_with_textures, TexturedGlbResult,
    };

    // Dyes module exports
    pub use crate::dyes::{
        ColorDef, ColorCategory, COLOR_REGISTRY, DEFAULT_HEX,
        colors_by_category, required_colors, find_color,
        ParsedDyeEntry, DyeLocalizationInfo, ImportedDyeEntry,
        parse_item_combos, parse_object_txt, parse_lsx_dye_presets,
        parse_root_templates_localization, parse_localization_xml,
        generate_color_nodes, generate_all_color_nodes, hex_to_fvec3,
    };

    // Mods module exports
    pub use crate::mods::{
        generate_info_json, InfoJsonResult,
        validate_mod_structure, ModValidationResult,
    };

    // Search module exports
    pub use crate::search::{
        SearchIndex, IndexedFile, FileType,
        ContentCache, CachedContent, ContentCacheStats, ContentMatch,
    };
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");