//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! glTF 2.0 types and builder.
//!
//! This module provides types and utilities for constructing glTF 2.0 documents.

mod builder;
mod materials;
mod types;

// Re-export builder
pub use builder::GltfBuilder;

// Re-export all types for external use
#[allow(unused_imports)]
pub use types::{
    GltfAccessor,
    GltfAsset,
    GltfBuffer,
    GltfBufferView,
    GltfDocument,
    GltfMesh,
    GltfNode,
    GltfPrimitive,
    GltfScene,
    GltfSkin,
};

// Re-export material types for external use
#[allow(unused_imports)]
pub use materials::{
    GltfImage,
    GltfMaterial,
    GltfNormalTextureInfo,
    GltfOcclusionTextureInfo,
    GltfPbrMetallicRoughness,
    GltfSampler,
    GltfTexture,
    GltfTextureInfo,
};
