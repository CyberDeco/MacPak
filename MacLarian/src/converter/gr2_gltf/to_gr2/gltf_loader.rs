//! glTF file loader.
//!
//! Loads glTF/GLB files and converts them to intermediate structures
//! suitable for GR2 export.
//!
//!

#![allow(clippy::never_loop, clippy::needless_range_loop)]

use std::collections::HashMap;
use std::path::Path;

use super::utils::encode_qtangent;
use crate::converter::gr2_gltf::to_gltf::{Bg3MeshProfile, Bg3SkeletonProfile};
use crate::error::{Error, Result};

// ============================================================================
// Data Structures
// ============================================================================

/// Vertex data for a mesh.
#[derive(Debug, Clone, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub bone_weights: [u8; 4],
    pub bone_indices: [u8; 4],
    pub qtangent: [i16; 4],
    pub color: [u8; 4],
    pub uv: [f32; 2],
}

/// Bone binding data extracted from glTF.
#[derive(Debug, Clone)]
pub struct BoneBindingData {
    pub bone_name: String,
    pub obb_min: [f32; 3],
    pub obb_max: [f32; 3],
    pub tri_count: i32,
    pub tri_indices: Vec<i32>,
}

/// Topology group data extracted from glTF.
#[derive(Debug, Clone)]
pub struct TopologyGroupData {
    pub material_index: i32,
    pub tri_first: i32,
    pub tri_count: i32,
}

/// Model data extracted from glTF.
#[derive(Debug, Clone)]
pub struct ModelData {
    pub name: String,
    pub mesh_binding_names: Vec<String>,
    pub initial_placement: Transform,
}

/// Mesh data extracted from glTF.
#[derive(Clone)]
pub struct MeshData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bg3_profile: Option<Bg3MeshProfile>,
    pub bone_bindings: Vec<BoneBindingData>,
    pub material_binding_names: Vec<String>,
    pub topology_groups: Vec<TopologyGroupData>,
    pub user_defined_properties: Option<String>,
}

/// Transform data for bones.
#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale_shear: [f32; 9],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale_shear: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Bone data.
#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub parent_index: i32,
    pub transform: Transform,
    pub inverse_world_transform: [f32; 16],
    pub lod_error: f32,
}

/// Skeleton data.
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<Bone>,
    pub lod_type: i32,
}

/// Complete glTF model.
pub struct GltfModel {
    pub meshes: Vec<MeshData>,
    pub skeleton: Option<Skeleton>,
    pub model: Option<ModelData>,
}

// ============================================================================
// Loading
// ============================================================================

impl GltfModel {
    /// Load a glTF or GLB file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or has no meshes.
    pub fn load(path: &Path) -> Result<Self> {
        let (document, buffers, _images) = gltf::import(path)
            .map_err(|e| Error::ConversionError(format!("Failed to load glTF: {e}")))?;

        Self::load_from_document(&document, &buffers)
    }

    /// Load from GLB bytes.
    ///
    /// # Errors
    /// Returns an error if the data cannot be parsed or has no meshes.
    pub fn load_from_bytes(data: &[u8]) -> Result<Self> {
        let (document, buffers, _images) = gltf::import_slice(data)
            .map_err(|e| Error::ConversionError(format!("Failed to load glTF: {e}")))?;

        Self::load_from_document(&document, &buffers)
    }

    fn load_from_document(
        document: &gltf::Document,
        buffers: &[gltf::buffer::Data],
    ) -> Result<Self> {
        let mut meshes = Vec::new();
        let mut skeleton = None;
        let mut model = None;

        // First pass: find skeleton from skins
        for skin in document.skins() {
            let (skel, skel_profile) = load_skeleton_with_profile(document, &skin, buffers)?;
            skeleton = Some(skel);

            // Extract model data from skin profile
            if let Some(ref profile) = skel_profile {
                if let Some(ref name) = profile.model_name {
                    model = Some(ModelData {
                        name: name.clone(),
                        mesh_binding_names: profile.model_mesh_bindings.clone().unwrap_or_default(),
                        initial_placement: profile
                            .initial_placement
                            .as_ref()
                            .map(|p| Transform {
                                translation: p.translation,
                                rotation: p.rotation,
                                scale_shear: p.scale_shear,
                            })
                            .unwrap_or_default(),
                    });
                }
            }
            break; // Only support one skeleton for now
        }

        // Second pass: load meshes
        for node in document.nodes() {
            if let Some(mesh) = node.mesh() {
                let node_name = node.name().unwrap_or("Mesh");
                let bg3_profile = parse_bg3_profile(&mesh);

                for (prim_idx, primitive) in mesh.primitives().enumerate() {
                    let name = if mesh.primitives().len() > 1 {
                        format!("{}_{}", mesh.name().unwrap_or(node_name), prim_idx)
                    } else {
                        mesh.name().unwrap_or(node_name).to_string()
                    };

                    if let Some(mut mesh_data) = load_primitive(&primitive, buffers, &name)? {
                        // Extract extension data before overwriting the profile
                        if let Some(ref profile) = bg3_profile {
                            extract_mesh_extension_data(&mut mesh_data, profile);
                        }
                        mesh_data.bg3_profile.clone_from(&bg3_profile);
                        meshes.push(mesh_data);
                    }
                }
            }
        }

        if meshes.is_empty() {
            return Err(Error::ConversionError(
                "No meshes found in glTF file".to_string(),
            ));
        }

        // Remap vertex bone indices from skeleton-global (glTF) to
        // mesh-local BoneBindings indices (GR2)
        if let Some(ref skel) = skeleton {
            for mesh in &mut meshes {
                remap_joint_indices_to_bone_bindings(mesh, skel);
            }
        }

        Ok(GltfModel {
            meshes,
            skeleton,
            model,
        })
    }
}

/// Load skeleton from a glTF skin, returning the skeleton and optional skin profile.
fn load_skeleton_with_profile(
    document: &gltf::Document,
    skin: &gltf::Skin,
    buffers: &[gltf::buffer::Data],
) -> Result<(Skeleton, Option<Bg3SkeletonProfile>)> {
    let name = skin.name().unwrap_or("Skeleton").to_string();
    let joints: Vec<_> = skin.joints().collect();

    // Parse skeleton profile from skin extensions
    let skel_profile = parse_bg3_skeleton_profile(skin);

    // Build parent index map
    let joint_indices: HashMap<usize, usize> = joints
        .iter()
        .enumerate()
        .map(|(i, j)| (j.index(), i))
        .collect();

    // Read inverse bind matrices if available
    let ibm: Vec<[f32; 16]> = if let Some(accessor) = skin.inverse_bind_matrices() {
        read_accessor_mat4(&accessor, buffers)?
    } else {
        // Identity matrices as fallback
        vec![
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ];
            joints.len()
        ]
    };

    // Extract per-bone LOD errors from the profile
    let bone_lod_errors = skel_profile
        .as_ref()
        .and_then(|p| p.bone_lod_error.as_ref());

    let lod_type = skel_profile.as_ref().and_then(|p| p.lod_type).unwrap_or(0);

    let mut bones = Vec::with_capacity(joints.len());

    for (bone_idx, joint) in joints.iter().enumerate() {
        let bone_name = joint
            .name()
            .unwrap_or(&format!("Bone_{bone_idx}"))
            .to_string();

        // Find parent index
        let parent_index = find_parent_bone_index(document, joint, &joint_indices);

        // Get local transform
        let (translation, rotation, scale) = joint.transform().decomposed();

        // Undo X-axis reflection applied during GR2→glTF export.
        // The reflection is its own inverse: applying S*M*S twice = identity.
        let transform = Transform {
            translation: [-translation[0], translation[1], translation[2]],
            rotation: [rotation[0], -rotation[1], -rotation[2], rotation[3]],
            scale_shear: [scale[0], 0.0, 0.0, 0.0, scale[1], 0.0, 0.0, 0.0, scale[2]],
        };

        // Undo X-axis reflection on inverse bind matrices (negate indices 1,2,3,4,8,12).
        let mut inverse_world_transform = ibm.get(bone_idx).copied().unwrap_or([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]);
        inverse_world_transform[1] = -inverse_world_transform[1];
        inverse_world_transform[2] = -inverse_world_transform[2];
        inverse_world_transform[3] = -inverse_world_transform[3];
        inverse_world_transform[4] = -inverse_world_transform[4];
        inverse_world_transform[8] = -inverse_world_transform[8];
        inverse_world_transform[12] = -inverse_world_transform[12];

        let lod_error = bone_lod_errors
            .and_then(|errors| errors.get(bone_idx).copied())
            .unwrap_or(0.0);

        bones.push(Bone {
            name: bone_name,
            parent_index,
            transform,
            inverse_world_transform,
            lod_error,
        });
    }

    Ok((
        Skeleton {
            name,
            bones,
            lod_type,
        },
        skel_profile,
    ))
}

/// Find the parent bone index for a joint.
fn find_parent_bone_index(
    document: &gltf::Document,
    joint: &gltf::Node,
    joint_indices: &HashMap<usize, usize>,
) -> i32 {
    // Search all nodes to find which one has this joint as a child
    for node in document.nodes() {
        for child in node.children() {
            if child.index() == joint.index() {
                // Found parent node, check if it's a joint
                if let Some(&bone_idx) = joint_indices.get(&node.index()) {
                    return bone_idx as i32;
                }
            }
        }
    }
    -1 // Root bone
}

/// Parse the `MACLARIAN_glTF_extensions` extension from a glTF mesh.
fn parse_bg3_profile(mesh: &gltf::Mesh) -> Option<Bg3MeshProfile> {
    let ext_map = mesh.extensions()?;
    let profile_json = ext_map.get("MACLARIAN_glTF_extensions")?;
    serde_json::from_value(profile_json.clone()).ok()
}

/// Parse the `MACLARIAN_glTF_extensions` extension from a glTF skin.
fn parse_bg3_skeleton_profile(skin: &gltf::Skin) -> Option<Bg3SkeletonProfile> {
    let ext_map = skin.extensions()?;
    let profile_json = ext_map.get("MACLARIAN_glTF_extensions")?;
    serde_json::from_value(profile_json.clone()).ok()
}

/// Extract mesh extension data (bone bindings, materials, topology groups, UDP) from a profile.
fn extract_mesh_extension_data(mesh_data: &mut MeshData, profile: &Bg3MeshProfile) {
    if let Some(ref bindings) = profile.bone_bindings {
        mesh_data.bone_bindings = bindings
            .iter()
            .map(|bb| BoneBindingData {
                bone_name: bb.bone_name.clone(),
                obb_min: bb.obb_min,
                obb_max: bb.obb_max,
                tri_count: bb.tri_count,
                tri_indices: bb.tri_indices.clone(),
            })
            .collect();
    }

    if let Some(ref mats) = profile.material_bindings {
        mesh_data.material_binding_names.clone_from(mats);
    }

    if let Some(ref groups) = profile.topology_groups {
        mesh_data.topology_groups = groups
            .iter()
            .map(|tg| TopologyGroupData {
                material_index: tg.material_index,
                tri_first: tg.tri_first,
                tri_count: tg.tri_count,
            })
            .collect();
    }

    mesh_data
        .user_defined_properties
        .clone_from(&profile.user_defined_properties);
}

/// Remap vertex bone indices from skeleton-global joint indices (glTF) to
/// mesh-local BoneBindings indices (GR2). This is the reverse of
/// `remap_mesh_bone_indices` in the to_gltf direction.
fn remap_joint_indices_to_bone_bindings(mesh: &mut MeshData, skeleton: &Skeleton) {
    if mesh.bone_bindings.is_empty() {
        return;
    }

    // Build mapping: skeleton bone index → BoneBindings array index
    let mut joint_to_binding = vec![0u8; skeleton.bones.len()];
    for (binding_idx, bb) in mesh.bone_bindings.iter().enumerate() {
        if let Some(bone_idx) = skeleton.bones.iter().position(|b| b.name == bb.bone_name) {
            joint_to_binding[bone_idx] = binding_idx as u8;
        }
    }

    for vertex in &mut mesh.vertices {
        for i in 0..4 {
            let joint_idx = vertex.bone_indices[i] as usize;
            vertex.bone_indices[i] = if joint_idx < joint_to_binding.len() {
                joint_to_binding[joint_idx]
            } else {
                0
            };
        }
    }
}

/// Load a mesh primitive.
fn load_primitive(
    primitive: &gltf::Primitive,
    buffers: &[gltf::buffer::Data],
    name: &str,
) -> Result<Option<MeshData>> {
    // Only support triangles
    if primitive.mode() != gltf::mesh::Mode::Triangles {
        return Ok(None);
    }

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    // Read positions (required)
    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or_else(|| Error::ConversionError("Missing positions in mesh".into()))?
        .collect();

    if positions.is_empty() {
        return Ok(None);
    }

    // Read normals
    let normals: Vec<[f32; 3]> = reader.read_normals().map_or_else(
        || vec![[0.0, 0.0, 1.0]; positions.len()],
        std::iter::Iterator::collect,
    );

    // Read tangents
    let tangents: Vec<[f32; 4]> = reader.read_tangents().map_or_else(
        || vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
        std::iter::Iterator::collect,
    );

    // Read UVs
    let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).map_or_else(
        || vec![[0.0, 0.0]; positions.len()],
        |iter| iter.into_f32().collect(),
    );

    // Read colors
    let colors: Vec<[u8; 4]> = reader.read_colors(0).map_or_else(
        || vec![[255, 255, 255, 255]; positions.len()],
        |iter| iter.into_rgba_u8().collect(),
    );

    // Read joints (bone indices)
    let joints: Vec<[u16; 4]> = reader.read_joints(0).map_or_else(
        || vec![[0, 0, 0, 0]; positions.len()],
        |iter| iter.into_u16().collect(),
    );

    // Read weights
    let weights: Vec<[f32; 4]> = reader.read_weights(0).map_or_else(
        || vec![[1.0, 0.0, 0.0, 0.0]; positions.len()],
        |iter| iter.into_f32().collect(),
    );

    // Read indices
    let indices: Vec<u32> = reader.read_indices().map_or_else(
        || (0..positions.len() as u32).collect(),
        |iter| iter.into_u32().collect(),
    );

    // Build vertices with coordinate system conversion
    let mut vertices = Vec::with_capacity(positions.len());

    for i in 0..positions.len() {
        let pos = positions[i];
        let normal = normals[i];
        let tangent = tangents[i];
        let uv = uvs[i];
        let color = colors[i];
        let joint = joints[i];
        let weight = weights[i];

        // Convert position: negate X (inverse of gr2_to_gltf which also negates X)
        let gr2_pos = [-pos[0], pos[1], pos[2]];

        // Convert normal: negate Y and Z (inverse of gr2_to_gltf: [n[0], -n[1], -n[2]])
        let gr2_normal = [normal[0], -normal[1], -normal[2]];

        // Convert tangent: negate X only (inverse of gr2_to_gltf: [-t[0], t[1], t[2], t[3]])
        let gr2_tangent = [-tangent[0], tangent[1], tangent[2], tangent[3]];

        // Encode to QTangent
        let qtangent = encode_qtangent(&gr2_normal, &gr2_tangent);

        // Convert weights to u8 (0-255)
        let bone_weights = [
            (weight[0] * 255.0).clamp(0.0, 255.0) as u8,
            (weight[1] * 255.0).clamp(0.0, 255.0) as u8,
            (weight[2] * 255.0).clamp(0.0, 255.0) as u8,
            (weight[3] * 255.0).clamp(0.0, 255.0) as u8,
        ];

        // Convert joint indices (glTF u16 to GR2 u8)
        let bone_indices = [
            joint[0].min(255) as u8,
            joint[1].min(255) as u8,
            joint[2].min(255) as u8,
            joint[3].min(255) as u8,
        ];

        vertices.push(Vertex {
            position: gr2_pos,
            bone_weights,
            bone_indices,
            qtangent,
            color,
            uv,
        });
    }

    // Flip winding order due to coordinate system change
    let mut flipped_indices = Vec::with_capacity(indices.len());
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            flipped_indices.push(chunk[0]);
            flipped_indices.push(chunk[2]);
            flipped_indices.push(chunk[1]);
        }
    }

    Ok(Some(MeshData {
        name: name.to_string(),
        vertices,
        indices: flipped_indices,
        bg3_profile: None,
        bone_bindings: Vec::new(),
        material_binding_names: Vec::new(),
        topology_groups: Vec::new(),
        user_defined_properties: None,
    }))
}

/// Read a MAT4 accessor.
fn read_accessor_mat4(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<[f32; 16]>> {
    let view = accessor
        .view()
        .ok_or_else(|| Error::ConversionError("Missing buffer view for accessor".into()))?;
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(64);

    let mut matrices = Vec::with_capacity(accessor.count());

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        let mut mat = [0.0f32; 16];
        for j in 0..16 {
            let idx = start + j * 4;
            mat[j] = f32::from_le_bytes([
                buffer[idx],
                buffer[idx + 1],
                buffer[idx + 2],
                buffer[idx + 3],
            ]);
        }
        matrices.push(mat);
    }

    Ok(matrices)
}
