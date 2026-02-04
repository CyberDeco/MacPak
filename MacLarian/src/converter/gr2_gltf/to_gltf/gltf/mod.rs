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
    Bg3BoneBinding, Bg3MeshProfile, Bg3SkeletonProfile, Bg3TopologyGroup, Bg3Transform,
    GltfAccessor, GltfAsset, GltfBuffer, GltfBufferView, GltfDocument, GltfMesh,
    GltfMeshExtensions, GltfNode, GltfPrimitive, GltfScene, GltfSkin, GltfSkinExtensions,
};

// Re-export material types for external use
#[allow(unused_imports)]
pub use materials::{
    GltfImage, GltfMaterial, GltfNormalTextureInfo, GltfOcclusionTextureInfo,
    GltfPbrMetallicRoughness, GltfSampler, GltfTexture, GltfTextureInfo,
};
