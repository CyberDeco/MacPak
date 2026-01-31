//! Virtual texture operations
//!
//! High-level API for working with BG3 virtual textures (GTS/GTP files).
//! Core functionality is in maclarian; this module provides re-exports and
//! any `MacPak`-specific wrappers.

use crate::{Error, Result};
use std::path::Path;

// Re-export types from maclarian for convenience
pub use maclarian::virtual_texture::{
    ExtractResult,
    GtpFile,
    GtpInfo,
    GtsFile,
    GtsInfo,
    PageFileInfo,
    VirtualTextureExtractor,
    extract_all as maclarian_extract_all,
    find_base_name,
    find_gts_path,
    get_subfolder_name,
    gtp_info as maclarian_gtp_info,
    // Utility functions and types
    list_gts as maclarian_list_gts,
};

/// List information about a GTS file.
///
/// # Errors
///
/// Returns an error if the GTS file cannot be read or parsed.
pub fn list_gts<P: AsRef<Path>>(gts_path: P) -> Result<GtsInfo> {
    maclarian_list_gts(gts_path).map_err(Error::MacLarian)
}

/// Get information about a GTP file.
///
/// # Errors
///
/// Returns an error if the GTP or GTS file cannot be read or parsed.
pub fn gtp_info<P1: AsRef<Path>, P2: AsRef<Path>>(gtp_path: P1, gts_path: P2) -> Result<GtpInfo> {
    maclarian_gtp_info(gtp_path, gts_path).map_err(Error::MacLarian)
}

/// Extract a single GTP file to DDS textures.
///
/// # Errors
///
/// Returns an error if extraction fails.
pub fn extract_gtp<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(
    gtp_path: P1,
    gts_path: P2,
    output_dir: P3,
) -> Result<()> {
    VirtualTextureExtractor::extract_with_gts(gtp_path, gts_path, output_dir)
        .map_err(Error::MacLarian)
}

/// Extract all GTP files referenced by a GTS file.
///
/// # Errors
///
/// Returns an error if extraction fails.
pub fn extract_all<P1: AsRef<Path>, P2: AsRef<Path>>(
    gts_path: P1,
    output_dir: P2,
) -> Result<ExtractResult> {
    maclarian_extract_all(gts_path, output_dir).map_err(Error::MacLarian)
}
