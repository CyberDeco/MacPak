//! Virtual texture utility functions
//!
//! Helper functions for working with GTS/GTP files.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

use std::path::Path;
use crate::error::{Error, Result};
use super::{GtsFile, GtpFile};

/// Information about a GTS file
#[derive(Debug, Clone)]
pub struct GtsInfo {
    pub version: u32,
    pub guid: [u8; 16],
    pub tile_width: i32,
    pub tile_height: i32,
    pub tile_border: i32,
    pub num_layers: u32,
    pub num_levels: u32,
    pub page_files: Vec<PageFileInfo>,
}

/// Information about a page file
#[derive(Debug, Clone)]
pub struct PageFileInfo {
    pub filename: String,
    pub num_pages: u32,
}

/// Information about a GTP file
#[derive(Debug, Clone)]
pub struct GtpInfo {
    pub version: u32,
    pub guid: [u8; 16],
    pub num_pages: usize,
    pub chunks_per_page: Vec<usize>,
}

/// Result of extracting virtual textures
#[derive(Debug, Clone)]
pub struct ExtractResult {
    pub extracted: usize,
    pub failed: usize,
    pub total: usize,
    pub errors: Vec<String>,
}

/// List information about a GTS file
///
/// # Errors
/// Returns an error if the GTS file cannot be read or parsed.
pub fn list_gts<P: AsRef<Path>>(gts_path: P) -> Result<GtsInfo> {
    let gts = GtsFile::open(gts_path.as_ref())?;

    Ok(GtsInfo {
        version: gts.header.version,
        guid: gts.header.guid,
        tile_width: gts.header.tile_width,
        tile_height: gts.header.tile_height,
        tile_border: gts.header.tile_border,
        num_layers: gts.header.num_layers,
        num_levels: gts.header.num_levels,
        page_files: gts.page_files.iter().map(|pf| PageFileInfo {
            filename: pf.filename.clone(),
            num_pages: pf.num_pages,
        }).collect(),
    })
}

/// Get information about a GTP file
///
/// # Errors
/// Returns an error if the GTP or GTS file cannot be read.
pub fn gtp_info<P1: AsRef<Path>, P2: AsRef<Path>>(gtp_path: P1, gts_path: P2) -> Result<GtpInfo> {
    let gts = GtsFile::open(gts_path.as_ref())?;
    let gtp = GtpFile::open(gtp_path.as_ref(), &gts)?;

    Ok(GtpInfo {
        version: gtp.header.version,
        guid: gtp.header.guid,
        num_pages: gtp.num_pages(),
        chunks_per_page: (0..gtp.num_pages())
            .map(|p| gtp.num_chunks(p))
            .collect(),
    })
}

/// Extract subfolder name from GTP filename
///
/// Strips the hash suffix from GTP filenames:
/// "`Albedo_Normal_Physical_0_abc123...def.gtp`" -> "`Albedo_Normal_Physical_0`"
#[must_use] 
pub fn get_subfolder_name(filename: &str) -> String {
    let stem = filename.strip_suffix(".gtp")
        .or_else(|| filename.strip_suffix(".GTP"))
        .unwrap_or(filename);

    // Strip hash suffix (32 hex chars after last underscore)
    if let Some(last_underscore) = stem.rfind('_') {
        let suffix = &stem[last_underscore + 1..];
        if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            return stem[..last_underscore].to_string();
        }
    }

    stem.to_string()
}

/// Extract the base name from a virtual texture filename
///
/// e.g., "`Albedo_Normal_Physical_1`" -> `Some("Albedo_Normal_Physical`")
#[must_use] 
pub fn find_base_name(name: &str) -> Option<&str> {
    // Check if name ends with _N where N is a digit
    if let Some(last_underscore) = name.rfind('_') {
        let suffix = &name[last_underscore + 1..];
        if suffix.chars().all(|c| c.is_ascii_digit()) {
            return Some(&name[..last_underscore]);
        }
    }
    None
}

/// Find the GTS file for a given path (handles both .gts and .gtp files)
///
/// This function resolves GTP files to their associated GTS metadata files,
/// handling various edge cases like NULL-padded GTS files and hash suffixes.
///
/// # Errors
/// Returns an error if the GTS file cannot be found or the input has an unsupported file type.
pub fn find_gts_path(input_path: &str) -> Result<String> {
    let path = Path::new(input_path);
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let parent = path.parent().unwrap_or(Path::new("."));

    if ext == "gts" {
        // Check if this GTS file has valid GRPG header
        if let Ok(data) = std::fs::read(input_path) {
            if data.len() >= 4 && &data[0..4] == b"GRPG" {
                return Ok(input_path.to_string());
            }
            // NULL-padded GTS - try to find the _0.gts version
            let stem = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(base) = find_base_name(&stem) {
                let gts_0_path = parent.join(format!("{base}_0.gts"));
                if gts_0_path.exists() {
                    return Ok(gts_0_path.to_string_lossy().to_string());
                }
            }
        }
        return Ok(input_path.to_string());
    }

    if ext == "gtp" {
        // Find associated GTS file
        let stem = path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // GTP files have pattern: BaseName_N_<hash>.gtp where N is index, hash is 32 hex chars
        // Each tile set index has its own GTS file: BaseName_N.gts

        // Strip the hash suffix first
        let name_without_hash = if let Some(last_underscore) = stem.rfind('_') {
            let suffix = &stem[last_underscore + 1..];
            if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
                &stem[..last_underscore]
            } else {
                stem.as_str()
            }
        } else {
            stem.as_str()
        };

        // First try the exact matching GTS file
        let gts_path = parent.join(format!("{name_without_hash}.gts"));
        if gts_path.exists() {
            // Check if it has a valid GRPG header
            if let Ok(data) = std::fs::read(&gts_path)
                && data.len() >= 4 && &data[0..4] == b"GRPG" {
                    return Ok(gts_path.to_string_lossy().to_string());
                }
        }

        // Try _0.gts as fallback
        if let Some(base) = find_base_name(name_without_hash) {
            let gts_0_path = parent.join(format!("{base}_0.gts"));
            if gts_0_path.exists() {
                return Ok(gts_0_path.to_string_lossy().to_string());
            }
        }

        // Look for any valid GTS file in the same directory that shares the base prefix
        if let Ok(entries) = std::fs::read_dir(parent) {
            let gtp_prefix = stem.split('_').take(3).collect::<Vec<_>>().join("_");
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.extension().map(|e| e.to_string_lossy().to_lowercase()) == Some("gts".to_string())
                    && let Some(gts_stem) = entry_path.file_stem() {
                        let gts_name = gts_stem.to_string_lossy();
                        if gts_name.starts_with(&gtp_prefix) {
                            // Check for valid GRPG header
                            if let Ok(data) = std::fs::read(&entry_path)
                                && data.len() >= 4 && &data[0..4] == b"GRPG" {
                                    return Ok(entry_path.to_string_lossy().to_string());
                                }
                        }
                    }
            }
        }

        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Could not find associated GTS file for {input_path}")
        )));
    }

    Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("Unsupported file type: {ext}")
    )))
}

/// Extract all GTP files referenced by a GTS file
///
/// # Errors
/// Returns an error if the GTS file cannot be read or the output directory cannot be created.
pub fn extract_all<P1: AsRef<Path>, P2: AsRef<Path>>(
    gts_path: P1,
    output_dir: P2,
) -> Result<ExtractResult> {
    use super::VirtualTextureExtractor;

    let gts = GtsFile::open(gts_path.as_ref())?;
    let gts_dir = gts_path.as_ref().parent().unwrap_or(Path::new("."));
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir)?;

    let mut extracted = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for page_file in &gts.page_files {
        let gtp_path = gts_dir.join(&page_file.filename);

        if !gtp_path.exists() {
            failed += 1;
            errors.push(format!("File not found: {}", page_file.filename));
            continue;
        }

        // Create subfolder for this GTP
        let subfolder = get_subfolder_name(&page_file.filename);
        let gtp_output = output_dir.join(&subfolder);

        match VirtualTextureExtractor::extract_with_gts(&gtp_path, gts_path.as_ref(), &gtp_output) {
            Ok(()) => extracted += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", page_file.filename, e));
            }
        }
    }

    Ok(ExtractResult {
        extracted,
        failed,
        total: gts.page_files.len(),
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_subfolder_name() {
        assert_eq!(
            get_subfolder_name("Albedo_Normal_Physical_0_0a0d1854395eb40436bec69fe14aa92b.gtp"),
            "Albedo_Normal_Physical_0"
        );
        assert_eq!(
            get_subfolder_name("simple.gtp"),
            "simple"
        );
    }

    #[test]
    fn test_find_base_name() {
        assert_eq!(find_base_name("Albedo_Normal_Physical_1"), Some("Albedo_Normal_Physical"));
        assert_eq!(find_base_name("Albedo_Normal_Physical_0"), Some("Albedo_Normal_Physical"));
        assert_eq!(find_base_name("no_number_suffix"), None);
    }
}
