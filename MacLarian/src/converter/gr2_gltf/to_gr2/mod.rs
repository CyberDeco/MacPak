//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! glTF to GR2 converter
//!
//! Converts glTF 2.0 files to Granny2 GR2 format.
//!
//! Note: Compression is currently disabled/broken. GR2 files are written
//! with uncompressed data for now.

mod gltf_loader;
mod gr2_writer;
mod utils;

pub use gltf_loader::{GltfModel, Vertex, MeshData, Skeleton, Bone, Transform};
pub use gr2_writer::Gr2Writer;
pub use utils::{encode_qtangent, f32_to_half, crc32};

use std::path::Path;
use crate::error::Result;

/// Convert a glTF/GLB file to GR2 format.
///
/// # Errors
/// Returns an error if conversion fails.
pub fn convert_gltf_to_gr2(input_path: &Path, output_path: &Path) -> Result<()> {
    // Load glTF file
    let model = GltfModel::load(input_path)?;

    // Write GR2 file
    let mut writer = Gr2Writer::new();

    if let Some(ref skeleton) = model.skeleton {
        writer.add_skeleton(skeleton);
    }

    for mesh in &model.meshes {
        writer.add_mesh(mesh);
    }

    writer.write(output_path)?;

    Ok(())
}

/// Convert glTF data bytes to GR2 data bytes.
///
/// # Errors
/// Returns an error if conversion fails.
pub fn convert_gltf_bytes_to_gr2(gltf_data: &[u8]) -> Result<Vec<u8>> {
    let model = GltfModel::load_from_bytes(gltf_data)?;

    let mut writer = Gr2Writer::new();

    if let Some(ref skeleton) = model.skeleton {
        writer.add_skeleton(skeleton);
    }

    for mesh in &model.meshes {
        writer.add_mesh(mesh);
    }

    writer.build()
}
