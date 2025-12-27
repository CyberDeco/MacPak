//! GR2 file inspection, extraction, and decompression

use std::path::Path;
use MacPak::operations::gr2;

pub fn inspect(path: &Path) -> anyhow::Result<()> {
    println!("=== GR2 File Inspector ===\n");
    println!("File: {}\n", path.display());

    let file_size = std::fs::metadata(path)?.len();
    let (compression_type, sections) = gr2::get_file_info(path)?;

    println!("File size: {} bytes\n", file_size);

    // Display header
    println!("=== Header Information ===");
    println!("Compression type: {} ({})", compression_type,
        if compression_type == 4 { "BitKnit" } else { "Unknown" });
    println!("Section count: {}\n", sections.len());

    // Display sections
    println!("=== Sections ({}) ===", sections.len());

    let mut total_compressed = 0u64;
    let mut total_decompressed = 0u64;

    for (i, section) in sections.iter().enumerate() {
        print!("Section {}: ", i);

        if section.decompressed_size == 0 {
            println!("empty");
        } else if section.compressed_size != section.decompressed_size {
            let ratio = section.decompressed_size as f64 / section.compressed_size as f64;
            println!(
                "{} -> {} bytes ({:.1}:1)",
                section.compressed_size,
                section.decompressed_size,
                ratio
            );
            total_compressed += section.compressed_size as u64;
            total_decompressed += section.decompressed_size as u64;
        } else {
            println!("{} bytes (uncompressed)", section.decompressed_size);
            total_compressed += section.decompressed_size as u64;
            total_decompressed += section.decompressed_size as u64;
        }
    }

    println!();
    println!("=== Summary ===");
    println!("Total compressed: {} bytes", total_compressed);
    println!("Total decompressed: {} bytes", total_decompressed);
    if total_compressed > 0 {
        let ratio = total_decompressed as f64 / total_compressed as f64;
        let saved = (1.0 - (total_compressed as f64 / total_decompressed as f64)) * 100.0;
        println!("Overall ratio: {:.2}:1", ratio);
        println!("Space saved: {:.1}%", saved);
    }

    Ok(())
}

pub fn extract_json(path: &Path, output: &Path) -> anyhow::Result<()> {
    println!("=== GR2 to JSON Extractor ===\n");
    println!("Input: {}", path.display());
    println!("Output: {}\n", output.display());

    anyhow::bail!("JSON extraction not yet implemented - use 'decompress' command first")
}

pub fn decompress(path: &Path, output: Option<&Path>) -> anyhow::Result<()> {
    println!("=== GR2 Decompressor ===\n");
    println!("Input: {}", path.display());

    // Generate output path if not provided
    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let ext = path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("gr2");
            path.parent()
                .unwrap_or(Path::new("."))
                .join(format!("{}_decompressed.{}", stem, ext))
        }
    };

    println!("Output: {}\n", output_path.display());

    // Get info before decompression
    let (compression_type, sections) = gr2::get_file_info(path)?;

    let compressed_sections = sections.iter()
        .filter(|s| s.compressed_size != s.decompressed_size && s.decompressed_size > 0)
        .count();

    if compressed_sections == 0 {
        println!("File has no compressed sections.");
        return Ok(());
    }

    println!("Found {} compressed sections (BitKnit)", compressed_sections);

    // Decompress
    gr2::decompress_file(path, &output_path)?;

    let output_size = std::fs::metadata(&output_path)?.len();
    let input_size = std::fs::metadata(path)?.len();

    println!("\nDecompression complete!");
    println!("  Input:  {} bytes", input_size);
    println!("  Output: {} bytes", output_size);

    Ok(())
}