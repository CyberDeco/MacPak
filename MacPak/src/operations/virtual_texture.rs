//! Virtual texture operations
//!
//! High-level API for working with BG3 virtual textures (GTS/GTP files).
//! Core functionality is in maclarian; this module provides re-exports and
//! any MacPak-specific wrappers.

use std::path::Path;
use crate::{Error, Result};

// Re-export types from maclarian for convenience
pub use maclarian::virtual_texture::{
    GtsFile, GtpFile, GtsHeader, GtsPageFileInfo, GtpHeader,
    VirtualTextureExtractor,
    // Utility functions and types
    list_gts as maclarian_list_gts, gtp_info as maclarian_gtp_info,
    find_gts_path, find_base_name, get_subfolder_name, extract_all as maclarian_extract_all,
    GtsInfo, PageFileInfo, GtpInfo, ExtractResult,
};

/// List information about a GTS file
pub fn list_gts<P: AsRef<Path>>(gts_path: P) -> Result<GtsInfo> {
    maclarian_list_gts(gts_path).map_err(|e| Error::MacLarian(e))
}

/// Get information about a GTP file
pub fn gtp_info<P1: AsRef<Path>, P2: AsRef<Path>>(gtp_path: P1, gts_path: P2) -> Result<GtpInfo> {
    maclarian_gtp_info(gtp_path, gts_path).map_err(|e| Error::MacLarian(e))
}

/// Extract a single GTP file to DDS textures
pub fn extract_gtp<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(
    gtp_path: P1,
    gts_path: P2,
    output_dir: P3,
) -> Result<()> {
    VirtualTextureExtractor::extract_with_gts(gtp_path, gts_path, output_dir)
        .map_err(|e| Error::MacLarian(e))
}

/// Extract all GTP files referenced by a GTS file
pub fn extract_all<P1: AsRef<Path>, P2: AsRef<Path>>(
    gts_path: P1,
    output_dir: P2,
) -> Result<ExtractResult> {
    maclarian_extract_all(gts_path, output_dir).map_err(|e| Error::MacLarian(e))
}
