//! Virtual texture operations
//!
//! High-level API for working with BG3 virtual textures (GTS/GTP files).

use std::path::Path;
use crate::{Error, Result};

// Re-export types for convenience
pub use MacLarian::formats::virtual_texture::{
    GtsFile, GtpFile, GtsHeader, GtsPageFileInfo, GtpHeader,
    VirtualTextureExtractor, DdsWriter,
};

/// List information about a GTS file
pub fn list_gts<P: AsRef<Path>>(gts_path: P) -> Result<GtsInfo> {
    let gts = GtsFile::open(gts_path.as_ref())
        .map_err(|e| Error::MacLarian(e))?;

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
pub fn gtp_info<P1: AsRef<Path>, P2: AsRef<Path>>(gtp_path: P1, gts_path: P2) -> Result<GtpInfo> {
    let gts = GtsFile::open(gts_path.as_ref())
        .map_err(|e| Error::MacLarian(e))?;

    let gtp = GtpFile::open(gtp_path.as_ref(), &gts)
        .map_err(|e| Error::MacLarian(e))?;

    Ok(GtpInfo {
        version: gtp.header.version,
        guid: gtp.header.guid,
        num_pages: gtp.num_pages(),
        chunks_per_page: (0..gtp.num_pages())
            .map(|p| gtp.num_chunks(p))
            .collect(),
    })
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
    let gts = GtsFile::open(gts_path.as_ref())
        .map_err(|e| Error::MacLarian(e))?;

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

/// Extract subfolder name from GTP filename
/// "Albedo_Normal_Physical_0_abc123...def.gtp" -> "Albedo_Normal_Physical_0"
fn get_subfolder_name(filename: &str) -> String {
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
