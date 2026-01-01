//! GR2 CLI commands
//!
//! Commands for inspecting, converting, and decompressing GR2 files.

use std::path::Path;
use crate::operations::gr2 as gr2_ops;

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
