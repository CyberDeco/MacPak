//! glTF 2.0 document builder.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::same_item_push)]

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::{Error, Result};
use crate::converter::gr2_gltf::to_gltf::gr2_reader::{MeshData, Skeleton};
use crate::converter::gr2_gltf::to_gltf::utils::decode_qtangent;

use super::types::{GltfBufferView, GltfAccessor, GltfMesh, GltfNode, GltfSkin, GltfPrimitive, GltfDocument, GltfAsset, GltfScene, GltfBuffer};
use super::materials::{GltfImage, GltfTexture, GltfSampler, GltfMaterial, GltfPbrMetallicRoughness, GltfTextureInfo, GltfNormalTextureInfo, GltfOcclusionTextureInfo};

/// Builder for constructing glTF documents.
pub struct GltfBuilder {
    buffer: Vec<u8>,
    buffer_views: Vec<GltfBufferView>,
    accessors: Vec<GltfAccessor>,
    meshes: Vec<GltfMesh>,
    nodes: Vec<GltfNode>,
    skins: Vec<GltfSkin>,
    images: Vec<GltfImage>,
    textures: Vec<GltfTexture>,
    samplers: Vec<GltfSampler>,
    materials: Vec<GltfMaterial>,
    pub bone_node_offset: usize,
}

impl GltfBuilder {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            buffer_views: Vec::new(),
            accessors: Vec::new(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            skins: Vec::new(),
            images: Vec::new(),
            textures: Vec::new(),
            samplers: Vec::new(),
            materials: Vec::new(),
            bone_node_offset: 0,
        }
    }

    fn align(&mut self, alignment: usize) {
        let padding = (alignment - (self.buffer.len() % alignment)) % alignment;
        self.buffer.extend(std::iter::repeat_n(0u8, padding));
    }

    // ========================================================================
    // Vertex Attribute Methods
    // ========================================================================

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

    // ========================================================================
    // Image/Texture/Material Methods
    // ========================================================================

    /// Add an embedded image (PNG bytes) to the GLB.
    /// Returns the image index.
    pub fn add_embedded_image(&mut self, png_data: &[u8], name: Option<String>) -> usize {
        // Align to 4 for consistency (not necessary but it's whatev)
        self.align(4);
        let byte_offset = self.buffer.len();
        self.buffer.extend_from_slice(png_data);

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: png_data.len(),
            target: None, // No target for images
        });

        let img_idx = self.images.len();
        self.images.push(GltfImage {
            buffer_view: bv_idx,
            mime_type: "image/png".to_string(),
            name,
        });

        img_idx
    }

    /// Add a texture sampler with default settings (linear filtering, repeat wrap).
    /// Returns the sampler index.
    pub fn add_sampler(&mut self) -> usize {
        let sampler_idx = self.samplers.len();
        self.samplers.push(GltfSampler::default());
        sampler_idx
    }

    /// Add a custom texture sampler.
    /// Returns the sampler index.
    pub fn add_sampler_custom(&mut self, sampler: GltfSampler) -> usize {
        let sampler_idx = self.samplers.len();
        self.samplers.push(sampler);
        sampler_idx
    }

    /// Add a texture referencing an image and optionally a sampler.
    /// Returns the texture index.
    pub fn add_texture(
        &mut self,
        image_index: usize,
        sampler_index: Option<usize>,
        name: Option<String>,
    ) -> usize {
        let tex_idx = self.textures.len();
        self.textures.push(GltfTexture {
            source: image_index,
            sampler: sampler_index,
            name,
        });
        tex_idx
    }

    /// Add a PBR material with optional textures.
    ///
    /// # Arguments
    /// * `name` - Optional material name
    /// * `base_color_texture` - Albedo/diffuse texture index
    /// * `normal_texture` - Normal map texture index
    /// * `metallic_roughness_texture` - Combined metallic-roughness texture index
    /// * `occlusion_texture` - Ambient occlusion texture index
    ///
    /// Returns the material index.
    pub fn add_material(
        &mut self,
        name: Option<String>,
        base_color_texture: Option<usize>,
        normal_texture: Option<usize>,
        metallic_roughness_texture: Option<usize>,
        occlusion_texture: Option<usize>,
    ) -> usize {
        let pbr = GltfPbrMetallicRoughness {
            base_color_factor: Some([1.0, 1.0, 1.0, 1.0]),
            base_color_texture: base_color_texture.map(|idx| GltfTextureInfo {
                index: idx,
                tex_coord: None,
            }),
            metallic_factor: Some(1.0),
            roughness_factor: Some(1.0),
            metallic_roughness_texture: metallic_roughness_texture.map(|idx| GltfTextureInfo {
                index: idx,
                tex_coord: None,
            }),
        };

        let mat_idx = self.materials.len();
        self.materials.push(GltfMaterial {
            name,
            pbr_metallic_roughness: Some(pbr),
            normal_texture: normal_texture.map(|idx| GltfNormalTextureInfo {
                index: idx,
                scale: Some(1.0),
                tex_coord: None,
            }),
            occlusion_texture: occlusion_texture.map(|idx| GltfOcclusionTextureInfo {
                index: idx,
                strength: Some(1.0),
                tex_coord: None,
            }),
            emissive_texture: None,
            emissive_factor: None,
            alpha_mode: None,
            alpha_cutoff: None,
            double_sided: None,
        });

        mat_idx
    }

    /// Add a simple material with just a base color texture.
    /// Returns the material index.
    pub fn add_simple_material(
        &mut self,
        name: Option<String>,
        base_color_texture: usize,
    ) -> usize {
        self.add_material(name, Some(base_color_texture), None, None, None)
    }

    /// Convenience method to add an image, create a texture for it, and return the texture index.
    /// Also creates a default sampler if none exists.
    pub fn add_image_as_texture(&mut self, png_data: &[u8], name: Option<String>) -> usize {
        // Ensure there's at least one sampler
        let sampler_idx = if self.samplers.is_empty() {
            Some(self.add_sampler())
        } else {
            Some(0)
        };

        let image_idx = self.add_embedded_image(png_data, name.clone());
        self.add_texture(image_idx, sampler_idx, name)
    }

    // ========================================================================
    // Skeleton Methods
    // ========================================================================

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

    // ========================================================================
    // Mesh Methods
    // ========================================================================

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
        let positions: Vec<[f32; 3]> = mesh_data.vertices.iter()
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

            let joints_idx = self.add_joints(&joints);
            let weights_idx = self.add_weights(&weights);
            attributes.insert("JOINTS_0".to_string(), joints_idx);
            attributes.insert("WEIGHTS_0".to_string(), weights_idx);
        }

        // Add indices - flip winding order to account for X-axis negation
        let indices_idx = if mesh_data.indices.is_empty() {
            None
        } else {
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

    /// Set material for an existing mesh (by mesh index).
    pub fn set_mesh_material(&mut self, mesh_idx: usize, material_idx: usize) {
        if let Some(mesh) = self.meshes.get_mut(mesh_idx) {
            for prim in &mut mesh.primitives {
                prim.material = Some(material_idx);
            }
        }
    }

    // ========================================================================
    // Export Methods
    // ========================================================================

    fn build_document(self, root_bone_idx: Option<usize>, buffer_uri: Option<String>) -> (GltfDocument, Vec<u8>) {
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
            materials: self.materials,
            textures: self.textures,
            images: self.images,
            samplers: self.samplers,
            accessors: self.accessors,
            buffer_views: self.buffer_views,
            buffers: vec![GltfBuffer {
                byte_length: self.buffer.len(),
                uri: buffer_uri,
            }],
        };

        (doc, self.buffer)
    }

    /// Build GLB data and return as bytes.
    ///
    /// # Errors
    /// Returns an error if JSON serialization fails.
    pub fn build_glb(self, root_bone_idx: Option<usize>) -> Result<Vec<u8>> {
        let (doc, buffer) = self.build_document(root_bone_idx, None);
        let json = serde_json::to_string(&doc)
            .map_err(|e| Error::ConversionError(format!("JSON serialization error: {e}")))?;
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

    /// Export as a GLB file.
    ///
    /// # Errors
    /// Returns an error if serialization or file writing fails.
    pub fn export_glb(self, path: &Path, root_bone_idx: Option<usize>) -> Result<()> {
        let glb_data = self.build_glb(root_bone_idx)?;
        let mut file = File::create(path)?;
        file.write_all(&glb_data)?;
        Ok(())
    }

    /// Export as separate .gltf (JSON) and .bin (binary buffer) files.
    ///
    /// # Errors
    /// Returns an error if serialization or file writing fails.
    pub fn export_gltf(self, path: &Path, root_bone_idx: Option<usize>) -> Result<()> {
        // Determine the .bin file name (same base name, .bin extension)
        let bin_filename = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| format!("{s}.bin"))
            .ok_or_else(|| Error::ConversionError("Invalid output path".to_string()))?;

        let bin_path = path.with_file_name(&bin_filename);

        // Build document with URI pointing to the .bin file
        let (doc, buffer) = self.build_document(root_bone_idx, Some(bin_filename));

        // Write JSON to .gltf file
        let json = serde_json::to_string_pretty(&doc)
            .map_err(|e| Error::ConversionError(format!("JSON serialization error: {e}")))?;
        let mut gltf_file = File::create(path)?;
        gltf_file.write_all(json.as_bytes())?;

        // Write binary buffer to .bin file
        let mut bin_file = File::create(&bin_path)?;
        bin_file.write_all(&buffer)?;

        Ok(())
    }
}

impl Default for GltfBuilder {
    fn default() -> Self {
        Self::new()
    }
}
