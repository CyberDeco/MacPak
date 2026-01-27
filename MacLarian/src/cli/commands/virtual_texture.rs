//! Virtual texture CLI commands

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Result, Context};
use indicatif::ProgressBar;

use crate::cli::progress::{bar_style, spinner_style};
use crate::virtual_texture;
use crate::virtual_texture::{VTexProgress, VTexPhase};
use crate::virtual_texture::builder::{
    VirtualTextureBuilder, SourceTexture, TileCompressionPreference,
};

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
    input_path: &Path,
    output_dir: &Path,
    _texture_name: Option<&str>,
    layers: Vec<usize>,
    _all_layers: bool,
) -> Result<()> {
    let ext = input_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let is_gtp = ext == "gtp";

    if !layers.is_empty() {
        println!("Layer filter: {:?}", layers);
    }

    // Create progress bar - use spinner for single GTP, bar for GTS with multiple GTPs
    let pb = if is_gtp {
        let pb = ProgressBar::new_spinner();
        pb.set_style(spinner_style());
        pb
    } else {
        let pb = ProgressBar::new(1); // Will be updated when we know the total
        pb.set_style(bar_style());
        pb
    };

    let result = virtual_texture::extract_gts_file(
        input_path,
        Some(output_dir),
        |progress: &VTexProgress| {
            let desc = progress.current_file.as_deref().unwrap_or(progress.phase.as_str());
            if is_gtp {
                pb.set_message(desc.to_string());
                pb.tick();
            } else {
                pb.set_length(progress.total as u64);
                pb.set_position(progress.current as u64);
                pb.set_message(desc.to_string());
            }
        },
    ).with_context(|| format!("Failed to extract {}", input_path.display()))?;

    pb.finish_and_clear();

    if is_gtp {
        println!("Extracted {} textures to {}", result.texture_count, output_dir.display());
    } else {
        println!("Extracted {}/{} GTP files to {}", result.texture_count, result.gtp_count, output_dir.display());
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

    // Create progress bar
    let pb = ProgressBar::new(gts_files.len() as u64);
    pb.set_style(bar_style());

    let result = virtual_texture::extract_batch(
        &gts_files,
        Some(output_dir),
        |progress: &VTexProgress| {
            let desc = progress.current_file.as_deref().unwrap_or(progress.phase.as_str());
            pb.set_position(progress.current as u64);
            pb.set_message(desc.to_string());
        },
    );

    pb.finish_and_clear();

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

/// Create a virtual texture set from DDS source textures
pub fn create(
    name: &str,
    base_map: Option<&Path>,
    normal_map: Option<&Path>,
    physical_map: Option<&Path>,
    output_dir: &Path,
    compression: Option<&str>,
    no_embed_mip: bool,
) -> Result<()> {
    // Build source texture
    let mut texture = SourceTexture::new(name);

    if let Some(p) = base_map {
        texture = texture.with_base_map(p);
    }
    if let Some(p) = normal_map {
        texture = texture.with_normal_map(p);
    }
    if let Some(p) = physical_map {
        texture = texture.with_physical_map(p);
    }

    if !texture.has_any_layer() {
        anyhow::bail!("At least one layer (--base, --normal, or --physical) must be specified");
    }

    // Build the virtual texture
    let mut builder = VirtualTextureBuilder::new()
        .name(name)
        .add_texture(texture);

    // Set compression preference
    if let Some(comp) = compression {
        let pref = match comp.to_lowercase().as_str() {
            "raw" => TileCompressionPreference::Raw,
            "fastlz" => TileCompressionPreference::FastLZ,
            _ => anyhow::bail!("Unknown compression: {}. Use: raw or fastlz (default)", comp),
        };
        builder = builder.compression(pref);
    }

    // Disable mip embedding if requested
    if no_embed_mip {
        builder = builder.embed_mip(false);
    }

    println!("Creating virtual texture '{}'...", name);
    println!("Output: {}", output_dir.display());

    // Create output directory
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    // Create progress bar - starts as spinner, switches to bar for compression
    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style());

    // Track current phase to detect transitions
    let last_phase = AtomicUsize::new(0);

    let result = builder.build_with_progress(output_dir, |progress: &VTexProgress| {
        let phase_num = progress.phase as usize;
        let prev_phase = last_phase.swap(phase_num, Ordering::SeqCst);

        // Switch to progress bar mode for compression phase
        if progress.phase == VTexPhase::Compressing {
            if prev_phase != phase_num {
                // Phase just changed to compressing - switch to bar style
                pb.set_style(bar_style());
                pb.set_length(progress.total as u64);
            }
            pb.set_position(progress.current as u64);
            pb.set_message(format!("Compressing tiles"));
        } else {
            // Other phases use spinner
            if prev_phase == VTexPhase::Compressing as usize && phase_num != prev_phase {
                // Just finished compression - switch back to spinner
                pb.set_style(spinner_style());
            }
            let desc = progress.current_file.as_deref().unwrap_or(progress.phase.as_str());
            pb.set_message(desc.to_string());
            pb.tick();
        }
    }).with_context(|| "Failed to create virtual texture")?;

    pb.finish_and_clear();

    println!();
    println!("Virtual texture created successfully!");
    println!("  GTS: {}", result.gts_path.display());
    for gtp in &result.gtp_paths {
        println!("  GTP: {}", gtp.display());
    }
    println!("  Tiles: {} ({} unique after deduplication)", result.tile_count, result.unique_tile_count);
    println!("  Total size: {} bytes", result.total_size_bytes);

    Ok(())
}
