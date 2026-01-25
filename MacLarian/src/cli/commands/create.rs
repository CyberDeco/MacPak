use std::path::Path;

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

    PakOperations::create_with_compression(source, destination, method)?;
    println!("PAK created successfully");
    Ok(())
}
