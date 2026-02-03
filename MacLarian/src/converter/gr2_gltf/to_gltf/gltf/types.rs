//! Core glTF 2.0 structure types.

use serde::Serialize;
use std::collections::HashMap;

use super::materials::{GltfImage, GltfMaterial, GltfSampler, GltfTexture};

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
}

/// Mesh definition
#[derive(Debug, Clone, Serialize)]
pub struct GltfMesh {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub primitives: Vec<GltfPrimitive>,
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
}
