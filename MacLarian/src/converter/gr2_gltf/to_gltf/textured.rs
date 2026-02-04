//! Textured GLB conversion with embedded textures.

use super::gltf::GltfBuilder;
use super::gr2_reader::Gr2Reader;
use super::texture_loading::load_textures_for_gr2;
use crate::error::{Error, Result};
use std::path::Path;

/// Result of textured GLB conversion.
#[derive(Debug)]
pub struct TexturedGlbResult {
    /// The GLB binary data.
    pub glb_data: Vec<u8>,
    /// Warnings encountered during texture loading.
    pub warnings: Vec<String>,
}

/// Convert GR2 data bytes to GLB with embedded textures.
///
/// This function looks up textures in the embedded database and extracts them
/// from the provided PAK files to embed in the GLB.
///
/// # Arguments
/// * `gr2_data` - The raw GR2 file bytes
/// * `gr2_filename` - The GR2 filename (used for database lookup, e.g., "`HUM_M_ARM_Shirt.GR2`")
/// * `textures_pak_path` - Path to Textures.pak (also looks for sibling VirtualTextures.pak)
///
/// # Returns
/// A `TexturedGlbResult` with the GLB data and any warnings
///
/// # Errors
/// Returns an error if the GR2 data cannot be parsed or has no meshes.
pub fn convert_gr2_bytes_to_glb_with_textures(
    gr2_data: &[u8],
    gr2_filename: &str,
    textures_pak_path: &Path,
) -> Result<TexturedGlbResult> {
    let mut warnings = Vec::new();

    // Parse GR2
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

    // Add skeleton first
    let (skin_idx, root_bone_idx) = if let Some(ref skel) = skeleton {
        let models = reader.parse_models(gr2_data).unwrap_or_default();
        let skel_profile = super::convert::to_bg3_skeleton_profile_from(skel, &models);
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

    // Try to load textures
    let material_idx =
        load_textures_for_gr2(gr2_filename, textures_pak_path, &mut builder, &mut warnings);

    // Add meshes with material (if present)
    for mesh in &meshes {
        builder.add_mesh_with_material(mesh, skin_idx, material_idx);
    }

    let glb_data = builder.build_glb(root_bone_idx)?;

    Ok(TexturedGlbResult { glb_data, warnings })
}
