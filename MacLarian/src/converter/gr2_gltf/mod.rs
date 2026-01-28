//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! GR2 and glTF 3D model format conversions
//!
//! Handles conversions between Granny2 (GR2) and glTF formats:
//! - GR2 → glTF/GLB: Export game models for use in Blender
//! - glTF → GR2: Import back into the game

pub mod shared;
pub mod to_gltf;
pub mod to_gr2;
pub mod types;

// Re-export shared utilities
pub use shared::{decode_qtangent, encode_qtangent, f32_to_half, half_to_f32};

// Re-export progress types
pub use types::{Gr2Phase, Gr2Progress, Gr2ProgressCallback};

// Re-export GR2 → glTF conversion functions
pub use to_gltf::{TexturedGlbResult, convert_gr2_bytes_to_glb_with_textures};
pub use to_gltf::{convert_gr2_bytes_to_glb, convert_gr2_to_glb, convert_gr2_to_gltf};
pub use to_gltf::{
    convert_gr2_bytes_to_glb_with_progress, convert_gr2_to_glb_with_progress,
    convert_gr2_to_gltf_with_progress,
};

// Re-export glTF → GR2 conversion functions
pub use to_gr2::{convert_gltf_bytes_to_gr2, convert_gltf_to_gr2};
pub use to_gr2::{convert_gltf_bytes_to_gr2_with_progress, convert_gltf_to_gr2_with_progress};
