//! GR2 CLI commands
//!
//! Commands for inspecting and converting GR2 files.

use std::path::{Path, PathBuf};

use super::expand_globs;
use crate::cli::progress::{
    CUBE, DISK, GEAR, LOOKING_GLASS, print_done, print_step, simple_spinner,
};
use crate::converter::{
    Gr2Phase, convert_gltf_to_gr2_with_progress, convert_gr2_to_glb_with_progress,
    convert_gr2_to_gltf_with_progress,
};
use crate::formats::gr2::{extract_gr2_info, inspect_gr2};

/// Default BG3 installation paths
const BG3_PATHS: &[&str] = &[
    // macOS
    "~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/Baldur's Gate 3.app/Contents/Data",
    // Windows
    "C:/Program Files (x86)/Steam/steamapps/common/Baldurs Gate 3/Data",
    // Linux
    "~/.steam/steam/steamapps/common/Baldurs Gate 3/Data",
];

/// Try to find BG3 installation path
fn find_bg3_path() -> Option<std::path::PathBuf> {
    for path in BG3_PATHS {
        let expanded = shellexpand::tilde(path);
        let path = std::path::Path::new(expanded.as_ref());
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

/// Inspect a GR2 file and display its structure.
pub fn inspect(path: &Path, output: Option<&Path>) -> anyhow::Result<()> {
    if let Some(out_path) = output {
        // Output to JSON file
        let model_info = extract_gr2_info(path)?;
        let json = serde_json::to_string_pretty(&model_info)?;
        std::fs::write(out_path, json)?;
        println!("Written to: {}", out_path.display());
    } else {
        // Print to CLI
        println!("Inspecting GR2 file: {}", path.display());
        println!();

        let info = inspect_gr2(path)?;

        println!("GR2 File Information");
        println!("====================");
        println!("Version:     {}", info.version);
        println!("Format:      {}-bit", if info.is_64bit { 64 } else { 32 });
        println!("File size:   {} bytes", info.file_size);
        println!("Sections:    {}", info.num_sections);
        println!();

        println!("Sections:");
        println!("---------");
        for section in &info.sections {
            let ratio = section
                .compression_ratio
                .map_or_else(|| "N/A".to_string(), |r| format!("{r:.2}x"));
            println!(
                "  [{:2}] {:8} | {:>8} -> {:>8} bytes ({})",
                section.index,
                section.compression,
                section.compressed_size,
                section.uncompressed_size,
                ratio
            );
        }

        // Also show mesh/skeleton info
        println!();
        match extract_gr2_info(path) {
            Ok(model_info) => {
                if let Some(ref skel) = model_info.skeleton {
                    println!("Skeleton: {} ({} bones)", skel.name, skel.bone_count);
                } else {
                    println!("Skeleton: None");
                }
                println!();
                println!("Meshes ({}):", model_info.meshes.len());
                for mesh in &model_info.meshes {
                    println!(
                        "  - {} ({} vertices, {} triangles)",
                        mesh.name, mesh.vertex_count, mesh.triangle_count
                    );
                }
            }
            Err(e) => {
                println!("(Could not parse mesh data: {e})");
            }
        }
    }

    Ok(())
}

/// Convert GR2 to glTF/GLB format.
pub fn from_gr2(
    sources: &[PathBuf],
    destination: &Path,
    format: &str,
    textures: Option<&str>,
    bg3_path: Option<&Path>,
    quiet: bool,
) -> anyhow::Result<()> {
    // Expand glob patterns
    let sources = expand_globs(sources)?;

    // Handle batch conversion
    if sources.len() > 1 {
        return from_gr2_batch(&sources, destination, format, textures, bg3_path, quiet);
    }

    let source = &sources[0];
    from_gr2_single(source, destination, format, textures, bg3_path, quiet)
}

/// Convert a single GR2 file to glTF/GLB
fn from_gr2_single(
    source: &Path,
    destination: &Path,
    format: &str,
    textures: Option<&str>,
    bg3_path: Option<&Path>,
    quiet: bool,
) -> anyhow::Result<()> {
    let use_gltf = format.to_lowercase() == "gltf";
    let format_name = if use_gltf { "glTF" } else { "GLB" };

    // Validate textures option
    let texture_mode = match textures {
        Some("extract") => Some(TextureMode::Extract),
        Some("embedded") => {
            if use_gltf {
                anyhow::bail!("--textures embedded is only valid with GLB format (--format glb)");
            }
            Some(TextureMode::Embedded)
        }
        Some(other) => {
            anyhow::bail!("Invalid --textures value: '{other}'. Use 'extract' or 'embedded'")
        }
        None => None,
    };

    // Find BG3 path if needed for textures
    let bg3_install = if texture_mode.is_some() {
        if let Some(path) = bg3_path {
            Some(path.to_path_buf())
        } else if let Some(path) = find_bg3_path() {
            if !quiet {
                println!("Auto-detected BG3 path: {}", path.display());
            }
            Some(path)
        } else {
            anyhow::bail!(
                "BG3 installation not found. Please provide --bg3-path.\n\
                 Searched locations:\n\
                 - macOS: ~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/\n\
                 - Windows: C:/Program Files (x86)/Steam/steamapps/common/Baldurs Gate 3/\n\
                 - Linux: ~/.steam/steam/steamapps/common/Baldurs Gate 3/"
            );
        }
    } else {
        None
    };

    if !quiet {
        println!("Converting GR2 to {format_name}...");
        println!("  Source:      {}", source.display());
        println!("  Destination: {}", destination.display());
        if let Some(mode) = &texture_mode {
            println!(
                "  Textures:    {}",
                match mode {
                    TextureMode::Extract => "extract (separate files)",
                    TextureMode::Embedded => "embedded (in GLB)",
                }
            );
        }
        println!();
    }

    let start = std::time::Instant::now();

    match texture_mode {
        Some(TextureMode::Embedded) => {
            // Use the textured GLB conversion
            let gr2_data = std::fs::read(source)?;
            let gr2_filename = source
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.GR2");

            let textures_pak = bg3_install.as_ref().unwrap().join("Textures.pak");

            if !quiet {
                let pb = simple_spinner("Converting with embedded textures...");
                let result = crate::converter::gr2_gltf::convert_gr2_bytes_to_glb_with_textures(
                    &gr2_data,
                    gr2_filename,
                    &textures_pak,
                )?;
                pb.finish_and_clear();

                std::fs::write(destination, &result.glb_data)?;

                println!();
                print_done(start.elapsed());
                println!("  Output size: {} bytes", result.glb_data.len());

                if !result.warnings.is_empty() {
                    println!("\nWarnings:");
                    for warning in &result.warnings {
                        println!("  - {warning}");
                    }
                }
            } else {
                let result = crate::converter::gr2_gltf::convert_gr2_bytes_to_glb_with_textures(
                    &gr2_data,
                    gr2_filename,
                    &textures_pak,
                )?;
                std::fs::write(destination, &result.glb_data)?;
            }
        }
        Some(TextureMode::Extract) => {
            // Create output directory and extract textures alongside GLB
            let output_dir = if destination.is_dir() {
                destination.to_path_buf()
            } else {
                destination.parent().unwrap_or(Path::new(".")).to_path_buf()
            };
            std::fs::create_dir_all(&output_dir)?;

            // Copy GR2 to output dir for processing
            let gr2_in_output = output_dir.join(source.file_name().unwrap_or_default());
            if source != gr2_in_output {
                std::fs::copy(source, &gr2_in_output)?;
            }

            // Build extraction options
            let options = crate::gr2_extraction::Gr2ExtractionOptions::default()
                .with_convert_to_glb(!use_gltf)
                .with_extract_textures(true)
                .with_extract_virtual_textures(true)
                .with_bg3_path(bg3_install);

            if !quiet {
                let pb = simple_spinner("Processing GR2 with textures...");
                let result =
                    crate::gr2_extraction::process_extracted_gr2(&gr2_in_output, &options)?;
                pb.finish_and_clear();

                println!();
                print_done(start.elapsed());

                if let Some(glb) = &result.glb_path {
                    let size = std::fs::metadata(glb).map(|m| m.len()).unwrap_or(0);
                    println!("  {}: {} ({} bytes)", format_name, glb.display(), size);
                }

                if !result.texture_paths.is_empty() {
                    println!("  Textures extracted: {}", result.texture_paths.len());
                    for tex_path in &result.texture_paths {
                        println!(
                            "    - {}",
                            tex_path.file_name().unwrap_or_default().to_string_lossy()
                        );
                    }
                }

                if !result.warnings.is_empty() {
                    println!("\nWarnings:");
                    for warning in &result.warnings {
                        println!("  - {warning}");
                    }
                }
            } else {
                crate::gr2_extraction::process_extracted_gr2(&gr2_in_output, &options)?;
            }
        }
        None => {
            // Simple conversion without textures
            if use_gltf {
                if !quiet {
                    convert_gr2_to_gltf_with_progress(source, destination, &|progress| {
                        let emoji = match progress.phase {
                            Gr2Phase::ReadingFile => LOOKING_GLASS,
                            Gr2Phase::ParsingSkeleton | Gr2Phase::ParsingMeshes => CUBE,
                            Gr2Phase::BuildingDocument => GEAR,
                            Gr2Phase::WritingOutput => DISK,
                            _ => GEAR,
                        };
                        print_step(
                            progress.current,
                            progress.total,
                            emoji,
                            progress.phase.as_str(),
                        );
                    })?;
                } else {
                    crate::converter::convert_gr2_to_gltf(source, destination)?;
                }
            } else if !quiet {
                convert_gr2_to_glb_with_progress(source, destination, &|progress| {
                    let emoji = match progress.phase {
                        Gr2Phase::ReadingFile => LOOKING_GLASS,
                        Gr2Phase::ParsingSkeleton | Gr2Phase::ParsingMeshes => CUBE,
                        Gr2Phase::BuildingDocument => GEAR,
                        Gr2Phase::WritingOutput => DISK,
                        _ => GEAR,
                    };
                    print_step(
                        progress.current,
                        progress.total,
                        emoji,
                        progress.phase.as_str(),
                    );
                })?;
            } else {
                crate::converter::convert_gr2_to_glb(source, destination)?;
            }

            if !quiet {
                let output_size = std::fs::metadata(destination)?.len();
                println!();
                print_done(start.elapsed());
                println!("  Output size: {output_size} bytes");
            }
        }
    }

    Ok(())
}

/// Batch convert multiple GR2 files to glTF/GLB
fn from_gr2_batch(
    sources: &[PathBuf],
    destination: &Path,
    format: &str,
    textures: Option<&str>,
    bg3_path: Option<&Path>,
    quiet: bool,
) -> anyhow::Result<()> {
    let use_gltf = format.to_lowercase() == "gltf";
    let out_ext = if use_gltf { "gltf" } else { "glb" };

    // Ensure destination directory exists
    std::fs::create_dir_all(destination)?;

    println!(
        "Batch converting {} GR2 files to {}",
        sources.len(),
        out_ext.to_uppercase()
    );

    let mut success = 0;
    let mut failed = 0;

    for source in sources {
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let dest_file = destination.join(format!("{stem}.{out_ext}"));

        if !quiet {
            println!("Converting: {}", source.display());
        }

        match from_gr2_single(source, &dest_file, format, textures, bg3_path, true) {
            Ok(()) => {
                success += 1;
            }
            Err(e) => {
                eprintln!("Failed to convert {}: {e}", source.display());
                failed += 1;
            }
        }
    }

    println!();
    println!("Batch conversion complete:");
    println!("  Success: {success}");
    if failed > 0 {
        println!("  Failed: {failed}");
    }

    Ok(())
}

/// Convert glTF/GLB to GR2 format.
pub fn to_gr2(sources: &[PathBuf], destination: &Path, quiet: bool) -> anyhow::Result<()> {
    // Expand glob patterns
    let sources = expand_globs(sources)?;

    // Handle batch conversion
    if sources.len() > 1 {
        return to_gr2_batch(&sources, destination, quiet);
    }

    let source = &sources[0];
    to_gr2_single(source, destination, quiet)
}

/// Convert a single glTF/GLB file to GR2
fn to_gr2_single(source: &Path, destination: &Path, quiet: bool) -> anyhow::Result<()> {
    if !quiet {
        println!("Converting glTF to GR2...");
        println!("  Source:      {}", source.display());
        println!("  Destination: {}", destination.display());
        println!();
        println!("Note: Output will be uncompressed (compression not yet implemented)");
        println!();
    }

    let start = std::time::Instant::now();

    if !quiet {
        convert_gltf_to_gr2_with_progress(source, destination, &|progress| {
            let emoji = match progress.phase {
                Gr2Phase::LoadingFile => LOOKING_GLASS,
                Gr2Phase::ParsingModel => CUBE,
                Gr2Phase::BuildingGr2 => GEAR,
                Gr2Phase::WritingFile => DISK,
                _ => GEAR,
            };
            print_step(
                progress.current,
                progress.total,
                emoji,
                progress.phase.as_str(),
            );
        })?;

        let output_size = std::fs::metadata(destination)?.len();
        println!();
        print_done(start.elapsed());
        println!("  Output size: {output_size} bytes");
    } else {
        crate::converter::convert_gltf_to_gr2(source, destination)?;
    }

    Ok(())
}

/// Batch convert multiple glTF/GLB files to GR2
fn to_gr2_batch(sources: &[PathBuf], destination: &Path, quiet: bool) -> anyhow::Result<()> {
    // Ensure destination directory exists
    std::fs::create_dir_all(destination)?;

    println!("Batch converting {} glTF/GLB files to GR2", sources.len());
    if !quiet {
        println!("Note: Output will be uncompressed (compression not yet implemented)");
    }

    let mut success = 0;
    let mut failed = 0;

    for source in sources {
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let dest_file = destination.join(format!("{stem}.GR2"));

        if !quiet {
            println!("Converting: {}", source.display());
        }

        match to_gr2_single(source, &dest_file, true) {
            Ok(()) => {
                success += 1;
            }
            Err(e) => {
                eprintln!("Failed to convert {}: {e}", source.display());
                failed += 1;
            }
        }
    }

    println!();
    println!("Batch conversion complete:");
    println!("  Success: {success}");
    if failed > 0 {
        println!("  Failed: {failed}");
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum TextureMode {
    Extract,
    Embedded,
}
