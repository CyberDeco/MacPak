//! Mod utilities - info.json generation and validation for BG3 mods
//!
//! This module provides tools for working with BG3 mod packages:
//! - Generate info.json for `BaldursModManager` compatibility
//! - Validate mod directory structure
//! - Find and parse mod metadata

pub mod info_json;
pub mod validation;

pub use info_json::{generate_info_json, InfoJsonResult};
pub use validation::{validate_mod_structure, validate_pak_mod_structure, ModValidationResult};
