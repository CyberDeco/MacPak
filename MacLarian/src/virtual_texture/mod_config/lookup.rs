//! Lookup and extraction functions for virtual textures
//!
//!

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::virtual_texture::utils::{extract_all, ExtractResult};

use super::discovery::discover_virtual_textures;
use super::types::DiscoveredVirtualTexture;

// ============================================================================
// Lookup functions
// ============================================================================

/// Find GTS path for a `GTex` hash across search paths
///
/// Searches `VTexConfig.xml` first (primary), then falls back to `VirtualTextures.json`.
/// Returns the first match found.
///
/// # Arguments
/// * `gtex_hash` - The `GTex` hash to look up
/// * `search_paths` - Paths to search (mod roots or directories containing mods)
///
/// # Returns
/// The resolved GTS file path if found, or `None` if not found
pub fn find_gts_for_gtex(gtex_hash: &str, search_paths: &[PathBuf]) -> Result<Option<PathBuf>> {
    let discovered = discover_virtual_textures(search_paths)?;

    for vt in discovered {
        if vt.gtex_hash == gtex_hash {
            return Ok(Some(vt.gts_path));
        }
    }

    Ok(None)
}

/// Find a discovered virtual texture by `GTex` hash
///
/// Like `find_gts_for_gtex` but returns full discovery information.
pub fn find_virtual_texture(
    gtex_hash: &str,
    search_paths: &[PathBuf],
) -> Result<Option<DiscoveredVirtualTexture>> {
    let discovered = discover_virtual_textures(search_paths)?;

    for vt in discovered {
        if vt.gtex_hash == gtex_hash {
            return Ok(Some(vt));
        }
    }

    Ok(None)
}

// ============================================================================
// High-level extraction
// ============================================================================

/// Extract a virtual texture by `GTex` hash
///
/// Discovers the GTS location from mod configs and extracts to the output directory.
///
/// # Arguments
/// * `gtex_hash` - The `GTex` hash to extract
/// * `search_paths` - Paths to search for the mod containing this texture
/// * `output_dir` - Directory to write extracted DDS files
///
/// # Errors
/// Returns an error if the `GTex` hash is not found or extraction fails.
pub fn extract_by_gtex(
    gtex_hash: &str,
    search_paths: &[PathBuf],
    output_dir: &Path,
) -> Result<ExtractResult> {
    let vt = find_virtual_texture(gtex_hash, search_paths)?.ok_or_else(|| {
        Error::ConversionError(format!("GTex hash '{gtex_hash}' not found in search paths"))
    })?;

    if !vt.gts_path.exists() {
        return Err(Error::ConversionError(format!(
            "GTS file not found at derived path: {}",
            vt.gts_path.display()
        )));
    }

    // Extract all GTPs referenced by this GTS
    extract_all(&vt.gts_path, output_dir)
}
