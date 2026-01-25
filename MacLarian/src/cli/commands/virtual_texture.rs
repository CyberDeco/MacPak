//! Virtual texture CLI commands

use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use crate::virtual_texture::{self, ExtractOptions};

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
    gtp_dir: Option<&Path>,
    output_dir: &Path,
    _texture_name: Option<&str>,
    layers: Vec<usize>,
    all_layers: bool,
) -> Result<()> {
    // Create output directory
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    // Build extraction options
    let options = ExtractOptions {
        layers: layers.clone(),
        all_layers,
    };

    // If a specific GTP directory is provided, extract just GTPs from there
    if let Some(gtp_path) = gtp_dir {
        println!("Extracting from {} with GTS {}...", gtp_path.display(), gts_path.display());

        virtual_texture::VirtualTextureExtractor::extract_with_options(
            gtp_path,
            gts_path,
            output_dir,
            &options,
        ).with_context(|| "Failed to extract virtual texture")?;

        println!("Extraction complete -> {}", output_dir.display());
        return Ok(());
    }

    // Otherwise, extract all GTPs referenced by this GTS
    let info = virtual_texture::list_gts(gts_path)
        .with_context(|| format!("Failed to parse GTS file: {}", gts_path.display()))?;

    println!("Layers: {} | Page files: {}", info.num_layers, info.page_files.len());

    if !layers.is_empty() {
        println!("Layer filter: {:?}", layers);
    }

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

/// Batch extract multiple GTS files in parallel
pub fn batch(
    input_dir: &Path,
    output_dir: &Path,
    layers: Vec<usize>,
    recursive: bool,
) -> Result<()> {
    // Find all GTS files
    let mut gts_files = Vec::new();
    find_gts_files(input_dir, &mut gts_files, recursive)?;

    if gts_files.is_empty() {
        println!("No GTS files found in {}", input_dir.display());
        return Ok(());
    }

    println!("Found {} GTS files", gts_files.len());

    // Create output directory
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    if !layers.is_empty() {
        println!("Layer filter: {:?}", layers);
    }

    // Use batch extraction
    let result = virtual_texture::extract_batch(
        &gts_files,
        Some(output_dir),
        |current, total, desc| {
            println!("[{}/{}] {}", current, total, desc);
        },
    );

    println!();
    println!("Batch extraction complete:");
    println!("  Succeeded: {}", result.success_count);
    println!("  Failed: {}", result.error_count);
    println!("  Total textures: {}", result.texture_count);

    if result.error_count > 0 {
        println!();
        println!("Errors:");
        for msg in &result.results {
            if msg.starts_with("Failed") {
                println!("  {}", msg);
            }
        }
    }

    Ok(())
}

/// Recursively find all GTS files in a directory
fn find_gts_files(dir: &Path, files: &mut Vec<PathBuf>, recursive: bool) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && recursive {
            find_gts_files(&path, files, recursive)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "gts" {
                    files.push(path);
                }
            }
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
