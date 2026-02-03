//! GR2 file inspection utilities
//!
//! Provides functions to inspect GR2 file structure and extract metadata.
//!
//!

#![allow(clippy::cast_possible_truncation)]

use super::{Compression, Gr2File, PointerSize};
use crate::converter::gr2_gltf::to_gltf::Gr2Reader;
use crate::error::Result;
use std::path::Path;

/// Information about a GR2 file.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Gr2Info {
    /// GR2 format version number.
    pub version: u32,
    /// Whether the file uses 64-bit pointers.
    pub is_64bit: bool,
    /// Total file size in bytes.
    pub file_size: u64,
    /// Number of data sections in the file.
    pub num_sections: usize,
    /// Information about each section.
    pub sections: Vec<SectionInfo>,
}

/// Information about a GR2 section.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SectionInfo {
    /// Section index (0-based).
    pub index: usize,
    /// Compression method name (e.g., "`BitKnit`", "`None`").
    pub compression: String,
    /// Size of compressed data in bytes.
    pub compressed_size: u32,
    /// Size of uncompressed data in bytes.
    pub uncompressed_size: u32,
    /// Compression ratio (compressed/uncompressed), if compressed.
    pub compression_ratio: Option<f64>,
}

/// Get information about a GR2 file structure.
///
/// # Errors
/// Returns an error if the file cannot be read or has an invalid format.
pub fn inspect_gr2<P: AsRef<Path>>(source: P) -> Result<Gr2Info> {
    let data = std::fs::read(source.as_ref())?;
    let file_size = data.len() as u64;
    let gr2 = Gr2File::from_bytes(&data)?;

    let is_64bit = matches!(gr2.pointer_size()?, PointerSize::Bit64);

    let sections: Vec<SectionInfo> = gr2
        .sections
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let compression = match s.compression {
                Compression::None => "None".to_string(),
                Compression::Oodle0 => "Oodle0".to_string(),
                Compression::Oodle1 => "Oodle1".to_string(),
                Compression::BitKnit => "BitKnit".to_string(),
            };
            SectionInfo {
                index: i,
                compression,
                compressed_size: s.compressed_size,
                uncompressed_size: s.uncompressed_size,
                compression_ratio: s.compression_ratio(),
            }
        })
        .collect();

    Ok(Gr2Info {
        version: gr2.header.version,
        is_64bit,
        file_size,
        num_sections: sections.len(),
        sections,
    })
}

/// Mesh data extracted from a GR2 file.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Gr2MeshInfo {
    /// Mesh name from the GR2 file.
    pub name: String,
    /// Number of vertices in the mesh.
    pub vertex_count: usize,
    /// Number of triangles in the mesh.
    pub triangle_count: usize,
    /// Whether the mesh has an associated skeleton.
    pub has_skeleton: bool,
}

/// Skeleton data extracted from a GR2 file.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Gr2SkeletonInfo {
    /// Skeleton name from the GR2 file.
    pub name: String,
    /// Number of bones in the skeleton.
    pub bone_count: usize,
    /// Information about each bone.
    pub bones: Vec<Gr2BoneInfo>,
}

/// Bone data from a GR2 skeleton.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Gr2BoneInfo {
    /// Bone name.
    pub name: String,
    /// Index of the parent bone (-1 for root bones).
    pub parent_index: i32,
}

/// Complete GR2 model info including meshes and skeleton.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Gr2ModelInfo {
    /// Path to the source GR2 file.
    pub file_path: String,
    /// GR2 format version number.
    pub version: u32,
    /// Whether the file uses 64-bit pointers.
    pub is_64bit: bool,
    /// Skeleton data, if present.
    pub skeleton: Option<Gr2SkeletonInfo>,
    /// List of meshes in the file.
    pub meshes: Vec<Gr2MeshInfo>,
}

/// Extract mesh and skeleton information from a GR2 file.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn extract_gr2_info<P: AsRef<Path>>(source: P) -> Result<Gr2ModelInfo> {
    let source_path = source.as_ref();
    let data = std::fs::read(source_path)?;
    let reader = Gr2Reader::new(&data)?;

    let skeleton_info = reader.parse_skeleton(&data)?.map(|skel| Gr2SkeletonInfo {
        name: skel.name,
        bone_count: skel.bones.len(),
        bones: skel
            .bones
            .iter()
            .map(|b| Gr2BoneInfo {
                name: b.name.clone(),
                parent_index: b.parent_index,
            })
            .collect(),
    });

    let meshes = reader.parse_meshes(&data)?;
    let mesh_infos: Vec<Gr2MeshInfo> = meshes
        .iter()
        .map(|m| Gr2MeshInfo {
            name: m.name.clone(),
            vertex_count: m.vertices.len(),
            triangle_count: m.indices.len() / 3,
            has_skeleton: skeleton_info.is_some(),
        })
        .collect();

    Ok(Gr2ModelInfo {
        file_path: source_path.display().to_string(),
        version: 7, // GR2 version 7 is standard for BG3
        is_64bit: reader.is_64bit,
        skeleton: skeleton_info,
        meshes: mesh_infos,
    })
}
