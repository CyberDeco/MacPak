//! GR2 file inspection and extraction with BitKnit decompression

use std::path::Path;
use MacPak::operations::gr2;

pub fn inspect(path: &Path) -> anyhow::Result<()> {
    println!("=== GR2 File Inspector ===\n");
    println!("File: {}\n", path.display());

    // Use high-level MacPak API
    let summary = gr2::inspect_gr2(path)?;
    let sections = gr2::get_section_info(path)?;

    println!("File size: {} bytes\n", summary.file_size);

    // Display header
    println!("=== Header Information ===");
    println!("Header size: {} bytes (0x{:x})", summary.header_size, summary.header_size);
    println!("CRC32: 0x{:08x}", summary.compression_tag);
    println!("Compression type: {}", summary.compression_type);
    println!("Section count: {}\n", summary.section_count);

    // Display sections
    println!("=== Sections ({}) ===", sections.len());

    for section in &sections {
        print!("Section {}: ", section.index);

        if section.decompressed_size == 0 {
            println!("empty");
        } else if section.is_compressed {
            println!(
                "{} → {} bytes ({:.1}:1)",
                section.compressed_size,
                section.decompressed_size,
                section.compression_ratio
            );
        } else {
            println!("{} bytes (uncompressed)", section.decompressed_size);
        }
    }

    println!();
    println!("=== Summary ===");
    println!("Total compressed: {} bytes", summary.total_compressed);
    println!("Total decompressed: {} bytes", summary.total_decompressed);
    if summary.total_compressed > 0 {
        println!("Overall ratio: {:.2}:1", summary.overall_ratio);
        println!("Space saved: {:.1}%", summary.space_saved_percent);
    }

    Ok(())
}

pub fn extract_json(path: &Path, output: &Path) -> anyhow::Result<()> {
    println!("=== GR2 to JSON Extractor ===\n");
    println!("Input: {}", path.display());
    println!("Output: {}\n", output.display());

    // Use high-level MacPak API
    gr2::extract_to_json(path, output)?;

    // Get summary for reporting
    let summary = gr2::inspect_gr2(path)?;

    println!("✓ Extracted {} sections to JSON", summary.section_count);
    println!("✓ Total decompressed: {} bytes", summary.total_decompressed);
    Ok(())
}