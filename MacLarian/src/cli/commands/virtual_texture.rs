//! Virtual texture CLI commands

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use indicatif::ProgressBar;
use serde::Serialize;

use super::expand_globs;
use crate::cli::progress::{bar_style, spinner_style};
use crate::virtual_texture;
use crate::virtual_texture::builder::{
    SourceTexture, TileCompressionPreference, VirtualTextureBuilder,
};
use crate::virtual_texture::mod_config::{DiscoverySource, discover_virtual_textures};
use crate::virtual_texture::{VTexPhase, VTexProgress};

/// JSON output format for discovered virtual textures
#[derive(Serialize)]
struct DiscoveredTextureJson {
    mod_name: String,
    gtex_hash: String,
    tileset_name: Option<String>,
    gts_path: String,
    source: String,
}

/// Discover virtual textures in mod directories
pub fn discover(sources: &[PathBuf], output: Option<&Path>, quiet: bool) -> Result<()> {
    // Expand glob patterns
    let sources = expand_globs(sources)?;

    let discovered = discover_virtual_textures(&sources)
        .with_context(|| "Failed to discover virtual textures")?;

    if discovered.is_empty() {
        if !quiet {
            println!("No virtual textures found in the specified paths.");
        }
        return Ok(());
    }

    // Convert to JSON-serializable format
    let json_data: Vec<DiscoveredTextureJson> = discovered
        .iter()
        .map(|vt| DiscoveredTextureJson {
            mod_name: vt.mod_name.clone(),
            gtex_hash: vt.gtex_hash.clone(),
            tileset_name: vt.tileset_name.clone(),
            gts_path: vt.gts_path.display().to_string(),
            source: match vt.source {
                DiscoverySource::VTexConfigXml => "VTexConfig.xml".to_string(),
                DiscoverySource::VirtualTexturesJson => "VirtualTextures.json".to_string(),
                DiscoverySource::GtsFileScan => "GTS file scan".to_string(),
            },
        })
        .collect();

    if let Some(out_path) = output {
        // Output to JSON file
        let json = serde_json::to_string_pretty(&json_data)?;
        std::fs::write(out_path, json)?;
        if !quiet {
            println!(
                "Discovered {} virtual texture(s), written to: {}",
                discovered.len(),
                out_path.display()
            );
        }
        return Ok(());
    }

    // Print to CLI
    if !quiet {
        println!("Discovered {} virtual texture(s):\n", discovered.len());
    }

    for vt in &discovered {
        let source_str = match vt.source {
            DiscoverySource::VTexConfigXml => "VTexConfig.xml",
            DiscoverySource::VirtualTexturesJson => "VirtualTextures.json",
            DiscoverySource::GtsFileScan => "GTS file scan",
        };

        println!("Mod: {}", vt.mod_name);
        println!("  GTex:    {}", vt.gtex_hash);
        if let Some(ref tileset) = vt.tileset_name {
            println!("  TileSet: {tileset}");
        }
        println!("  GTS:     {}", vt.gts_path.display());
        println!("  Source:  {source_str}");
        println!();
    }

    Ok(())
}

/// List textures in a GTS file
pub fn list(gts_path: &Path, detailed: bool, output: Option<&Path>) -> Result<()> {
    let info = virtual_texture::list_gts(gts_path)
        .with_context(|| format!("Failed to parse GTS file: {}", gts_path.display()))?;

    if let Some(out_path) = output {
        // Output to JSON file
        let json = serde_json::to_string_pretty(&info)?;
        std::fs::write(out_path, json)?;
        println!("Written to: {}", out_path.display());
        return Ok(());
    }

    // Print to CLI
    println!("Virtual Texture Set: {}", gts_path.display());
    println!("GUID: {:02x?}", info.guid);
    println!("Version: {}", info.version);
    println!(
        "Tile size: {}x{} (border: {})",
        info.tile_width, info.tile_height, info.tile_border
    );
    println!("Layers: {}", info.num_layers);
    println!("Levels: {}", info.num_levels);
    println!("Page files: {}", info.page_files.len());
    println!();

    if detailed {
        println!("Page files:");
        for (i, pf) in info.page_files.iter().enumerate() {
            println!("  [{}] {} ({} pages)", i, pf.filename, pf.num_pages);
        }
    } else {
        println!(
            "Page files: {} (use -d for full list)",
            info.page_files.len()
        );
    }

    Ok(())
}

/// Extract textures from GTS/GTP files
pub fn extract(
    sources: &[PathBuf],
    output_dir: &Path,
    gtex_filter: Option<&str>,
    layers: &[usize],
    quiet: bool,
) -> Result<()> {
    // Expand glob patterns
    let sources = expand_globs(sources)?;

    // Handle multiple sources (batch extraction)
    if sources.len() > 1 {
        return extract_batch(&sources, output_dir, layers, quiet);
    }

    let input_path = &sources[0];
    let ext = input_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let is_gtp = ext == "gtp";

    if !quiet {
        if let Some(gtex) = gtex_filter {
            println!("Texture filter: {gtex}");
        }
        if !layers.is_empty() {
            println!("Layer filter: {layers:?}");
        }
    }

    // Create progress bar - use spinner for single GTP, bar for GTS with multiple GTPs
    let pb = if quiet {
        None
    } else if is_gtp {
        let pb = ProgressBar::new_spinner();
        pb.set_style(spinner_style());
        Some(pb)
    } else {
        let pb = ProgressBar::new(1);
        pb.set_style(bar_style());
        Some(pb)
    };

    let result = virtual_texture::extract_gts_file(
        input_path,
        Some(output_dir),
        |progress: &VTexProgress| {
            if let Some(ref pb) = pb {
                let desc = progress
                    .current_file
                    .as_deref()
                    .unwrap_or(progress.phase.as_str());
                if is_gtp {
                    pb.set_message(desc.to_string());
                    pb.tick();
                } else {
                    pb.set_length(progress.total as u64);
                    pb.set_position(progress.current as u64);
                    pb.set_message(desc.to_string());
                }
            }
        },
    )
    .with_context(|| format!("Failed to extract {}", input_path.display()))?;

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    if !quiet {
        if is_gtp {
            println!(
                "Extracted {} textures to {}",
                result.texture_count,
                output_dir.display()
            );
        } else {
            println!(
                "Extracted {}/{} GTP files to {}",
                result.texture_count,
                result.gtp_count,
                output_dir.display()
            );
        }
    }

    Ok(())
}

/// Batch extract multiple GTS files
fn extract_batch(
    sources: &[PathBuf],
    output_dir: &Path,
    layers: &[usize],
    quiet: bool,
) -> Result<()> {
    if !quiet {
        println!("Batch extracting {} files", sources.len());
        if !layers.is_empty() {
            println!("Layer filter: {layers:?}");
        }
    }

    // Create output directory
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Create progress bar
    let pb = if quiet {
        None
    } else {
        let pb = ProgressBar::new(sources.len() as u64);
        pb.set_style(bar_style());
        Some(pb)
    };

    let result =
        virtual_texture::extract_batch(sources, Some(output_dir), |progress: &VTexProgress| {
            if let Some(ref pb) = pb {
                let desc = progress
                    .current_file
                    .as_deref()
                    .unwrap_or(progress.phase.as_str());
                pb.set_position(progress.current as u64);
                pb.set_message(desc.to_string());
            }
        });

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    if !quiet {
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
                    println!("  {msg}");
                }
            }
        }
    }

    Ok(())
}

/// Create a virtual texture set from DDS source textures
pub fn create(
    source_dir: &Path,
    output_dir: &Path,
    gtex_name: Option<&str>,
    base_map: Option<&Path>,
    normal_map: Option<&Path>,
    physical_map: Option<&Path>,
    compression: &str,
    no_embed_mip: bool,
    quiet: bool,
) -> Result<()> {
    // Determine texture name
    let name = gtex_name.unwrap_or_else(|| {
        source_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("texture")
    });

    // Build source texture - either from explicit paths or auto-detect
    let mut texture = SourceTexture::new(name);

    if let Some(p) = base_map {
        texture = texture.with_base_map(p);
    } else {
        // Auto-detect base map
        if let Some(path) = find_layer_file(
            source_dir,
            &["_BaseMap", "_BM", "_Base", "_Diffuse", "_Albedo"],
        ) {
            texture = texture.with_base_map(&path);
            if !quiet {
                println!("Auto-detected base map: {}", path.display());
            }
        }
    }

    if let Some(p) = normal_map {
        texture = texture.with_normal_map(p);
    } else {
        // Auto-detect normal map
        if let Some(path) = find_layer_file(source_dir, &["_NormalMap", "_NM", "_Normal"]) {
            texture = texture.with_normal_map(&path);
            if !quiet {
                println!("Auto-detected normal map: {}", path.display());
            }
        }
    }

    if let Some(p) = physical_map {
        texture = texture.with_physical_map(p);
    } else {
        // Auto-detect physical map
        if let Some(path) = find_layer_file(source_dir, &["_PhysicalMap", "_PM", "_Physical"]) {
            texture = texture.with_physical_map(&path);
            if !quiet {
                println!("Auto-detected physical map: {}", path.display());
            }
        }
    }

    if !texture.has_any_layer() {
        anyhow::bail!(
            "No texture layers found. Provide --base, --normal, or --physical, or ensure\n\
             DDS files are named with suffixes like _BaseMap, _NormalMap, _PhysicalMap"
        );
    }

    // Build the virtual texture
    let mut builder = VirtualTextureBuilder::new().name(name).add_texture(texture);

    // Set compression preference
    let pref = match compression.to_lowercase().as_str() {
        "raw" => TileCompressionPreference::Raw,
        "fastlz" => TileCompressionPreference::FastLZ,
        _ => anyhow::bail!("Unknown compression: {compression}. Use: raw or fastlz (default)"),
    };
    builder = builder.compression(pref);

    // Disable mip embedding if requested
    if no_embed_mip {
        builder = builder.embed_mip(false);
    }

    if !quiet {
        println!("Creating virtual texture '{name}'...");
        println!("Output: {}", output_dir.display());
    }

    // Create output directory
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Create progress bar - starts as spinner, switches to bar for compression
    let pb = if quiet {
        None
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(spinner_style());
        Some(pb)
    };

    // Track current phase to detect transitions
    let last_phase = AtomicUsize::new(0);

    let result = builder
        .build_with_progress(output_dir, |progress: &VTexProgress| {
            if let Some(ref pb) = pb {
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
                    pb.set_message("Compressing tiles".to_string());
                } else {
                    // Other phases use spinner
                    if prev_phase == VTexPhase::Compressing as usize && phase_num != prev_phase {
                        // Just finished compression - switch back to spinner
                        pb.set_style(spinner_style());
                    }
                    let desc = progress
                        .current_file
                        .as_deref()
                        .unwrap_or(progress.phase.as_str());
                    pb.set_message(desc.to_string());
                    pb.tick();
                }
            }
        })
        .with_context(|| "Failed to create virtual texture")?;

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    if !quiet {
        println!();
        println!("Virtual texture created successfully!");
        println!("  GTS: {}", result.gts_path.display());
        for gtp in &result.gtp_paths {
            println!("  GTP: {}", gtp.display());
        }
        println!(
            "  Tiles: {} ({} unique after deduplication)",
            result.tile_count, result.unique_tile_count
        );
        println!("  Total size: {} bytes", result.total_size_bytes);
    }

    Ok(())
}

/// Find a DDS file with one of the given suffixes in a directory
fn find_layer_file(dir: &Path, suffixes: &[&str]) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("dds") {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        for suffix in suffixes {
                            if stem.to_lowercase().ends_with(&suffix.to_lowercase()) {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
