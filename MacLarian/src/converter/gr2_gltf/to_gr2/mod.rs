//! glTF to GR2 converter
//!
//! Converts glTF 2.0 files to Granny2 GR2 format.
//!
//! Note: Compression is currently disabled/broken. GR2 files are written
//! with uncompressed data for now.
//!
//!

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::needless_pass_by_value
)]

mod gltf_loader;
mod gr2_writer;
mod utils;

use crate::error::Result;
use gltf_loader::GltfModel;
use gr2_writer::Gr2Writer;
use std::path::Path;

/// Convert a glTF/GLB file to GR2 format.
///
/// # Errors
/// Returns an error if conversion fails.
pub fn convert_gltf_to_gr2(input_path: &Path, output_path: &Path) -> Result<()> {
    convert_gltf_to_gr2_with_progress(input_path, output_path, &|_| {})
}

/// Convert a glTF/GLB file to GR2 format with progress callback.
///
/// # Errors
/// Returns an error if conversion fails.
pub fn convert_gltf_to_gr2_with_progress(
    input_path: &Path,
    output_path: &Path,
    progress: crate::converter::gr2_gltf::Gr2ProgressCallback,
) -> Result<()> {
    use crate::converter::gr2_gltf::{Gr2Phase, Gr2Progress};

    progress(&Gr2Progress::with_file(
        Gr2Phase::LoadingFile,
        1,
        4,
        input_path.display().to_string(),
    ));
    let model = GltfModel::load(input_path)?;

    progress(&Gr2Progress::with_file(
        Gr2Phase::BuildingGr2,
        2,
        4,
        format!("{} meshes", model.meshes.len()),
    ));
    let mut writer = Gr2Writer::new();

    if let Some(ref skeleton) = model.skeleton {
        writer.add_skeleton(skeleton);
    }

    if let Some(ref model_data) = model.model {
        writer.set_model(model_data);
    }

    for mesh in &model.meshes {
        writer.add_mesh(mesh);
    }

    progress(&Gr2Progress::with_file(
        Gr2Phase::WritingFile,
        3,
        4,
        output_path.display().to_string(),
    ));
    writer.write(output_path)?;

    progress(&Gr2Progress::new(Gr2Phase::Complete, 4, 4));
    Ok(())
}

/// Convert glTF data bytes to GR2 data bytes.
///
/// # Errors
/// Returns an error if conversion fails.
pub fn convert_gltf_bytes_to_gr2(gltf_data: &[u8]) -> Result<Vec<u8>> {
    convert_gltf_bytes_to_gr2_with_progress(gltf_data, &|_| {})
}

/// Convert glTF data bytes to GR2 data bytes with progress callback.
///
/// # Errors
/// Returns an error if conversion fails.
pub fn convert_gltf_bytes_to_gr2_with_progress(
    gltf_data: &[u8],
    progress: crate::converter::gr2_gltf::Gr2ProgressCallback,
) -> Result<Vec<u8>> {
    use crate::converter::gr2_gltf::{Gr2Phase, Gr2Progress};

    progress(&Gr2Progress::new(Gr2Phase::LoadingFile, 1, 4));
    let model = GltfModel::load_from_bytes(gltf_data)?;

    progress(&Gr2Progress::with_file(
        Gr2Phase::BuildingGr2,
        2,
        4,
        format!("{} meshes", model.meshes.len()),
    ));
    let mut writer = Gr2Writer::new();

    if let Some(ref skeleton) = model.skeleton {
        writer.add_skeleton(skeleton);
    }

    if let Some(ref model_data) = model.model {
        writer.set_model(model_data);
    }

    for mesh in &model.meshes {
        writer.add_mesh(mesh);
    }

    progress(&Gr2Progress::new(Gr2Phase::WritingFile, 3, 4));
    let result = writer.build()?;

    progress(&Gr2Progress::new(Gr2Phase::Complete, 4, 4));
    Ok(result)
}
