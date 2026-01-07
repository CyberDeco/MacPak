//! glTF 2.0 material and texture types.

use serde::Serialize;

/// Image embedded in the GLB binary buffer
#[derive(Debug, Clone, Serialize)]
pub struct GltfImage {
    #[serde(rename = "bufferView")]
    pub buffer_view: usize,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Texture sampler defining filtering and wrapping
#[derive(Debug, Clone, Serialize)]
pub struct GltfSampler {
    #[serde(rename = "magFilter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag_filter: Option<u32>,
    #[serde(rename = "minFilter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_filter: Option<u32>,
    #[serde(rename = "wrapS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_s: Option<u32>,
    #[serde(rename = "wrapT")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_t: Option<u32>,
}

impl Default for GltfSampler {
    fn default() -> Self {
        Self {
            mag_filter: Some(9729),  // LINEAR
            min_filter: Some(9987),  // LINEAR_MIPMAP_LINEAR
            wrap_s: Some(10497),     // REPEAT
            wrap_t: Some(10497),     // REPEAT
        }
    }
}

/// Texture referencing an image and sampler
#[derive(Debug, Clone, Serialize)]
pub struct GltfTexture {
    pub source: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampler: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Texture info used in materials
#[derive(Debug, Clone, Serialize)]
pub struct GltfTextureInfo {
    pub index: usize,
    #[serde(rename = "texCoord")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tex_coord: Option<usize>,
}

/// Normal texture info with scale
#[derive(Debug, Clone, Serialize)]
pub struct GltfNormalTextureInfo {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
    #[serde(rename = "texCoord")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tex_coord: Option<usize>,
}

/// Occlusion texture info with strength
#[derive(Debug, Clone, Serialize)]
pub struct GltfOcclusionTextureInfo {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<f32>,
    #[serde(rename = "texCoord")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tex_coord: Option<usize>,
}

/// PBR Metallic-Roughness material model
#[derive(Debug, Clone, Serialize)]
pub struct GltfPbrMetallicRoughness {
    #[serde(rename = "baseColorFactor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color_factor: Option<[f32; 4]>,
    #[serde(rename = "baseColorTexture")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color_texture: Option<GltfTextureInfo>,
    #[serde(rename = "metallicFactor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic_factor: Option<f32>,
    #[serde(rename = "roughnessFactor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness_factor: Option<f32>,
    #[serde(rename = "metallicRoughnessTexture")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic_roughness_texture: Option<GltfTextureInfo>,
}

impl Default for GltfPbrMetallicRoughness {
    fn default() -> Self {
        Self {
            base_color_factor: Some([1.0, 1.0, 1.0, 1.0]),
            base_color_texture: None,
            metallic_factor: Some(1.0),
            roughness_factor: Some(1.0),
            metallic_roughness_texture: None,
        }
    }
}

/// Material definition
#[derive(Debug, Clone, Serialize)]
pub struct GltfMaterial {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "pbrMetallicRoughness")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pbr_metallic_roughness: Option<GltfPbrMetallicRoughness>,
    #[serde(rename = "normalTexture")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal_texture: Option<GltfNormalTextureInfo>,
    #[serde(rename = "occlusionTexture")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occlusion_texture: Option<GltfOcclusionTextureInfo>,
    #[serde(rename = "emissiveTexture")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_texture: Option<GltfTextureInfo>,
    #[serde(rename = "emissiveFactor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_factor: Option<[f32; 3]>,
    #[serde(rename = "alphaMode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha_mode: Option<String>,
    #[serde(rename = "alphaCutoff")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha_cutoff: Option<f32>,
    #[serde(rename = "doubleSided")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_sided: Option<bool>,
}
