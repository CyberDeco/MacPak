//! CLI interface for format conversion
use std::path::Path;

pub fn execute(
    source: &Path,
    destination: &Path,
    input_format: Option<&str>,
    output_format: Option<&str>
) -> anyhow::Result<()> {
    println!("Converting {:?} to {:?}", source, destination);

    // Auto-detect or use provided formats
    let input = if let Some(fmt) = input_format {
        fmt.to_lowercase()
    } else {
        source.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Cannot detect input format from source file extension"))?
    };

    let output = if let Some(fmt) = output_format {
        fmt.to_lowercase()
    } else {
        destination.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Cannot detect output format from destination file extension"))?
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
            println!("Converting LSF -> LSX");
            crate::converter::lsf_to_lsx(source, destination)?;
        }
        ("lsf" | "lsbc" | "lsbs" | "lsfx", "lsj") => {
            println!("Converting LSF -> LSJ");
            crate::converter::lsf_to_lsj(source, destination)?;
        }

        // LSX conversions
        ("lsx", "lsf" | "lsbc" | "lsbs" | "lsfx", ) => {
            println!("Converting LSX -> LSF");
            crate::converter::lsx_to_lsf(source, destination)?;
        }
        ("lsx", "lsj") => {
            println!("Converting LSX -> LSJ");
            crate::converter::lsx_to_lsj(source, destination)?;
        }

        // LSJ conversions
        ("lsj", "lsx") => {
            println!("Converting LSJ -> LSX");
            crate::converter::lsj_to_lsx(source, destination)?;
        }
        ("lsj", "lsf" | "lsbc" | "lsbs" | "lsfx") => {
            println!("Converting LSJ -> LSF");
            crate::converter::lsj_to_lsf(source, destination)?;
        }

        // GR2/glTF conversions
        ("gr2", "glb") => {
            println!("Converting GR2 -> GLB");
            crate::converter::convert_gr2_to_glb(source, destination)?;
        }
        ("gr2", "gltf") => {
            println!("Converting GR2 -> glTF");
            crate::converter::convert_gr2_to_gltf(source, destination)?;
        }
        ("glb" | "gltf", "gr2") => {
            println!("Converting glTF -> GR2");
            println!("Note: Output will be uncompressed (compression not yet implemented)");
            crate::converter::convert_gltf_to_gr2(source, destination)?;
        }

        // LOCA conversions
        ("loca", "xml") => {
            println!("Converting LOCA -> XML");
            crate::converter::convert_loca_to_xml(source, destination)?;
        }
        ("xml", "loca") => {
            println!("Converting XML -> LOCA");
            crate::converter::convert_xml_to_loca(source, destination)?;
        }

        // DDS/PNG conversions
        ("dds", "png") => {
            println!("Converting DDS -> PNG");
            crate::converter::convert_dds_to_png(source, destination)?;
        }
        ("png", "dds") => {
            println!("Converting PNG -> DDS (BC3 compression)");
            crate::converter::convert_png_to_dds(source, destination)?;
        }

        // Same format (copy)
        (fmt1, fmt2) if fmt1 == fmt2 => {
            println!("Source and destination formats are the same, copying file...");
            std::fs::copy(source, destination)?;
        }

        // Unsupported
        _ => {
            anyhow::bail!(
                "Unsupported conversion: {} -> {}\n\
                 Supported conversions:\n\
                 - LSF <-> LSX\n\
                 - LSF <-> LSJ (via intermediary LSX)\n\
                 - LSX <-> LSJ\n\
                 - GR2 -> GLB/glTF\n\
                 - GLB/glTF -> GR2\n\
                 - LOCA <-> XML\n\
                 - DDS <-> PNG",
                input, output
            );
        }
    }

    println!("Conversion complete");
    Ok(())
}
