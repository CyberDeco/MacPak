//! Basic GR2 to glTF/GLB conversion functions.

use std::path::Path;
use crate::error::{Error, Result};
use super::gr2_reader::Gr2Reader;
use super::gltf::GltfBuilder;

/// Convert a GR2 file to glTF format (separate .gltf and .bin files).
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_gr2_to_gltf(input_path: &Path, output_path: &Path) -> Result<()> {
    convert_gr2_to_gltf_with_progress(input_path, output_path, &|_| {})
}

/// Convert a GR2 file to glTF format with progress callback.
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_gr2_to_gltf_with_progress(
    input_path: &Path,
    output_path: &Path,
    progress: crate::converter::gr2_gltf::Gr2ProgressCallback,
) -> Result<()> {
    use crate::converter::gr2_gltf::{Gr2Progress, Gr2Phase};

    progress(&Gr2Progress::with_file(Gr2Phase::ReadingFile, 1, 5, input_path.display().to_string()));
    let file_data = std::fs::read(input_path)?;
    let reader = Gr2Reader::new(&file_data)?;

    progress(&Gr2Progress::new(Gr2Phase::ParsingSkeleton, 2, 5));
    let skeleton = reader.parse_skeleton(&file_data)?;

    progress(&Gr2Progress::new(Gr2Phase::ParsingMeshes, 3, 5));
    let meshes = reader.parse_meshes(&file_data)?;

    if meshes.is_empty() {
        let info = reader.get_content_info(&file_data)?;
        return Err(Error::ConversionError(format!(
            "No meshes found in GR2 file (contains: {})",
            info.describe()
        )));
    }

    progress(&Gr2Progress::with_file(Gr2Phase::BuildingDocument, 4, 5, format!("{} meshes", meshes.len())));
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

    progress(&Gr2Progress::with_file(Gr2Phase::WritingOutput, 5, 5, output_path.display().to_string()));
    builder.export_gltf(output_path, root_bone_idx)?;

    progress(&Gr2Progress::new(Gr2Phase::Complete, 5, 5));
    Ok(())
}

/// Convert a GR2 file to GLB format.
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_gr2_to_glb(input_path: &Path, output_path: &Path) -> Result<()> {
    convert_gr2_to_glb_with_progress(input_path, output_path, &|_| {})
}

/// Convert a GR2 file to GLB format with progress callback.
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_gr2_to_glb_with_progress(
    input_path: &Path,
    output_path: &Path,
    progress: crate::converter::gr2_gltf::Gr2ProgressCallback,
) -> Result<()> {
    use crate::converter::gr2_gltf::{Gr2Progress, Gr2Phase};

    progress(&Gr2Progress::with_file(Gr2Phase::ReadingFile, 1, 5, input_path.display().to_string()));
    let file_data = std::fs::read(input_path)?;
    let reader = Gr2Reader::new(&file_data)?;

    progress(&Gr2Progress::new(Gr2Phase::ParsingSkeleton, 2, 5));
    let skeleton = reader.parse_skeleton(&file_data)?;

    progress(&Gr2Progress::new(Gr2Phase::ParsingMeshes, 3, 5));
    let meshes = reader.parse_meshes(&file_data)?;

    if meshes.is_empty() {
        let info = reader.get_content_info(&file_data)?;
        return Err(Error::ConversionError(format!(
            "No meshes found in GR2 file (contains: {})",
            info.describe()
        )));
    }

    progress(&Gr2Progress::with_file(Gr2Phase::BuildingDocument, 4, 5, format!("{} meshes", meshes.len())));
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

    progress(&Gr2Progress::with_file(Gr2Phase::WritingOutput, 5, 5, output_path.display().to_string()));
    builder.export_glb(output_path, root_bone_idx)?;

    progress(&Gr2Progress::new(Gr2Phase::Complete, 5, 5));
    Ok(())
}

/// Convert GR2 data bytes to GLB data bytes.
///
/// # Errors
/// Returns an error if the data cannot be parsed or conversion fails.
pub fn convert_gr2_bytes_to_glb(gr2_data: &[u8]) -> Result<Vec<u8>> {
    convert_gr2_bytes_to_glb_with_progress(gr2_data, &|_| {})
}

/// Convert GR2 data bytes to GLB data bytes with progress callback.
///
/// # Errors
/// Returns an error if the data cannot be parsed or conversion fails.
pub fn convert_gr2_bytes_to_glb_with_progress(
    gr2_data: &[u8],
    progress: crate::converter::gr2_gltf::Gr2ProgressCallback,
) -> Result<Vec<u8>> {
    use crate::converter::gr2_gltf::{Gr2Progress, Gr2Phase};

    progress(&Gr2Progress::new(Gr2Phase::ReadingFile, 1, 5));
    let reader = Gr2Reader::new(gr2_data)?;

    progress(&Gr2Progress::new(Gr2Phase::ParsingSkeleton, 2, 5));
    let skeleton = reader.parse_skeleton(gr2_data)?;

    progress(&Gr2Progress::new(Gr2Phase::ParsingMeshes, 3, 5));
    let meshes = reader.parse_meshes(gr2_data)?;

    if meshes.is_empty() {
        let info = reader.get_content_info(gr2_data)?;
        return Err(Error::ConversionError(format!(
            "No meshes found in GR2 file (contains: {})",
            info.describe()
        )));
    }

    progress(&Gr2Progress::with_file(Gr2Phase::BuildingDocument, 4, 5, format!("{} meshes", meshes.len())));
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

    progress(&Gr2Progress::new(Gr2Phase::WritingOutput, 5, 5));
    let result = builder.build_glb(root_bone_idx)?;

    progress(&Gr2Progress::new(Gr2Phase::Complete, 5, 5));
    Ok(result)
}
