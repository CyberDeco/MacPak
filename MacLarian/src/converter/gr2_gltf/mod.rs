//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! GR2 and glTF 3D model format conversions
//!
//! Handles conversions between Granny2 (GR2) and glTF formats:
//! - GR2 → glTF/GLB: Export game models for use in 3D editors
//! - glTF → GR2: Import custom models into the game

pub mod shared;
pub mod to_gltf;
pub mod to_gr2;

// Re-export shared utilities
pub use shared::{half_to_f32, f32_to_half, decode_qtangent, encode_qtangent};

// Re-export GR2 → glTF conversion functions
pub use to_gltf::{convert_gr2_to_glb, convert_gr2_to_gltf, convert_gr2_bytes_to_glb};
pub use to_gltf::{convert_gr2_bytes_to_glb_with_textures, TexturedGlbResult};
pub use to_gltf::{Gr2Reader, MeshData, Vertex, Skeleton, Bone, Transform, Gr2ContentInfo};
pub use to_gltf::GltfBuilder;

// Re-export glTF → GR2 conversion functions
pub use to_gr2::{convert_gltf_to_gr2, convert_gltf_bytes_to_gr2};
