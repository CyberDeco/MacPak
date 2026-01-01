//! Virtual texture CLI commands

use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use crate::operations::virtual_texture;

/// List textures in a GTS file
pub fn list(gts_path: &Path) -> Result<()> {
    let info = virtual_texture::list_gts(gts_path)
        .with_context(|| format!("Failed to parse GTS file: {}", gts_path.display()))?;

    println!("Virtual Texture Set: {}", gts_path.display());
    println!("GUID: {:02x?}", info.guid);
    println!("Version: {}", info.version);
    println!("Tile size: {}x{} (border: {})",
        info.tile_width, info.tile_height, info.tile_border);
    println!("Layers: {}", info.num_layers);
    println!("Levels: {}", info.num_levels);
    println!("Page files: {}", info.page_files.len());
    println!();

    println!("Page files:");
    for (i, pf) in info.page_files.iter().enumerate() {
        println!("  [{}] {} ({} pages)", i, pf.filename, pf.num_pages);
    }

    Ok(())
}

/// Extract textures from GTS/GTP files
pub fn extract(
    gts_path: &Path,
    gtp_path: Option<&Path>,
    output_dir: &Path,
    _texture_name: Option<&str>,
    _layer: Option<usize>,
    _all_layers: bool,
) -> Result<()> {
    // Create output directory
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    // If a specific GTP path is provided, use it directly
    if let Some(gtp) = gtp_path {
        println!("Extracting {} with GTS {}...", gtp.display(), gts_path.display());

        virtual_texture::extract_gtp(gtp, gts_path, output_dir)
            .with_context(|| "Failed to extract virtual texture")?;

        println!("Extraction complete -> {}", output_dir.display());
        return Ok(());
    }

    // Otherwise, extract all GTPs referenced by this GTS
    let info = virtual_texture::list_gts(gts_path)
        .with_context(|| format!("Failed to parse GTS file: {}", gts_path.display()))?;

    println!("Layers: {} | Page files: {}", info.num_layers, info.page_files.len());

    let result = virtual_texture::extract_all(gts_path, output_dir)
        .with_context(|| "Failed to extract virtual textures")?;

    println!("Extracted {}/{} GTP files to {}", result.extracted, result.total, output_dir.display());

    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("Warning: {}", err);
        }
    }

    Ok(())
}

/// Info about a GTP file
pub fn gtp_info(gtp_path: &Path, gts_path: Option<&Path>) -> Result<()> {
    // We need a GTS file to properly parse the GTP
    let gts_path = match gts_path {
        Some(p) => p.to_path_buf(),
        None => {
            // Try to find GTS in same directory
            let gtp_dir = gtp_path.parent().unwrap_or(Path::new("."));
            let mut found = None;
            if let Ok(entries) = fs::read_dir(gtp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("gts") {
                        found = Some(path);
                        break;
                    }
                }
            }
            found.ok_or_else(|| anyhow::anyhow!("No GTS file found. Provide one with --gts"))?
        }
    };

    let info = virtual_texture::gtp_info(gtp_path, &gts_path)
        .with_context(|| format!("Failed to parse GTP file: {}", gtp_path.display()))?;

    println!("GTP File: {}", gtp_path.display());
    println!("GUID: {:02x?}", info.guid);
    println!("Version: {}", info.version);
    println!("Pages: {}", info.num_pages);

    for (page, chunks) in info.chunks_per_page.iter().enumerate() {
        println!("  Page {}: {} chunks", page, chunks);
    }

    Ok(())
}
