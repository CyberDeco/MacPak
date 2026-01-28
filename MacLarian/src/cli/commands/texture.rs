//! CLI commands for texture operations

use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;

use crate::converter::{DdsFormat, convert_dds_to_png, convert_png_to_dds_with_format};

/// Show info about a DDS texture file
pub fn info(path: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::open(path)?;
    let dds = ddsfile::Dds::read(file).map_err(|e| anyhow::anyhow!("Failed to read DDS: {e}"))?;

    println!("DDS Information: {}", path.display());
    println!();
    println!("Dimensions: {}x{}", dds.get_width(), dds.get_height());
    println!("Depth: {}", dds.get_depth());
    println!("Mip levels: {}", dds.get_num_mipmap_levels());
    println!("Array layers: {}", dds.get_num_array_layers());

    if let Some(dxgi) = dds.get_dxgi_format() {
        println!("Format: {dxgi:?} (DXGI)");
    } else if let Some(d3d) = dds.get_d3d_format() {
        println!("Format: {d3d:?} (D3D)");
    } else {
        println!("Format: Unknown");
    }

    // Calculate data size
    if let Ok(data) = dds.get_data(0) {
        println!("Data size (mip 0): {} bytes", data.len());
    }

    Ok(())
}

/// Convert a texture file (DDS<->PNG)
pub fn convert(input: &Path, output: &Path, format: Option<&str>) -> anyhow::Result<()> {
    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);
    let output_ext = output
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);

    match (input_ext.as_deref(), output_ext.as_deref()) {
        (Some("dds"), Some("png")) => {
            convert_dds_to_png(input, output)?;
            println!("Converted {} -> {}", input.display(), output.display());
        }
        (Some("png"), Some("dds")) => {
            let dds_format = parse_dds_format(format.unwrap_or("bc3"))?;
            convert_png_to_dds_with_format(input, output, dds_format)?;
            println!(
                "Converted {} -> {} ({:?})",
                input.display(),
                output.display(),
                dds_format
            );
        }
        _ => {
            anyhow::bail!(
                "Unsupported conversion: {input_ext:?} -> {output_ext:?}. Supported: DDS<->PNG"
            );
        }
    }

    Ok(())
}

/// Batch convert textures in a directory
pub fn batch_convert(
    dir: &Path,
    output_dir: &Path,
    to_format: &str,
    dds_format: Option<&str>,
) -> anyhow::Result<()> {
    let (source_ext, target_ext) = match to_format.to_lowercase().as_str() {
        "png" => ("dds", "png"),
        "dds" => ("png", "dds"),
        other => anyhow::bail!("Unknown format: {other}. Use 'png' or 'dds'"),
    };

    // Find all files with source extension
    let files: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case(source_ext))
        })
        .collect();

    if files.is_empty() {
        println!(
            "No {} files found in: {}",
            source_ext.to_uppercase(),
            dir.display()
        );
        return Ok(());
    }

    println!(
        "Converting {} {} files to {}",
        files.len(),
        source_ext.to_uppercase(),
        target_ext.to_uppercase()
    );

    std::fs::create_dir_all(output_dir)?;

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
            .expect("valid template")
            .progress_chars("##-"),
    );

    let mut success = 0;
    let mut failed = 0;

    for entry in &files {
        let relative = entry.path().strip_prefix(dir).unwrap_or(entry.path());
        let output_path = output_dir.join(relative).with_extension(target_ext);

        // Create parent directory
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        pb.set_message(relative.display().to_string());

        let result = match (source_ext, target_ext) {
            ("dds", "png") => convert_dds_to_png(entry.path(), &output_path),
            ("png", "dds") => {
                let fmt = parse_dds_format(dds_format.unwrap_or("bc3"))?;
                convert_png_to_dds_with_format(entry.path(), &output_path, fmt)
            }
            _ => unreachable!(),
        };

        match result {
            Ok(()) => success += 1,
            Err(e) => {
                failed += 1;
                tracing::warn!("Failed to convert {}: {e}", relative.display());
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    println!();
    println!("Conversion complete:");
    println!("  Success: {success}");
    println!("  Failed: {failed}");
    println!("  Output: {}", output_dir.display());

    Ok(())
}

/// Parse DDS format string
fn parse_dds_format(s: &str) -> anyhow::Result<DdsFormat> {
    match s.to_lowercase().as_str() {
        "bc1" | "dxt1" => Ok(DdsFormat::BC1),
        "bc2" | "dxt3" => Ok(DdsFormat::BC2),
        "bc3" | "dxt5" => Ok(DdsFormat::BC3),
        "rgba" | "uncompressed" => Ok(DdsFormat::Rgba),
        other => anyhow::bail!("Unknown DDS format: '{other}'. Valid options: bc1, bc2, bc3, rgba"),
    }
}
