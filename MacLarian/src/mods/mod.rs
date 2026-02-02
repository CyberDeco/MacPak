//! Mod utilities - info.json generation and validation for BG3 mods
//!
//! This module provides tools for working with BG3 mod packages:
//! - Generate info.json for `BaldursModManager` compatibility
//! - Generate meta.lsx mod metadata files
//! - Validate mod directory structure
//! - Batch validation of multiple mods
//! - PAK integrity checking
//! - Find and parse mod metadata

pub mod batch_validate;
pub mod info_json;
pub mod meta_generator;
pub mod types;
pub mod validation;

pub use batch_validate::{
    BatchValidationOptions, BatchValidationResult, DryRunResult, ModValidationEntry,
    PakIntegrityResult, check_pak_integrity, check_pak_integrity_with_progress,
    validate_directory_recursive, validate_directory_recursive_with_progress,
    validate_for_pak_creation, validate_for_pak_creation_with_progress, validate_mods_batch,
    validate_mods_batch_with_progress,
};
pub use info_json::{
    InfoJsonResult, generate_info_json, generate_info_json_from_source,
    generate_info_json_with_progress,
};
pub use meta_generator::{
    generate_meta_lsx, parse_version_string, to_folder_name, version_to_int64,
};
pub use types::{ModPhase, ModProgress, ModProgressCallback};
pub use validation::{
    ModValidationResult, validate_mod_structure, validate_mod_structure_with_progress,
    validate_pak_mod_structure, validate_pak_mod_structure_with_progress,
};
