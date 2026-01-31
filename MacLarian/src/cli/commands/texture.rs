//! CLI commands for texture operations

use std::path::Path;

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
