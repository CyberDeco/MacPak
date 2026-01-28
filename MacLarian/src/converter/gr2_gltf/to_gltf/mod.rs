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

mod gr2_reader;
mod gltf;
mod utils;
mod convert;
mod texture_loading;
mod textured;

pub use gr2_reader::{Gr2Reader, MeshData, Vertex, Skeleton, Bone, Transform, Gr2ContentInfo};
pub use gltf::GltfBuilder;
pub use utils::{half_to_f32, decode_qtangent};

// Re-export conversion functions
pub use convert::{
    convert_gr2_to_gltf,
    convert_gr2_to_gltf_with_progress,
    convert_gr2_to_glb,
    convert_gr2_to_glb_with_progress,
    convert_gr2_bytes_to_glb,
    convert_gr2_bytes_to_glb_with_progress,
};

pub use textured::{TexturedGlbResult, convert_gr2_bytes_to_glb_with_textures};
