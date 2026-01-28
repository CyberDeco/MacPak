//! glTF 2.0 document builder.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::same_item_push)]

mod export;
mod material_methods;
mod mesh;
mod skeleton;
mod vertex_attributes;

use super::types::{GltfBufferView, GltfAccessor, GltfMesh, GltfNode, GltfSkin};
use super::materials::{GltfImage, GltfTexture, GltfSampler, GltfMaterial};

/// Builder for constructing glTF documents.
pub struct GltfBuilder {
    pub(crate) buffer: Vec<u8>,
    pub(crate) buffer_views: Vec<GltfBufferView>,
    pub(crate) accessors: Vec<GltfAccessor>,
    pub(crate) meshes: Vec<GltfMesh>,
    pub(crate) nodes: Vec<GltfNode>,
    pub(crate) skins: Vec<GltfSkin>,
    pub(crate) images: Vec<GltfImage>,
    pub(crate) textures: Vec<GltfTexture>,
    pub(crate) samplers: Vec<GltfSampler>,
    pub(crate) materials: Vec<GltfMaterial>,
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

    pub(crate) fn align(&mut self, alignment: usize) {
        let padding = (alignment - (self.buffer.len() % alignment)) % alignment;
        self.buffer.extend(std::iter::repeat_n(0u8, padding));
    }
}

impl Default for GltfBuilder {
    fn default() -> Self {
        Self::new()
    }
}
