//! CLI interface for format conversion
use std::path::Path;

use crate::cli::progress::simple_spinner;

pub fn execute(
    source: &Path,
    destination: &Path,
    input_format: Option<&str>,
    output_format: Option<&str>,
) -> anyhow::Result<()> {
    println!("Converting {} to {}", source.display(), destination.display());

    // Auto-detect or use provided formats
    let input = if let Some(fmt) = input_format {
        fmt.to_lowercase()
    } else {
        source
            .extension()
            .and_then(|s| s.to_str())
            .map(str::to_lowercase)
            .ok_or_else(|| {
                anyhow::anyhow!("Cannot detect input format from source file extension")
            })?
    };

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
            let pb = simple_spinner("Converting LSF -> LSX...");
            crate::converter::lsf_to_lsx_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }
        ("lsf" | "lsbc" | "lsbs" | "lsfx", "lsj") => {
            let pb = simple_spinner("Converting LSF -> LSJ...");
            crate::converter::lsf_to_lsj_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }

        // LSX conversions
        ("lsx", "lsf" | "lsbc" | "lsbs" | "lsfx") => {
            let pb = simple_spinner("Converting LSX -> LSF...");
            crate::converter::lsx_to_lsf_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }
        ("lsx", "lsj") => {
            let pb = simple_spinner("Converting LSX -> LSJ...");
            crate::converter::lsx_to_lsj_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }

        // LSJ conversions
        ("lsj", "lsx") => {
            let pb = simple_spinner("Converting LSJ -> LSX...");
            crate::converter::lsj_to_lsx_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }
        ("lsj", "lsf" | "lsbc" | "lsbs" | "lsfx") => {
            let pb = simple_spinner("Converting LSJ -> LSF...");
            crate::converter::lsj_to_lsf_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }

        // GR2/glTF conversions
        ("gr2", "glb") => {
            let pb = simple_spinner("Converting GR2 -> GLB...");
            crate::converter::convert_gr2_to_glb_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }
        ("gr2", "gltf") => {
            let pb = simple_spinner("Converting GR2 -> glTF...");
            crate::converter::convert_gr2_to_gltf_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }
        ("glb" | "gltf", "gr2") => {
            println!("Note: Output will be uncompressed (compression not yet implemented)");
            let pb = simple_spinner("Converting glTF -> GR2...");
            crate::converter::convert_gltf_to_gr2_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }

        // LOCA conversions
        ("loca", "xml") => {
            let pb = simple_spinner("Converting LOCA -> XML...");
            crate::converter::convert_loca_to_xml_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }
        ("xml", "loca") => {
            let pb = simple_spinner("Converting XML -> LOCA...");
            crate::converter::convert_xml_to_loca_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }

        // DDS/PNG conversions
        ("dds", "png") => {
            let pb = simple_spinner("Converting DDS -> PNG...");
            crate::converter::convert_dds_to_png_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }
        ("png", "dds") => {
            let pb = simple_spinner("Converting PNG -> DDS (BC3)...");
            crate::converter::convert_png_to_dds_with_progress(source, destination, &|p| {
                if let Some(ref msg) = p.current_file {
                    pb.set_message(msg.clone());
                }
            })?;
            pb.finish_and_clear();
        }

        // Same format (copy)
        (fmt1, fmt2) if fmt1 == fmt2 => {
            println!("Source and destination formats are the same, copying file...");
            std::fs::copy(source, destination)?;
        }

        // Unsupported
        _ => {
            anyhow::bail!(
                "Unsupported conversion: {input} -> {output}\n\
                 Supported conversions:\n\
                 - LSF <-> LSX\n\
                 - LSF <-> LSJ (via intermediary LSX)\n\
                 - LSX <-> LSJ\n\
                 - GR2 -> GLB/glTF\n\
                 - GLB/glTF -> GR2\n\
                 - LOCA <-> XML\n\
                 - DDS <-> PNG"
            );
        }
    }

    println!("Conversion complete");
    Ok(())
}
