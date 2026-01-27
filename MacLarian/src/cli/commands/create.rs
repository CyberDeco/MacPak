use std::path::Path;

use crate::cli::progress::simple_bar;
use crate::pak::{CompressionMethod, PakOperations};

pub fn execute(source: &Path, destination: &Path, compression: &str) -> anyhow::Result<()> {
    let method = match compression.to_lowercase().as_str() {
        "lz4" => CompressionMethod::Lz4,
        "zlib" => CompressionMethod::Zlib,
        "none" => CompressionMethod::None,
        other => {
            anyhow::bail!(
                "Unknown compression method: '{}'. Valid options: lz4, zlib, none",
                other
            );
        }
    };

    println!(
        "Creating PAK from {} to {} (compression: {:?})",
        source.display(),
        destination.display(),
        method
    );

    let pb = simple_bar(100, "Creating PAK");
    PakOperations::create_with_compression_and_progress(source, destination, method, &|p| {
        pb.set_position((p.percentage() * 100.0) as u64);
        if let Some(ref file) = p.current_file {
            pb.set_message(file.clone());
        }
    })?;
    pb.finish_and_clear();

    println!("PAK created successfully");
    Ok(())
}
