//! GR2 to glTF converter
//!
//! Converts Granny2 GR2 files to glTF 2.0 format.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::collapsible_if,
    clippy::struct_excessive_bools
)]

mod convert;
mod gltf;
mod gr2_reader;
mod texture_loading;
mod textured;
mod utils;

pub use gltf::GltfBuilder;
pub use gr2_reader::{Bone, Gr2ContentInfo, Gr2Reader, MeshData, Skeleton, Transform, Vertex};
pub use utils::{decode_qtangent, half_to_f32};

// Re-export conversion functions
pub use convert::{
    convert_gr2_bytes_to_glb, convert_gr2_bytes_to_glb_with_progress, convert_gr2_to_glb,
    convert_gr2_to_glb_with_progress, convert_gr2_to_gltf, convert_gr2_to_gltf_with_progress,
};

pub use textured::{TexturedGlbResult, convert_gr2_bytes_to_glb_with_textures};
