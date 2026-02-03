//! Mesh methods for `GltfBuilder`

use std::collections::HashMap;

use crate::converter::gr2_gltf::to_gltf::gr2_reader::MeshData;
use crate::converter::gr2_gltf::to_gltf::utils::decode_qtangent;

use super::super::types::{GltfMesh, GltfNode, GltfPrimitive};
use super::GltfBuilder;

impl GltfBuilder {
    pub fn add_mesh(&mut self, mesh_data: &MeshData, skin_idx: Option<usize>) -> usize {
        self.add_mesh_internal(mesh_data, skin_idx, None)
    }

    /// Add a mesh with an associated material.
    /// Returns the node index.
    pub fn add_mesh_with_material(
        &mut self,
        mesh_data: &MeshData,
        skin_idx: Option<usize>,
        material_idx: Option<usize>,
    ) -> usize {
        self.add_mesh_internal(mesh_data, skin_idx, material_idx)
    }

    fn add_mesh_internal(
        &mut self,
        mesh_data: &MeshData,
        skin_idx: Option<usize>,
        material_idx: Option<usize>,
    ) -> usize {
        // Extract vertex attributes with X-axis negation for coordinate system conversion
        let positions: Vec<[f32; 3]> = mesh_data
            .vertices
            .iter()
            .map(|v| [-v.position[0], v.position[1], v.position[2]])
            .collect();
        let uvs: Vec<[f32; 2]> = mesh_data.vertices.iter().map(|v| v.uv).collect();
        let colors: Vec<[u8; 4]> = mesh_data.vertices.iter().map(|v| v.color).collect();

        // Decode QTangent to normal/tangent
        let (normals, tangents): (Vec<[f32; 3]>, Vec<[f32; 4]>) = mesh_data
            .vertices
            .iter()
            .map(|v| {
                let (n, t) = decode_qtangent(&v.qtangent);
                ([n[0], -n[1], -n[2]], [-t[0], t[1], t[2], t[3]])
            })
            .unzip();

        // Add all attributes
        let pos_idx = self.add_positions(&positions);
        let norm_idx = self.add_normals(&normals);
        let tan_idx = self.add_tangents(&tangents);
        let uv_idx = self.add_texcoords(&uvs);
        let color_idx = self.add_colors(&colors);

        let mut attributes = HashMap::new();
        attributes.insert("POSITION".to_string(), pos_idx);
        attributes.insert("NORMAL".to_string(), norm_idx);
        attributes.insert("TANGENT".to_string(), tan_idx);
        attributes.insert("TEXCOORD_0".to_string(), uv_idx);
        attributes.insert("COLOR_0".to_string(), color_idx);

        // Only add joints/weights if mesh has skinning
        if skin_idx.is_some() {
            // Clean joints/weights: glTF requires joint index to be 0 when weight is 0
            let (joints, weights): (Vec<[u8; 4]>, Vec<[u8; 4]>) = mesh_data
                .vertices
                .iter()
                .map(|v| {
                    let mut j = v.bone_indices;
                    let w = v.bone_weights;
                    for i in 0..4 {
                        if w[i] == 0 {
                            j[i] = 0;
                        }
                    }
                    (j, w)
                })
                .unzip();

            let joints_idx = self.add_joints(&joints);
            let weights_idx = self.add_weights(&weights);
            attributes.insert("JOINTS_0".to_string(), joints_idx);
            attributes.insert("WEIGHTS_0".to_string(), weights_idx);
        }

        // Add indices - flip winding order to account for X-axis negation
        let indices_idx = if mesh_data.indices.is_empty() {
            None
        } else {
            let flipped_indices: Vec<u32> = mesh_data
                .indices
                .chunks(3)
                .flat_map(|tri| {
                    if tri.len() == 3 {
                        vec![tri[0], tri[2], tri[1]]
                    } else {
                        tri.to_vec()
                    }
                })
                .collect();
            Some(self.add_indices(&flipped_indices, mesh_data.is_32bit_indices))
        };

        let mesh_idx = self.meshes.len();
        self.meshes.push(GltfMesh {
            name: Some(mesh_data.name.clone()),
            primitives: vec![GltfPrimitive {
                attributes,
                indices: indices_idx,
                material: material_idx,
            }],
        });

        // Add node for mesh
        let node_idx = self.nodes.len();
        self.nodes.push(GltfNode {
            name: Some(mesh_data.name.clone()),
            mesh: Some(mesh_idx),
            skin: skin_idx,
            children: Vec::new(),
            translation: None,
            rotation: None,
            scale: None,
        });

        node_idx
    }
}
