//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! glTF file loader.
//!
//! Loads glTF/GLB files and converts them to intermediate structures
//! suitable for GR2 export.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{Error, Result};
use super::utils::encode_qtangent;

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

/// Mesh data extracted from glTF.
#[derive(Clone)]
pub struct MeshData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
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
}

/// Skeleton data.
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<Bone>,
}

/// Complete glTF model.
pub struct GltfModel {
    pub meshes: Vec<MeshData>,
    pub skeleton: Option<Skeleton>,
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

    fn load_from_document(document: &gltf::Document, buffers: &[gltf::buffer::Data]) -> Result<Self> {
        let mut meshes = Vec::new();
        let mut skeleton = None;

        // First pass: find skeleton from skins
        for skin in document.skins() {
            skeleton = Some(load_skeleton(document, &skin, buffers)?);
            break; // Only support one skeleton for now
        }

        // Build a map of node indices to bone indices (for future use with bone bindings)
        let _bone_map: HashMap<usize, usize> = skeleton.as_ref()
            .map(|_| {
                document.skins().next()
                    .map(|skin| {
                        skin.joints()
                            .enumerate()
                            .map(|(bone_idx, joint)| (joint.index(), bone_idx))
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        // Second pass: load meshes
        for node in document.nodes() {
            if let Some(mesh) = node.mesh() {
                let node_name = node.name().unwrap_or("Mesh");

                for (prim_idx, primitive) in mesh.primitives().enumerate() {
                    let name = if mesh.primitives().len() > 1 {
                        format!("{}_{}", mesh.name().unwrap_or(node_name), prim_idx)
                    } else {
                        mesh.name().unwrap_or(node_name).to_string()
                    };

                    if let Some(mesh_data) = load_primitive(&primitive, buffers, &name)? {
                        meshes.push(mesh_data);
                    }
                }
            }
        }

        if meshes.is_empty() {
            return Err(Error::ConversionError("No meshes found in glTF file".to_string()));
        }

        Ok(GltfModel { meshes, skeleton })
    }
}

/// Load skeleton from a glTF skin.
fn load_skeleton(
    document: &gltf::Document,
    skin: &gltf::Skin,
    buffers: &[gltf::buffer::Data],
) -> Result<Skeleton> {
    let name = skin.name().unwrap_or("Skeleton").to_string();
    let joints: Vec<_> = skin.joints().collect();

    // Build parent index map
    let joint_indices: HashMap<usize, usize> = joints.iter()
        .enumerate()
        .map(|(i, j)| (j.index(), i))
        .collect();

    // Read inverse bind matrices if available
    let ibm: Vec<[f32; 16]> = if let Some(accessor) = skin.inverse_bind_matrices() {
        read_accessor_mat4(&accessor, buffers)?
    } else {
        // Identity matrices as fallback
        vec![[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]; joints.len()]
    };

    let mut bones = Vec::with_capacity(joints.len());

    for (bone_idx, joint) in joints.iter().enumerate() {
        let bone_name = joint.name().unwrap_or(&format!("Bone_{bone_idx}")).to_string();

        // Find parent index
        let parent_index = find_parent_bone_index(document, joint, &joint_indices);

        // Get local transform
        let (translation, rotation, scale) = joint.transform().decomposed();

        // NO coordinate conversion for bone transforms - gr2_to_gltf exports them as-is
        let transform = Transform {
            translation,
            rotation,
            scale_shear: [
                scale[0], 0.0, 0.0,
                0.0, scale[1], 0.0,
                0.0, 0.0, scale[2],
            ],
        };

        // NO coordinate conversion for inverse bind matrices - gr2_to_gltf exports them as-is
        let inverse_world_transform = ibm.get(bone_idx)
            .copied()
            .unwrap_or([
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ]);

        bones.push(Bone {
            name: bone_name,
            parent_index,
            transform,
            inverse_world_transform,
        });
    }

    Ok(Skeleton { name, bones })
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
    let positions: Vec<[f32; 3]> = reader.read_positions()
        .ok_or_else(|| Error::ConversionError("Missing positions in mesh".into()))?
        .collect();

    if positions.is_empty() {
        return Ok(None);
    }

    // Read normals
    let normals: Vec<[f32; 3]> = reader.read_normals().map_or_else(|| vec![[0.0, 0.0, 1.0]; positions.len()], std::iter::Iterator::collect);

    // Read tangents
    let tangents: Vec<[f32; 4]> = reader.read_tangents().map_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()], std::iter::Iterator::collect);

    // Read UVs
    let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).map_or_else(|| vec![[0.0, 0.0]; positions.len()], |iter| iter.into_f32().collect());

    // Read colors
    let colors: Vec<[u8; 4]> = reader.read_colors(0).map_or_else(|| vec![[255, 255, 255, 255]; positions.len()], |iter| iter.into_rgba_u8().collect());

    // Read joints (bone indices)
    let joints: Vec<[u16; 4]> = reader.read_joints(0).map_or_else(|| vec![[0, 0, 0, 0]; positions.len()], |iter| iter.into_u16().collect());

    // Read weights
    let weights: Vec<[f32; 4]> = reader.read_weights(0).map_or_else(|| vec![[1.0, 0.0, 0.0, 0.0]; positions.len()], |iter| iter.into_f32().collect());

    // Read indices
    let indices: Vec<u32> = reader.read_indices().map_or_else(|| (0..positions.len() as u32).collect(), |iter| iter.into_u32().collect());

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
    }))
}

/// Read a MAT4 accessor.
fn read_accessor_mat4(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<[f32; 16]>> {
    let view = accessor.view()
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
