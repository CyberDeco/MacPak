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
#[derive(Debug, Clone)]
pub struct Gr2Info {
    pub version: u32,
    pub is_64bit: bool,
    pub file_size: u64,
    pub num_sections: usize,
    pub sections: Vec<SectionInfo>,
}

/// Information about a GR2 section.
#[derive(Debug, Clone)]
pub struct SectionInfo {
    pub index: usize,
    pub compression: String,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
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
    pub name: String,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub has_skeleton: bool,
}

/// Skeleton data extracted from a GR2 file.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Gr2SkeletonInfo {
    pub name: String,
    pub bone_count: usize,
    pub bones: Vec<Gr2BoneInfo>,
}

/// Bone data from a GR2 skeleton.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Gr2BoneInfo {
    pub name: String,
    pub parent_index: i32,
}

/// Complete GR2 model info including meshes and skeleton.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Gr2ModelInfo {
    pub file_path: String,
    pub version: u32,
    pub is_64bit: bool,
    pub skeleton: Option<Gr2SkeletonInfo>,
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
