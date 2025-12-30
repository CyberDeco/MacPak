//! Virtual texture CLI commands

use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use MacPak::MacLarian::formats::virtual_texture::{VirtualTextureExtractor, GtsFile, GtpFile};

/// List textures in a GTS file
pub fn list(gts_path: &Path) -> Result<()> {
    let gts = GtsFile::open(gts_path)
        .with_context(|| format!("Failed to parse GTS file: {}", gts_path.display()))?;

    println!("Virtual Texture Set: {}", gts_path.display());
    println!("GUID: {:02x?}", gts.header.guid);
    println!("Version: {}", gts.header.version);
    println!("Tile size: {}x{} (border: {})",
        gts.header.tile_width, gts.header.tile_height, gts.header.tile_border);
    println!("Layers: {}", gts.header.num_layers);
    println!("Levels: {}", gts.header.num_levels);
    println!("Page files: {}", gts.header.num_page_files);
    println!();

    println!("Page files:");
    for (i, pf) in gts.page_files.iter().enumerate() {
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

        VirtualTextureExtractor::extract_with_gts(
            gtp,
            gts_path,
            output_dir,
        ).with_context(|| "Failed to extract virtual texture")?;

        println!("Extraction complete -> {}", output_dir.display());
        return Ok(());
    }

    // Otherwise, find GTP files in the same directory as the GTS
    let gts = GtsFile::open(gts_path)
        .with_context(|| format!("Failed to parse GTS file: {}", gts_path.display()))?;

    let gts_dir = gts_path.parent().unwrap_or(Path::new("."));

    println!("Layers: {} | Page files: {}",
        gts.header.num_layers, gts.page_files.len());

    let mut extracted_count = 0;
    let total = gts.page_files.len();
    for (i, pf) in gts.page_files.iter().enumerate() {
        let gtp_path = gts_dir.join(&pf.filename);
        if gtp_path.exists() {
            println!("[{}/{}] Extracting {}...", i + 1, total, pf.filename);

            match VirtualTextureExtractor::extract_with_gts(
                &gtp_path,
                gts_path,
                output_dir,
            ) {
                Ok(()) => extracted_count += 1,
                Err(e) => eprintln!("Warning: Failed to extract {}: {}", pf.filename, e),
            }
        } else {
            eprintln!("Warning: GTP file not found: {}", gtp_path.display());
        }
    }

    println!("Extracted {} GTP files to {}", extracted_count, output_dir.display());
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

    let gts = GtsFile::open(&gts_path)
        .with_context(|| format!("Failed to parse GTS file: {}", gts_path.display()))?;

    let gtp = GtpFile::open(gtp_path, &gts)
        .with_context(|| format!("Failed to parse GTP file: {}", gtp_path.display()))?;

    println!("GTP File: {}", gtp_path.display());
    println!("GUID: {:02x?}", gtp.header.guid);
    println!("Version: {}", gtp.header.version);
    println!("Pages: {}", gtp.num_pages());

    for page in 0..gtp.num_pages() {
        println!("  Page {}: {} chunks", page, gtp.num_chunks(page));
    }

    Ok(())
}
