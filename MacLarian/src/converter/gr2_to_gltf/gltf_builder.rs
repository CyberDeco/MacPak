//! glTF 2.0 structures and builder.

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::error::{Error, Result};
use super::gr2_reader::{MeshData, Skeleton};
use super::utils::decode_qtangent;

// ============================================================================
// glTF Structures
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct GltfAsset {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfScene {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfSkin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "inverseBindMatrices")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inverse_bind_matrices: Option<usize>,
    pub joints: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfMesh {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub primitives: Vec<GltfPrimitive>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfPrimitive {
    pub attributes: HashMap<String, usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indices: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfAccessor {
    #[serde(rename = "bufferView")]
    pub buffer_view: usize,
    #[serde(rename = "componentType")]
    pub component_type: u32,
    pub count: usize,
    #[serde(rename = "type")]
    pub accessor_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfBufferView {
    pub buffer: usize,
    #[serde(rename = "byteOffset")]
    pub byte_offset: usize,
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfBuffer {
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GltfDocument {
    pub asset: GltfAsset,
    pub scene: usize,
    pub scenes: Vec<GltfScene>,
    pub nodes: Vec<GltfNode>,
    pub meshes: Vec<GltfMesh>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skins: Vec<GltfSkin>,
    pub accessors: Vec<GltfAccessor>,
    #[serde(rename = "bufferViews")]
    pub buffer_views: Vec<GltfBufferView>,
    pub buffers: Vec<GltfBuffer>,
}

// ============================================================================
// glTF Builder
// ============================================================================

pub struct GltfBuilder {
    buffer: Vec<u8>,
    buffer_views: Vec<GltfBufferView>,
    accessors: Vec<GltfAccessor>,
    meshes: Vec<GltfMesh>,
    nodes: Vec<GltfNode>,
    skins: Vec<GltfSkin>,
    pub bone_node_offset: usize,
}

impl GltfBuilder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            buffer_views: Vec::new(),
            accessors: Vec::new(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            skins: Vec::new(),
            bone_node_offset: 0,
        }
    }

    fn align(&mut self, alignment: usize) {
        let padding = (alignment - (self.buffer.len() % alignment)) % alignment;
        self.buffer.extend(std::iter::repeat(0u8).take(padding));
    }

    fn add_positions(&mut self, positions: &[[f32; 3]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for pos in positions {
            for i in 0..3 {
                min[i] = min[i].min(pos[i]);
                max[i] = max[i].max(pos[i]);
            }
            for &v in pos {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: positions.len() * 12,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126, // FLOAT
            count: positions.len(),
            accessor_type: "VEC3".to_string(),
            min: Some(min.to_vec()),
            max: Some(max.to_vec()),
            normalized: None,
        });

        acc_idx
    }

    fn add_normals(&mut self, normals: &[[f32; 3]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for n in normals {
            for &v in n {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: normals.len() * 12,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126,
            count: normals.len(),
            accessor_type: "VEC3".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    fn add_tangents(&mut self, tangents: &[[f32; 4]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for t in tangents {
            for &v in t {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: tangents.len() * 16,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126,
            count: tangents.len(),
            accessor_type: "VEC4".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    fn add_texcoords(&mut self, uvs: &[[f32; 2]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for uv in uvs {
            for &v in uv {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: uvs.len() * 8,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126,
            count: uvs.len(),
            accessor_type: "VEC2".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    fn add_colors(&mut self, colors: &[[u8; 4]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for c in colors {
            self.buffer.extend_from_slice(c);
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: colors.len() * 4,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5121, // UNSIGNED_BYTE
            count: colors.len(),
            accessor_type: "VEC4".to_string(),
            min: None,
            max: None,
            normalized: Some(true),
        });

        acc_idx
    }

    fn add_joints(&mut self, joints: &[[u8; 4]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for j in joints {
            self.buffer.extend_from_slice(j);
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: joints.len() * 4,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5121, // UNSIGNED_BYTE
            count: joints.len(),
            accessor_type: "VEC4".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    fn add_weights(&mut self, weights: &[[u8; 4]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for w in weights {
            self.buffer.extend_from_slice(w);
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: weights.len() * 4,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5121,
            count: weights.len(),
            accessor_type: "VEC4".to_string(),
            min: None,
            max: None,
            normalized: Some(true),
        });

        acc_idx
    }

    fn add_indices(&mut self, indices: &[u32], use_32bit: bool) -> usize {
        let byte_offset;
        let byte_length;
        let component_type;

        if use_32bit || indices.iter().any(|&i| i > 65535) {
            self.align(4);
            byte_offset = self.buffer.len();
            for &idx in indices {
                self.buffer.extend_from_slice(&idx.to_le_bytes());
            }
            byte_length = indices.len() * 4;
            component_type = 5125; // UNSIGNED_INT
        } else {
            self.align(2);
            byte_offset = self.buffer.len();
            for &idx in indices {
                self.buffer.extend_from_slice(&(idx as u16).to_le_bytes());
            }
            byte_length = indices.len() * 2;
            component_type = 5123; // UNSIGNED_SHORT
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length,
            target: Some(34963), // ELEMENT_ARRAY_BUFFER
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type,
            count: indices.len(),
            accessor_type: "SCALAR".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    fn add_inverse_bind_matrices(&mut self, matrices: &[[f32; 16]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for mat in matrices {
            for &v in mat {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: matrices.len() * 64,
            target: None,
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126, // FLOAT
            count: matrices.len(),
            accessor_type: "MAT4".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    pub fn add_skeleton(&mut self, skeleton: &Skeleton) -> usize {
        self.bone_node_offset = self.nodes.len();

        // Add bone nodes
        for (bone_idx, bone) in skeleton.bones.iter().enumerate() {
            let children: Vec<usize> = skeleton.bones.iter()
                .enumerate()
                .filter(|(_, b)| b.parent_index >= 0 && b.parent_index as usize == bone_idx)
                .map(|(i, _)| self.bone_node_offset + i)
                .collect();

            let translation = Some(bone.transform.translation);
            let rotation = Some(bone.transform.rotation);
            let scale = Some([
                bone.transform.scale_shear[0],
                bone.transform.scale_shear[4],
                bone.transform.scale_shear[8],
            ]);

            self.nodes.push(GltfNode {
                name: Some(bone.name.clone()),
                mesh: None,
                skin: None,
                children,
                translation,
                rotation,
                scale,
            });
        }

        let ibm: Vec<[f32; 16]> = skeleton.bones.iter()
            .map(|b| b.inverse_world_transform)
            .collect();

        let ibm_accessor = self.add_inverse_bind_matrices(&ibm);

        let joints: Vec<usize> = (0..skeleton.bones.len())
            .map(|i| self.bone_node_offset + i)
            .collect();

        let root_bone_idx = skeleton.bones.iter()
            .position(|b| b.parent_index < 0)
            .map(|i| self.bone_node_offset + i);

        let skin_idx = self.skins.len();
        self.skins.push(GltfSkin {
            name: Some(skeleton.name.clone()),
            inverse_bind_matrices: Some(ibm_accessor),
            joints,
            skeleton: root_bone_idx,
        });

        skin_idx
    }

    pub fn add_mesh(&mut self, mesh_data: &MeshData, skin_idx: Option<usize>) -> usize {
        // Extract vertex attributes with X-axis negation for coordinate system conversion
        let positions: Vec<[f32; 3]> = mesh_data.vertices.iter()
            .map(|v| [-v.position[0], v.position[1], v.position[2]])
            .collect();
        let uvs: Vec<[f32; 2]> = mesh_data.vertices.iter().map(|v| v.uv).collect();
        let colors: Vec<[u8; 4]> = mesh_data.vertices.iter().map(|v| v.color).collect();

        // Clean joints/weights: glTF requires joint index to be 0 when weight is 0
        let (joints, weights): (Vec<[u8; 4]>, Vec<[u8; 4]>) = mesh_data.vertices.iter()
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
        let joints_idx = self.add_joints(&joints);
        let weights_idx = self.add_weights(&weights);

        let mut attributes = HashMap::new();
        attributes.insert("POSITION".to_string(), pos_idx);
        attributes.insert("NORMAL".to_string(), norm_idx);
        attributes.insert("TANGENT".to_string(), tan_idx);
        attributes.insert("TEXCOORD_0".to_string(), uv_idx);
        attributes.insert("COLOR_0".to_string(), color_idx);
        attributes.insert("JOINTS_0".to_string(), joints_idx);
        attributes.insert("WEIGHTS_0".to_string(), weights_idx);

        // Add indices - flip winding order to account for X-axis negation
        let indices_idx = if !mesh_data.indices.is_empty() {
            let flipped_indices: Vec<u32> = mesh_data.indices
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
        } else {
            None
        };

        let mesh_idx = self.meshes.len();
        self.meshes.push(GltfMesh {
            name: Some(mesh_data.name.clone()),
            primitives: vec![GltfPrimitive {
                attributes,
                indices: indices_idx,
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

    fn build_document(self, root_bone_idx: Option<usize>) -> (GltfDocument, Vec<u8>) {
        let mut scene_nodes = Vec::new();

        if let Some(root_idx) = root_bone_idx {
            scene_nodes.push(root_idx);
        }

        for (i, node) in self.nodes.iter().enumerate() {
            if node.mesh.is_some() {
                scene_nodes.push(i);
            }
        }

        let doc = GltfDocument {
            asset: GltfAsset {
                version: "2.0".to_string(),
                generator: Some("MacLarian GR2 to glTF converter".to_string()),
            },
            scene: 0,
            scenes: vec![GltfScene {
                name: Some("Scene".to_string()),
                nodes: scene_nodes,
            }],
            nodes: self.nodes,
            meshes: self.meshes,
            skins: self.skins,
            accessors: self.accessors,
            buffer_views: self.buffer_views,
            buffers: vec![GltfBuffer {
                byte_length: self.buffer.len(),
            }],
        };

        (doc, self.buffer)
    }

    /// Build GLB data and return as bytes.
    pub fn build_glb(self, root_bone_idx: Option<usize>) -> Result<Vec<u8>> {
        let (doc, buffer) = self.build_document(root_bone_idx);
        let json = serde_json::to_string(&doc)
            .map_err(|e| Error::ConversionError(format!("JSON serialization error: {}", e)))?;
        let json_bytes = json.as_bytes();

        let json_padding = (4 - (json_bytes.len() % 4)) % 4;
        let json_chunk_len = json_bytes.len() + json_padding;

        let bin_padding = (4 - (buffer.len() % 4)) % 4;
        let bin_chunk_len = buffer.len() + bin_padding;

        let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

        let mut output = Vec::with_capacity(total_len);

        // GLB header
        output.extend_from_slice(b"glTF");
        output.extend_from_slice(&2u32.to_le_bytes());
        output.extend_from_slice(&(total_len as u32).to_le_bytes());

        // JSON chunk
        output.extend_from_slice(&(json_chunk_len as u32).to_le_bytes());
        output.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
        output.extend_from_slice(json_bytes);
        for _ in 0..json_padding {
            output.push(b' ');
        }

        // Binary chunk
        output.extend_from_slice(&(bin_chunk_len as u32).to_le_bytes());
        output.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
        output.extend_from_slice(&buffer);
        for _ in 0..bin_padding {
            output.push(0u8);
        }

        Ok(output)
    }

    pub fn export_glb(self, path: &Path, root_bone_idx: Option<usize>) -> Result<()> {
        let glb_data = self.build_glb(root_bone_idx)?;
        let mut file = File::create(path)?;
        file.write_all(&glb_data)?;
        Ok(())
    }
}

impl Default for GltfBuilder {
    fn default() -> Self {
        Self::new()
    }
}
