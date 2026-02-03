//! CLI interface for format conversion

use std::path::{Path, PathBuf};

use super::expand_globs;
use crate::cli::progress::simple_spinner;

/// Execute format conversion for the given sources and destination.
pub fn execute(
    sources: &[PathBuf],
    destination: &Path,
    output_format: Option<&str>,
    texture_format: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    // Expand glob patterns
    let sources = expand_globs(sources)?;

    // Handle multiple sources (batch conversion)
    if sources.len() > 1 {
        return convert_batch(&sources, destination, output_format, texture_format, quiet);
    }

    let source = &sources[0];
    convert_single(source, destination, output_format, texture_format, quiet)
}

fn convert_single(
    source: &Path,
    destination: &Path,
    output_format: Option<&str>,
    texture_format: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    if !quiet {
        println!(
            "Converting {} to {}",
            source.display(),
            destination.display()
        );
    }

    // Auto-detect input format
    let input = source
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_lowercase)
        .ok_or_else(|| anyhow::anyhow!("Cannot detect input format from source file extension"))?;

    // Use provided output format or auto-detect
    let output = if let Some(fmt) = output_format {
        fmt.to_lowercase()
    } else {
        destination
            .extension()
            .and_then(|s| s.to_str())
            .map(str::to_lowercase)
            .ok_or_else(|| {
                anyhow::anyhow!("Cannot detect output format from destination file extension")
            })?
    };

    // Check for unsupported LSB format
    if input == "lsb" || output == "lsb" {
        anyhow::bail!(
            "LSB format is not supported.\n\
             \n\
             The LSB format was deprecated by Larian Studios in Patch 6 (February 2024)\n\
             and replaced with the LSF format. Modern BG3 uses LSF files exclusively.\n\
             \n\
             If you need to work with legacy Early Access save files, please use\n\
             the original LSLib tool: https://github.com/Norbyte/lslib"
        );
    }

    // Execute conversion based on input/output format
    match (input.as_str(), output.as_str()) {
        // LSF conversions
        ("lsf" | "lsbc" | "lsbs" | "lsfx", "lsx") => {
            if !quiet {
                let pb = simple_spinner("Converting LSF -> LSX...");
                crate::converter::lsf_to_lsx_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::lsf_to_lsx(source, destination)?;
            }
        }
        ("lsf" | "lsbc" | "lsbs" | "lsfx", "lsj") => {
            if !quiet {
                let pb = simple_spinner("Converting LSF -> LSJ...");
                crate::converter::lsf_to_lsj_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::lsf_to_lsj(source, destination)?;
            }
        }

        // LSX conversions
        ("lsx", "lsf" | "lsbc" | "lsbs" | "lsfx") => {
            if !quiet {
                let pb = simple_spinner("Converting LSX -> LSF...");
                crate::converter::lsx_to_lsf_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::lsx_to_lsf(source, destination)?;
            }
        }
        ("lsx", "lsj") => {
            if !quiet {
                let pb = simple_spinner("Converting LSX -> LSJ...");
                crate::converter::lsx_to_lsj_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::lsx_to_lsj(source, destination)?;
            }
        }

        // LSJ conversions
        ("lsj", "lsx") => {
            if !quiet {
                let pb = simple_spinner("Converting LSJ -> LSX...");
                crate::converter::lsj_to_lsx_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::lsj_to_lsx(source, destination)?;
            }
        }
        ("lsj", "lsf" | "lsbc" | "lsbs" | "lsfx") => {
            if !quiet {
                let pb = simple_spinner("Converting LSJ -> LSF...");
                crate::converter::lsj_to_lsf_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::lsj_to_lsf(source, destination)?;
            }
        }

        // GR2/glTF conversions
        ("gr2", "glb") => {
            if !quiet {
                let pb = simple_spinner("Converting GR2 -> GLB...");
                crate::converter::convert_gr2_to_glb_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::convert_gr2_to_glb(source, destination)?;
            }
        }
        ("gr2", "gltf") => {
            if !quiet {
                let pb = simple_spinner("Converting GR2 -> glTF...");
                crate::converter::convert_gr2_to_gltf_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::convert_gr2_to_gltf(source, destination)?;
            }
        }
        ("glb" | "gltf", "gr2") => {
            if !quiet {
                println!("Note: Output will be uncompressed (compression not yet implemented)");
                let pb = simple_spinner("Converting glTF -> GR2...");
                crate::converter::convert_gltf_to_gr2_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::convert_gltf_to_gr2(source, destination)?;
            }
        }

        // LOCA conversions
        ("loca", "xml") => {
            if !quiet {
                let pb = simple_spinner("Converting LOCA -> XML...");
                crate::converter::convert_loca_to_xml_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::loca::convert_loca_to_xml(source, destination)?;
            }
        }
        ("xml", "loca") => {
            if !quiet {
                let pb = simple_spinner("Converting XML -> LOCA...");
                crate::converter::convert_xml_to_loca_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::loca::convert_xml_to_loca(source, destination)?;
            }
        }

        // DDS/PNG conversions
        ("dds", "png") => {
            if !quiet {
                let pb = simple_spinner("Converting DDS -> PNG...");
                crate::converter::convert_dds_to_png_with_progress(source, destination, &|p| {
                    if let Some(ref msg) = p.current_file {
                        pb.set_message(msg.clone());
                    }
                })?;
                pb.finish_and_clear();
            } else {
                crate::converter::convert_dds_to_png(source, destination)?;
            }
        }
        ("png", "dds") => {
            let dds_format = parse_dds_format(texture_format)?;
            if !quiet {
                let pb = simple_spinner(&format!("Converting PNG -> DDS ({texture_format})..."));
                crate::converter::convert_png_to_dds_with_format(source, destination, dds_format)?;
                pb.finish_and_clear();
            } else {
                crate::converter::convert_png_to_dds_with_format(source, destination, dds_format)?;
            }
        }

        // Same format (copy)
        (fmt1, fmt2) if fmt1 == fmt2 => {
            if !quiet {
                println!("Source and destination formats are the same, copying file...");
            }
            std::fs::copy(source, destination)?;
        }

        // Unsupported
        _ => {
            anyhow::bail!(
                "Unsupported conversion: {input} -> {output}\n\
                 Supported conversions:\n\
                 - LSF <-> LSX\n\
                 - LSF <-> LSJ\n\
                 - LSX <-> LSJ\n\
                 - GR2 -> GLB/glTF\n\
                 - GLB/glTF -> GR2\n\
                 - LOCA <-> XML\n\
                 - DDS <-> PNG"
            );
        }
    }

    if !quiet {
        println!("Conversion complete");
    }
    Ok(())
}

fn convert_batch(
    sources: &[PathBuf],
    destination: &Path,
    output_format: Option<&str>,
    texture_format: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    // Ensure destination directory exists
    std::fs::create_dir_all(destination)?;

    println!("Batch converting {} files", sources.len());

    let mut success = 0;
    let mut failed = 0;

    for source in sources {
        // Determine output filename
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        // Determine output extension
        let out_ext = if let Some(fmt) = output_format {
            fmt.to_lowercase()
        } else {
            // Infer from input extension
            let in_ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
            match in_ext.to_lowercase().as_str() {
                "lsf" | "lsbc" | "lsbs" | "lsfx" => "lsx".to_string(),
                "lsx" => "lsf".to_string(),
                "lsj" => "lsx".to_string(),
                "gr2" => "glb".to_string(),
                "glb" | "gltf" => "gr2".to_string(),
                "loca" => "xml".to_string(),
                "dds" => "png".to_string(),
                "png" => "dds".to_string(),
                _ => in_ext.to_string(),
            }
        };

        let dest_file = destination.join(format!("{stem}.{out_ext}"));

        match convert_single(source, &dest_file, output_format, texture_format, true) {
            Ok(()) => {
                if !quiet {
                    println!("Converted: {}", source.display());
                }
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

/// Parse DDS format string
fn parse_dds_format(s: &str) -> anyhow::Result<crate::converter::DdsFormat> {
    use crate::converter::DdsFormat;
    match s.to_lowercase().as_str() {
        "bc1" | "dxt1" => Ok(DdsFormat::BC1),
        "bc2" | "dxt3" => Ok(DdsFormat::BC2),
        "bc3" | "dxt5" => Ok(DdsFormat::BC3),
        "rgba" | "uncompressed" => Ok(DdsFormat::Rgba),
        other => anyhow::bail!("Unknown DDS format: '{other}'. Valid options: bc1, bc2, bc3, rgba"),
    }
}
