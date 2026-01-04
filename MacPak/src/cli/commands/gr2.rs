//! GR2 CLI commands
//!
//! Commands for inspecting, converting, and decompressing GR2 files.

use std::path::Path;
use crate::operations::gr2 as gr2_ops;
use MacLarian::gr2_extraction::{process_extracted_gr2, Gr2ExtractionOptions};
use MacLarian::converter::gr2_to_gltf::convert_gr2_bytes_to_glb_with_textures;

/// Inspect a GR2 file and display its structure.
pub fn inspect(path: &Path) -> anyhow::Result<()> {
    println!("Inspecting GR2 file: {}", path.display());
    println!();

    let info = gr2_ops::inspect_gr2(path)?;

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
        let ratio = section.compression_ratio
            .map(|r| format!("{:.2}x", r))
            .unwrap_or_else(|| "N/A".to_string());
        println!(
            "  [{:2}] {:8} | {:>8} → {:>8} bytes ({})",
            section.index,
            section.compression,
            section.compressed_size,
            section.uncompressed_size,
            ratio
        );
    }

    // Also show mesh/skeleton info
    println!();
    match gr2_ops::extract_gr2_info(path) {
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
            println!("(Could not parse mesh data: {})", e);
        }
    }

    Ok(())
}

/// Extract mesh information to JSON.
pub fn extract_json(path: &Path, output: &Path) -> anyhow::Result<()> {
    println!("Extracting GR2 info to JSON: {}", path.display());

    let model_info = gr2_ops::extract_gr2_info(path)?;
    let json = serde_json::to_string_pretty(&model_info)?;
    std::fs::write(output, json)?;

    println!("Written to: {}", output.display());
    Ok(())
}

/// Decompress a GR2 file.
pub fn decompress(path: &Path, output: Option<&Path>) -> anyhow::Result<()> {
    let output_path = if let Some(out) = output {
        out.to_path_buf()
    } else {
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("gr2");
        path.with_file_name(format!("{}_decompressed.{}", stem, ext))
    };

    println!("Decompressing GR2 file...");
    println!("  Source:      {}", path.display());
    println!("  Destination: {}", output_path.display());

    gr2_ops::decompress_gr2(path, &output_path)?;

    let original_size = std::fs::metadata(path)?.len();
    let decompressed_size = std::fs::metadata(&output_path)?.len();

    println!();
    println!("Decompression complete!");
    println!("  Original size:     {} bytes", original_size);
    println!("  Decompressed size: {} bytes", decompressed_size);

    Ok(())
}

/// Convert GR2 to GLB format.
pub fn convert_to_glb(path: &Path, output: Option<&Path>) -> anyhow::Result<()> {
    let output_path = if let Some(out) = output {
        out.to_path_buf()
    } else {
        path.with_extension("glb")
    };

    println!("Converting GR2 to GLB...");
    println!("  Source:      {}", path.display());
    println!("  Destination: {}", output_path.display());

    gr2_ops::gr2_to_glb(path, &output_path)?;

    let output_size = std::fs::metadata(&output_path)?.len();
    println!();
    println!("Conversion complete!");
    println!("  Output size: {} bytes", output_size);

    Ok(())
}

/// Convert GLB/glTF to GR2 format.
pub fn convert_to_gr2(path: &Path, output: Option<&Path>) -> anyhow::Result<()> {
    let output_path = if let Some(out) = output {
        out.to_path_buf()
    } else {
        path.with_extension("gr2")
    };

    println!("Converting glTF to GR2...");
    println!("  Source:      {}", path.display());
    println!("  Destination: {}", output_path.display());
    println!();
    println!("Note: Output will be uncompressed (compression not yet implemented)");

    gr2_ops::gltf_to_gr2(path, &output_path)?;

    let output_size = std::fs::metadata(&output_path)?.len();
    println!();
    println!("Conversion complete!");
    println!("  Output size: {} bytes", output_size);

    Ok(())
}

/// Convert GR2 to GLB with embedded textures (for testing).
pub fn convert_to_glb_textured(
    path: &Path,
    textures_pak: &Path,
    output: Option<&Path>,
) -> anyhow::Result<()> {
    let output_path = if let Some(out) = output {
        out.to_path_buf()
    } else {
        path.with_extension("textured.glb")
    };

    println!("Converting GR2 to textured GLB...");
    println!("  Source:       {}", path.display());
    println!("  Textures PAK: {}", textures_pak.display());
    println!("  Destination:  {}", output_path.display());
    println!();

    // Read GR2 bytes
    let gr2_data = std::fs::read(path)?;
    let gr2_filename = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown.GR2");

    println!("  GR2 filename for lookup: {}", gr2_filename);

    // Convert with textures
    let result = convert_gr2_bytes_to_glb_with_textures(&gr2_data, gr2_filename, textures_pak)?;

    // Write output
    std::fs::write(&output_path, &result.glb_data)?;

    println!();
    println!("Conversion complete!");
    println!("  Output size: {} bytes", result.glb_data.len());

    if !result.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &result.warnings {
            println!("  - {}", warning);
        }
    }

    Ok(())
}

/// Bundle a GR2 file: convert to GLB and extract associated textures.
pub fn bundle(
    path: &Path,
    output: Option<&Path>,
    game_data: Option<&Path>,
    virtual_textures: Option<&Path>,
    no_glb: bool,
    no_textures: bool,
) -> anyhow::Result<()> {
    // Determine output directory
    let output_dir = if let Some(out) = output {
        out.to_path_buf()
    } else {
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    };

    println!("Bundling GR2 file with textures...");
    println!("  Source:     {}", path.display());
    println!("  Output dir: {}", output_dir.display());

    // Build options
    let mut options = Gr2ExtractionOptions::default();

    if no_glb {
        options = options.no_conversion();
        println!("  GLB conversion: disabled");
    }

    if no_textures {
        options = options.no_textures();
        println!("  Texture extraction: disabled");
    }

    if let Some(gd) = game_data {
        options = options.with_game_data_path(gd);
        println!("  Game data: {}", gd.display());
    }

    if let Some(vt) = virtual_textures {
        options = options.with_virtual_textures_path(vt);
        println!("  Virtual textures: {}", vt.display());
    }

    println!();

    // If output dir is different from source dir, copy GR2 there first
    let gr2_in_output = if output_dir != path.parent().unwrap_or(Path::new("")) {
        std::fs::create_dir_all(&output_dir)?;
        let dest = output_dir.join(path.file_name().unwrap_or_default());
        std::fs::copy(path, &dest)?;
        dest
    } else {
        path.to_path_buf()
    };

    // Process the GR2
    let result = process_extracted_gr2(&gr2_in_output, &options)?;

    // Report results
    println!("Bundle complete!");
    println!();

    if let Some(glb) = &result.glb_path {
        let size = std::fs::metadata(glb).map(|m| m.len()).unwrap_or(0);
        println!("  GLB: {} ({} bytes)", glb.file_name().unwrap_or_default().to_string_lossy(), size);
    }

    if !result.texture_paths.is_empty() {
        println!("  Textures extracted: {}", result.texture_paths.len());
        for tex_path in &result.texture_paths {
            println!("    - {}", tex_path.file_name().unwrap_or_default().to_string_lossy());
        }
    } else if !no_textures {
        println!("  Textures: none found in database");
    }

    if !result.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &result.warnings {
            println!("  - {}", warning);
        }
    }

    Ok(())
}
