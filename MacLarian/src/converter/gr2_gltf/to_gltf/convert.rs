//! Basic GR2 to glTF/GLB conversion functions.

use super::gltf::{
    Bg3BoneBinding, Bg3MeshProfile, Bg3SkeletonProfile, Bg3TopologyGroup, Bg3Transform, GltfBuilder,
};
use super::gr2_reader::{Gr2Reader, MeshData, MeshExtendedData, Model, Skeleton};
use crate::error::{Error, Result};
use std::path::Path;

fn bool_flag(v: bool) -> Option<bool> {
    if v { Some(true) } else { None }
}

fn to_bg3_profile(ext: &MeshExtendedData, mesh: &MeshData, idx: usize) -> Bg3MeshProfile {
    let (proxy_geometry, cloth_physics, cloth_01, cloth_02, cloth_04, impostor, lod_distance) =
        if let Some(ref props) = ext.mesh_properties {
            (
                bool_flag(props.model_flags.has_proxy_geometry),
                bool_flag(props.cloth_flags.cloth_physics),
                bool_flag(props.cloth_flags.cloth_01),
                bool_flag(props.cloth_flags.cloth_02),
                bool_flag(props.cloth_flags.cloth_04),
                bool_flag(props.is_impostor),
                if props.lod_distance < f32::MAX {
                    Some(props.lod_distance)
                } else {
                    None
                },
            )
        } else {
            (None, None, None, None, None, None, None)
        };

    let bone_bindings = if mesh.bone_bindings.is_empty() {
        None
    } else {
        Some(
            mesh.bone_bindings
                .iter()
                .map(|bb| Bg3BoneBinding {
                    bone_name: bb.bone_name.clone(),
                    obb_min: bb.obb_min,
                    obb_max: bb.obb_max,
                    tri_count: bb.tri_count,
                    tri_indices: bb.tri_indices.clone(),
                })
                .collect(),
        )
    };

    let material_bindings = if mesh.material_binding_names.is_empty() {
        None
    } else {
        Some(mesh.material_binding_names.clone())
    };

    let topology_groups = if mesh.topology_groups.is_empty() {
        None
    } else {
        Some(
            mesh.topology_groups
                .iter()
                .map(|tg| Bg3TopologyGroup {
                    material_index: tg.material_index,
                    tri_first: tg.tri_first,
                    tri_count: tg.tri_count,
                })
                .collect(),
        )
    };

    let user_defined_properties = ext.user_defined_properties.clone();

    Bg3MeshProfile {
        rigid: bool_flag(ext.rigid != 0),
        cloth: bool_flag(ext.cloth != 0),
        mesh_proxy: bool_flag(ext.mesh_proxy != 0),
        spring: bool_flag(ext.spring != 0),
        occluder: bool_flag(ext.occluder != 0),
        lod: if ext.lod != 0 { Some(ext.lod) } else { None },
        export_order: Some(idx as i32),
        proxy_geometry,
        cloth_physics,
        cloth_01,
        cloth_02,
        cloth_04,
        impostor,
        lod_distance,
        parent_bone: None,
        bone_bindings,
        material_bindings,
        topology_groups,
        user_defined_properties,
    }
}

pub(super) fn to_bg3_skeleton_profile_from(
    skel: &Skeleton,
    models: &[Model],
) -> Bg3SkeletonProfile {
    let model = models.first();
    Bg3SkeletonProfile {
        lod_type: Some(skel.lod_type),
        bone_lod_error: Some(skel.bones.iter().map(|b| b.lod_error).collect()),
        model_name: model.map(|m| m.name.clone()),
        model_mesh_bindings: model.map(|m| m.mesh_binding_names.clone()),
        initial_placement: model.map(|m| Bg3Transform {
            translation: m.initial_placement.translation,
            rotation: m.initial_placement.rotation,
            scale_shear: m.initial_placement.scale_shear,
        }),
    }
}

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
    use crate::converter::gr2_gltf::{Gr2Phase, Gr2Progress};

    progress(&Gr2Progress::with_file(
        Gr2Phase::ReadingFile,
        1,
        5,
        input_path.display().to_string(),
    ));
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

    progress(&Gr2Progress::with_file(
        Gr2Phase::BuildingDocument,
        4,
        5,
        format!("{} meshes", meshes.len()),
    ));
    let mut builder = GltfBuilder::new();

    let (skin_idx, root_bone_idx) = if let Some(ref skel) = skeleton {
        let models = reader.parse_models(&file_data).unwrap_or_default();
        let skel_profile = to_bg3_skeleton_profile_from(skel, &models);
        let skin_idx = builder.add_skeleton_with_profile(skel, skel_profile);
        let root_idx = skel
            .bones
            .iter()
            .position(|b| b.parent_index < 0)
            .map(|i| builder.bone_node_offset + i);
        (Some(skin_idx), root_idx)
    } else {
        (None, None)
    };

    for (i, mesh) in meshes.iter().enumerate() {
        let profile = mesh
            .extended_data
            .as_ref()
            .map(|ext| to_bg3_profile(ext, mesh, i));
        builder.add_mesh_with_profile(mesh, skin_idx, profile);
    }

    progress(&Gr2Progress::with_file(
        Gr2Phase::WritingOutput,
        5,
        5,
        output_path.display().to_string(),
    ));
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
    use crate::converter::gr2_gltf::{Gr2Phase, Gr2Progress};

    progress(&Gr2Progress::with_file(
        Gr2Phase::ReadingFile,
        1,
        5,
        input_path.display().to_string(),
    ));
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

    progress(&Gr2Progress::with_file(
        Gr2Phase::BuildingDocument,
        4,
        5,
        format!("{} meshes", meshes.len()),
    ));
    let mut builder = GltfBuilder::new();

    let (skin_idx, root_bone_idx) = if let Some(ref skel) = skeleton {
        let models = reader.parse_models(&file_data).unwrap_or_default();
        let skel_profile = to_bg3_skeleton_profile_from(skel, &models);
        let skin_idx = builder.add_skeleton_with_profile(skel, skel_profile);
        let root_idx = skel
            .bones
            .iter()
            .position(|b| b.parent_index < 0)
            .map(|i| builder.bone_node_offset + i);
        (Some(skin_idx), root_idx)
    } else {
        (None, None)
    };

    for (i, mesh) in meshes.iter().enumerate() {
        let profile = mesh
            .extended_data
            .as_ref()
            .map(|ext| to_bg3_profile(ext, mesh, i));
        builder.add_mesh_with_profile(mesh, skin_idx, profile);
    }

    progress(&Gr2Progress::with_file(
        Gr2Phase::WritingOutput,
        5,
        5,
        output_path.display().to_string(),
    ));
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
    use crate::converter::gr2_gltf::{Gr2Phase, Gr2Progress};

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

    progress(&Gr2Progress::with_file(
        Gr2Phase::BuildingDocument,
        4,
        5,
        format!("{} meshes", meshes.len()),
    ));
    let mut builder = GltfBuilder::new();

    let (skin_idx, root_bone_idx) = if let Some(ref skel) = skeleton {
        let models = reader.parse_models(gr2_data).unwrap_or_default();
        let skel_profile = to_bg3_skeleton_profile_from(skel, &models);
        let skin_idx = builder.add_skeleton_with_profile(skel, skel_profile);
        let root_idx = skel
            .bones
            .iter()
            .position(|b| b.parent_index < 0)
            .map(|i| builder.bone_node_offset + i);
        (Some(skin_idx), root_idx)
    } else {
        (None, None)
    };

    for (i, mesh) in meshes.iter().enumerate() {
        let profile = mesh
            .extended_data
            .as_ref()
            .map(|ext| to_bg3_profile(ext, mesh, i));
        builder.add_mesh_with_profile(mesh, skin_idx, profile);
    }

    progress(&Gr2Progress::new(Gr2Phase::WritingOutput, 5, 5));
    let result = builder.build_glb(root_bone_idx)?;

    progress(&Gr2Progress::new(Gr2Phase::Complete, 5, 5));
    Ok(result)
}
