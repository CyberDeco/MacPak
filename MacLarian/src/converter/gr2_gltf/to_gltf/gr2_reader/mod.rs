//! GR2 format structures and parser.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

mod types;
mod vertex_types;
mod reader;

pub use types::{Vertex, MeshData, Transform, Bone, Skeleton, Gr2ContentInfo};
pub use reader::Gr2Reader;

// ============================================================================
// Constants
// ============================================================================

pub const MAGIC_LE64: [u8; 16] = [
    0xE5, 0x9B, 0x49, 0x5E, 0x6F, 0x63, 0x1F, 0x14,
    0x1E, 0x13, 0xEB, 0xA9, 0x90, 0xBE, 0xED, 0xC4,
];

pub const MAGIC_LE32: [u8; 16] = [
    0x29, 0xDE, 0x6C, 0xC0, 0xBA, 0xA4, 0x53, 0x2B,
    0x25, 0xF5, 0xB7, 0xA5, 0xF6, 0x66, 0xE2, 0xEE,
];
