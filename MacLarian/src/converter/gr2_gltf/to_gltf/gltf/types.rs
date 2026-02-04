//! Core glTF 2.0 structure types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::materials::{GltfImage, GltfMaterial, GltfSampler, GltfTexture};

/// A bone binding for the glTF extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bg3BoneBinding {
    #[serde(rename = "BoneName")]
    pub bone_name: String,
    #[serde(rename = "OBBMin")]
    pub obb_min: [f32; 3],
    #[serde(rename = "OBBMax")]
    pub obb_max: [f32; 3],
    #[serde(rename = "TriCount")]
    pub tri_count: i32,
    #[serde(rename = "TriIndices")]
    pub tri_indices: Vec<i32>,
}

/// A topology group for the glTF extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bg3TopologyGroup {
    #[serde(rename = "MaterialIndex")]
    pub material_index: i32,
    #[serde(rename = "TriFirst")]
    pub tri_first: i32,
    #[serde(rename = "TriCount")]
    pub tri_count: i32,
}

/// Mesh-level metadata for the `MACLARIAN_glTF_extensions` glTF extension.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bg3MeshProfile {
    #[serde(rename = "Rigid", skip_serializing_if = "Option::is_none")]
    pub rigid: Option<bool>,
    #[serde(rename = "Cloth", skip_serializing_if = "Option::is_none")]
    pub cloth: Option<bool>,
    #[serde(rename = "MeshProxy", skip_serializing_if = "Option::is_none")]
    pub mesh_proxy: Option<bool>,
    #[serde(rename = "ProxyGeometry", skip_serializing_if = "Option::is_none")]
    pub proxy_geometry: Option<bool>,
    #[serde(rename = "Spring", skip_serializing_if = "Option::is_none")]
    pub spring: Option<bool>,
    #[serde(rename = "Occluder", skip_serializing_if = "Option::is_none")]
    pub occluder: Option<bool>,
    #[serde(rename = "ClothPhysics", skip_serializing_if = "Option::is_none")]
    pub cloth_physics: Option<bool>,
    #[serde(rename = "Cloth01", skip_serializing_if = "Option::is_none")]
    pub cloth_01: Option<bool>,
    #[serde(rename = "Cloth02", skip_serializing_if = "Option::is_none")]
    pub cloth_02: Option<bool>,
    #[serde(rename = "Cloth04", skip_serializing_if = "Option::is_none")]
    pub cloth_04: Option<bool>,
    #[serde(rename = "Impostor", skip_serializing_if = "Option::is_none")]
    pub impostor: Option<bool>,
    #[serde(rename = "ExportOrder", skip_serializing_if = "Option::is_none")]
    pub export_order: Option<i32>,
    #[serde(rename = "LOD", skip_serializing_if = "Option::is_none")]
    pub lod: Option<i32>,
    #[serde(rename = "LODDistance", skip_serializing_if = "Option::is_none")]
    pub lod_distance: Option<f32>,
    #[serde(rename = "ParentBone", skip_serializing_if = "Option::is_none")]
    pub parent_bone: Option<String>,
    #[serde(rename = "BoneBindings", skip_serializing_if = "Option::is_none")]
    pub bone_bindings: Option<Vec<Bg3BoneBinding>>,
    #[serde(rename = "MaterialBindings", skip_serializing_if = "Option::is_none")]
    pub material_bindings: Option<Vec<String>>,
    #[serde(rename = "TopologyGroups", skip_serializing_if = "Option::is_none")]
    pub topology_groups: Option<Vec<Bg3TopologyGroup>>,
    #[serde(rename = "UserDefinedProperties", skip_serializing_if = "Option::is_none")]
    pub user_defined_properties: Option<String>,
}

/// Wrapper for mesh-level glTF extensions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GltfMeshExtensions {
    #[serde(
        rename = "MACLARIAN_glTF_extensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub bg3_profile: Option<Bg3MeshProfile>,
}

/// Asset metadata
#[derive(Debug, Clone, Serialize)]
pub struct GltfAsset {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
}

/// Scene definition
#[derive(Debug, Clone, Serialize)]
pub struct GltfScene {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<usize>,
}

/// Node in the scene graph
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

/// Transform for the glTF extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bg3Transform {
    #[serde(rename = "Translation")]
    pub translation: [f32; 3],
    #[serde(rename = "Rotation")]
    pub rotation: [f32; 4],
    #[serde(rename = "ScaleShear")]
    pub scale_shear: [f32; 9],
}

/// Skin-level metadata for the `MACLARIAN_glTF_extensions` glTF extension.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bg3SkeletonProfile {
    #[serde(rename = "LODType", skip_serializing_if = "Option::is_none")]
    pub lod_type: Option<i32>,
    #[serde(rename = "BoneLODError", skip_serializing_if = "Option::is_none")]
    pub bone_lod_error: Option<Vec<f32>>,
    #[serde(rename = "ModelName", skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(rename = "ModelMeshBindings", skip_serializing_if = "Option::is_none")]
    pub model_mesh_bindings: Option<Vec<String>>,
    #[serde(rename = "InitialPlacement", skip_serializing_if = "Option::is_none")]
    pub initial_placement: Option<Bg3Transform>,
}

/// Wrapper for skin-level glTF extensions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GltfSkinExtensions {
    #[serde(
        rename = "MACLARIAN_glTF_extensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub bg3_profile: Option<Bg3SkeletonProfile>,
}

/// Skin for skeletal animation
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<GltfSkinExtensions>,
}

/// Mesh definition
#[derive(Debug, Clone, Serialize)]
pub struct GltfMesh {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub primitives: Vec<GltfPrimitive>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<GltfMeshExtensions>,
}

/// Mesh primitive (geometry + material)
#[derive(Debug, Clone, Serialize)]
pub struct GltfPrimitive {
    pub attributes: HashMap<String, usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indices: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<usize>,
}

/// Accessor for typed buffer data
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

/// Buffer view (slice of a buffer)
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

/// Binary buffer
#[derive(Debug, Clone, Serialize)]
pub struct GltfBuffer {
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// Complete glTF document
#[derive(Debug, Clone, Serialize)]
pub struct GltfDocument {
    pub asset: GltfAsset,
    pub scene: usize,
    pub scenes: Vec<GltfScene>,
    pub nodes: Vec<GltfNode>,
    pub meshes: Vec<GltfMesh>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skins: Vec<GltfSkin>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<GltfMaterial>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub textures: Vec<GltfTexture>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<GltfImage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub samplers: Vec<GltfSampler>,
    pub accessors: Vec<GltfAccessor>,
    #[serde(rename = "bufferViews")]
    pub buffer_views: Vec<GltfBufferView>,
    pub buffers: Vec<GltfBuffer>,
    #[serde(rename = "extensionsUsed", skip_serializing_if = "Vec::is_empty")]
    pub extensions_used: Vec<String>,
}
