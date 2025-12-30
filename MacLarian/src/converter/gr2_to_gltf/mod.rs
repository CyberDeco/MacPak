//! GR2 to glTF converter
//!
//! Converts Granny2 GR2 files to glTF 2.0 format.

mod gr2_reader;
mod gltf_builder;
mod utils;

pub use gr2_reader::{Gr2Reader, MeshData, Vertex, Skeleton, Bone, Transform, Gr2ContentInfo};
pub use gltf_builder::GltfBuilder;
pub use utils::{half_to_f32, decode_qtangent};

use std::path::Path;
use crate::error::{Error, Result};

/// Convert a GR2 file to glTF format (separate .gltf and .bin files).
pub fn convert_gr2_to_gltf(input_path: &Path, output_path: &Path) -> Result<()> {
    // Load and parse GR2 file
    let file_data = std::fs::read(input_path)?;
    let reader = Gr2Reader::new(&file_data)?;

    // Parse skeleton and meshes
    let skeleton = reader.parse_skeleton(&file_data)?;
    let meshes = reader.parse_meshes(&file_data)?;

    if meshes.is_empty() {
        // Get content info for better error message
        let info = reader.get_content_info(&file_data)?;
        return Err(Error::ConversionError(format!(
            "No meshes found in GR2 file (contains: {})",
            info.describe()
        )));
    }

    // Build glTF
    let mut builder = GltfBuilder::new();

    // Add skeleton first (so bone nodes come first)
    let (skin_idx, root_bone_idx) = if let Some(ref skel) = skeleton {
        let skin_idx = builder.add_skeleton(skel);
        let root_idx = skel.bones.iter()
            .position(|b| b.parent_index < 0)
            .map(|i| builder.bone_node_offset + i);
        (Some(skin_idx), root_idx)
    } else {
        (None, None)
    };

    for mesh in &meshes {
        builder.add_mesh(mesh, skin_idx);
    }

    // Export to separate .gltf and .bin files
    builder.export_gltf(output_path, root_bone_idx)?;

    Ok(())
}

/// Convert a GR2 file to GLB format.
pub fn convert_gr2_to_glb(input_path: &Path, output_path: &Path) -> Result<()> {
    // Load and parse GR2 file
    let file_data = std::fs::read(input_path)?;
    let reader = Gr2Reader::new(&file_data)?;

    // Parse skeleton and meshes
    let skeleton = reader.parse_skeleton(&file_data)?;
    let meshes = reader.parse_meshes(&file_data)?;

    if meshes.is_empty() {
        let info = reader.get_content_info(&file_data)?;
        return Err(Error::ConversionError(format!(
            "No meshes found in GR2 file (contains: {})",
            info.describe()
        )));
    }

    // Build glTF
    let mut builder = GltfBuilder::new();

    // Add skeleton first (so bone nodes come first)
    let (skin_idx, root_bone_idx) = if let Some(ref skel) = skeleton {
        let skin_idx = builder.add_skeleton(skel);
        let root_idx = skel.bones.iter()
            .position(|b| b.parent_index < 0)
            .map(|i| builder.bone_node_offset + i);
        (Some(skin_idx), root_idx)
    } else {
        (None, None)
    };

    for mesh in &meshes {
        builder.add_mesh(mesh, skin_idx);
    }

    // Export to GLB
    builder.export_glb(output_path, root_bone_idx)?;

    Ok(())
}

/// Convert GR2 data bytes to GLB data bytes.
pub fn convert_gr2_bytes_to_glb(gr2_data: &[u8]) -> Result<Vec<u8>> {
    let reader = Gr2Reader::new(gr2_data)?;
    let skeleton = reader.parse_skeleton(gr2_data)?;
    let meshes = reader.parse_meshes(gr2_data)?;

    if meshes.is_empty() {
        let info = reader.get_content_info(gr2_data)?;
        return Err(Error::ConversionError(format!(
            "No meshes found in GR2 file (contains: {})",
            info.describe()
        )));
    }

    let mut builder = GltfBuilder::new();

    let (skin_idx, root_bone_idx) = if let Some(ref skel) = skeleton {
        let skin_idx = builder.add_skeleton(skel);
        let root_idx = skel.bones.iter()
            .position(|b| b.parent_index < 0)
            .map(|i| builder.bone_node_offset + i);
        (Some(skin_idx), root_idx)
    } else {
        (None, None)
    };

    for mesh in &meshes {
        builder.add_mesh(mesh, skin_idx);
    }

    builder.build_glb(root_bone_idx)
}
