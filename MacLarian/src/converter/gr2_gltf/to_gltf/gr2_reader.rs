//! GR2 format structures and parser.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::trivially_copy_pass_by_ref, clippy::needless_range_loop)]

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::{Error, Result};
use crate::formats::gr2::bitknit_decompress as decompress_bitknit;
use super::utils::half_to_f32;

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

// ============================================================================
// Section Header
// ============================================================================

#[derive(Debug, Clone, Copy)]
struct SectionHeader {
    compression: u32,
    offset_in_file: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    relocations_offset: u32,
    num_relocations: u32,
}

// ============================================================================
// Vertex Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum MemberType {
    None = 0,
    Real32 = 10,
    UInt8 = 12,
    NormalUInt8 = 14,
    BinormalInt16 = 17,
    Real16 = 21,
    Unknown(u32),
}

impl MemberType {
    fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::None,
            10 => Self::Real32,
            12 => Self::UInt8,
            14 => Self::NormalUInt8,
            17 => Self::BinormalInt16,
            21 => Self::Real16,
            _ => Self::Unknown(v),
        }
    }

    fn element_size(&self) -> usize {
        match self {
            Self::Real32 => 4,
            Self::Real16 | Self::BinormalInt16 => 2,
            Self::UInt8 | Self::NormalUInt8 => 1,
            _ => 4,
        }
    }
}

#[derive(Debug, Clone)]
struct MemberDef {
    name: String,
    member_type: MemberType,
    array_size: u32,
}

impl MemberDef {
    fn total_size(&self) -> usize {
        self.member_type.element_size() * self.array_size.max(1) as usize
    }
}

#[derive(Debug, Clone)]
struct VertexType {
    members: Vec<MemberDef>,
}

impl VertexType {
    fn stride(&self) -> usize {
        self.members.iter().map(MemberDef::total_size).sum()
    }
}

// ============================================================================
// Parsed Data Structures
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub bone_weights: [u8; 4],
    pub bone_indices: [u8; 4],
    pub qtangent: [i16; 4],
    pub color: [u8; 4],
    pub uv: [f32; 2],
}

pub struct MeshData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub is_32bit_indices: bool,
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale_shear: [f32; 9],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale_shear: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub parent_index: i32,
    pub transform: Transform,
    pub inverse_world_transform: [f32; 16],
}

#[derive(Debug, Clone)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<Bone>,
}

// ============================================================================
// GR2 Reader
// ============================================================================

pub struct Gr2Reader {
    pub data: Vec<u8>,
    pub is_64bit: bool,
    section_offsets: Vec<usize>,
}

impl Gr2Reader {
    /// Create a new GR2 reader from file data.
    ///
    /// # Errors
    /// Returns an error if the data is too small, has an invalid magic signature, or uses unsupported compression.
    ///
    /// # Panics
    /// This function does not panic under normal conditions.
    pub fn new(file_data: &[u8]) -> Result<Self> {
        if file_data.len() < 16 {
            return Err(Error::DecompressionError("GR2 file too small".to_string()));
        }

        let magic: [u8; 16] = file_data[0..16].try_into().unwrap();
        let is_64bit = if magic == MAGIC_LE64 {
            true
        } else if magic == MAGIC_LE32 {
            false
        } else {
            return Err(Error::DecompressionError("Invalid GR2 magic signature".to_string()));
        };

        let mut cursor = std::io::Cursor::new(&file_data[0x20..]);
        let version = cursor.read_u32::<LittleEndian>()?;
        if version != 6 && version != 7 {
            return Err(Error::DecompressionError(format!("Unsupported GR2 version: {version}")));
        }

        cursor.set_position(12);
        let sections_offset = cursor.read_u32::<LittleEndian>()?;
        let num_sections = cursor.read_u32::<LittleEndian>()?;

        let section_header_pos = 0x20 + sections_offset as usize;
        let mut sections = Vec::with_capacity(num_sections as usize);

        for i in 0..num_sections as usize {
            let offset = section_header_pos + i * 44;
            let mut c = std::io::Cursor::new(&file_data[offset..]);

            let compression = c.read_u32::<LittleEndian>()?;
            let offset_in_file = c.read_u32::<LittleEndian>()?;
            let compressed_size = c.read_u32::<LittleEndian>()?;
            let uncompressed_size = c.read_u32::<LittleEndian>()?;
            c.set_position(28);
            let relocations_offset = c.read_u32::<LittleEndian>()?;
            let num_relocations = c.read_u32::<LittleEndian>()?;

            sections.push(SectionHeader {
                compression,
                offset_in_file,
                compressed_size,
                uncompressed_size,
                relocations_offset,
                num_relocations,
            });
        }

        let total_size: usize = sections.iter().map(|s| s.uncompressed_size as usize).sum();
        let mut data = vec![0u8; total_size];
        let mut section_offsets = Vec::with_capacity(sections.len());
        let mut current_offset = 0usize;

        for section in &sections {
            section_offsets.push(current_offset);

            if section.compressed_size == 0 {
                current_offset += section.uncompressed_size as usize;
                continue;
            }

            let start = section.offset_in_file as usize;
            let end = start + section.compressed_size as usize;
            let compressed = &file_data[start..end];

            let decompressed = match section.compression {
                0 => compressed.to_vec(),
                4 => decompress_bitknit(compressed, section.uncompressed_size as usize)?,
                c => return Err(Error::DecompressionError(format!("Unsupported compression: {c}"))),
            };

            let dest_end = current_offset + decompressed.len();
            data[current_offset..dest_end].copy_from_slice(&decompressed);
            current_offset += section.uncompressed_size as usize;
        }

        // Apply relocations
        for (section_idx, section) in sections.iter().enumerate() {
            if section.num_relocations == 0 {
                continue;
            }

            let rel_data = if section.compression == 4 {
                let rel_offset = section.relocations_offset as usize;
                let rel_compressed_size = u32::from_le_bytes([
                    file_data[rel_offset],
                    file_data[rel_offset + 1],
                    file_data[rel_offset + 2],
                    file_data[rel_offset + 3],
                ]) as usize;

                let rel_compressed = &file_data[rel_offset + 4..rel_offset + 4 + rel_compressed_size];
                decompress_bitknit(rel_compressed, section.num_relocations as usize * 12)?
            } else {
                let rel_offset = section.relocations_offset as usize;
                let rel_size = section.num_relocations as usize * 12;
                file_data[rel_offset..rel_offset + rel_size].to_vec()
            };

            for i in 0..section.num_relocations as usize {
                let offset_in_section = u32::from_le_bytes([
                    rel_data[i * 12], rel_data[i * 12 + 1],
                    rel_data[i * 12 + 2], rel_data[i * 12 + 3],
                ]) as usize;

                let target_section = u32::from_le_bytes([
                    rel_data[i * 12 + 4], rel_data[i * 12 + 5],
                    rel_data[i * 12 + 6], rel_data[i * 12 + 7],
                ]) as usize;

                let target_offset = u32::from_le_bytes([
                    rel_data[i * 12 + 8], rel_data[i * 12 + 9],
                    rel_data[i * 12 + 10], rel_data[i * 12 + 11],
                ]) as usize;

                let src_addr = section_offsets[section_idx] + offset_in_section;
                let target_addr = section_offsets[target_section] + target_offset;

                if is_64bit {
                    let bytes = (target_addr as u64).to_le_bytes();
                    data[src_addr..src_addr + 8].copy_from_slice(&bytes);
                } else {
                    let bytes = (target_addr as u32).to_le_bytes();
                    data[src_addr..src_addr + 4].copy_from_slice(&bytes);
                }
            }
        }

        Ok(Self {
            data,
            is_64bit,
            section_offsets,
        })
    }

    fn ptr_size(&self) -> usize {
        if self.is_64bit { 8 } else { 4 }
    }

    fn read_ptr(&self, offset: usize) -> usize {
        if self.is_64bit {
            u64::from_le_bytes(self.data[offset..offset + 8].try_into().unwrap()) as usize
        } else {
            u32::from_le_bytes(self.data[offset..offset + 4].try_into().unwrap()) as usize
        }
    }

    fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.data[offset..offset + 4].try_into().unwrap())
    }

    fn read_u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes(self.data[offset..offset + 2].try_into().unwrap())
    }

    fn read_i16(&self, offset: usize) -> i16 {
        i16::from_le_bytes(self.data[offset..offset + 2].try_into().unwrap())
    }

    fn read_f32(&self, offset: usize) -> f32 {
        f32::from_le_bytes(self.data[offset..offset + 4].try_into().unwrap())
    }

    fn read_f16(&self, offset: usize) -> f32 {
        half_to_f32(self.read_u16(offset))
    }

    fn read_string(&self, offset: usize) -> String {
        if offset == 0 || offset >= self.data.len() {
            return String::new();
        }
        let mut end = offset;
        while end < self.data.len() && self.data[end] != 0 {
            end += 1;
        }
        String::from_utf8_lossy(&self.data[offset..end]).to_string()
    }

    fn read_string_ptr(&self, offset: usize) -> String {
        let ptr = self.read_ptr(offset);
        self.read_string(ptr)
    }

    fn parse_vertex_type(&self, offset: usize) -> VertexType {
        let member_size = if self.is_64bit { 44 } else { 32 };
        let mut members = Vec::new();
        let mut pos = offset;

        for _ in 0..30 {
            if pos + member_size > self.data.len() {
                break;
            }

            let type_val = self.read_u32(pos);
            if type_val == 0 {
                break;
            }

            let name = self.read_string_ptr(pos + 4);
            let array_size_offset = if self.is_64bit { 20 } else { 12 };
            let array_size = self.read_u32(pos + array_size_offset);

            members.push(MemberDef {
                name,
                member_type: MemberType::from_u32(type_val),
                array_size: array_size.max(1),
            });

            pos += member_size;
        }

        VertexType { members }
    }

    fn read_vertex(&self, offset: usize, vertex_type: &VertexType) -> Vertex {
        let mut vertex = Vertex::default();
        let mut pos = offset;

        for member in &vertex_type.members {
            match member.name.as_str() {
                "Position" => {
                    for i in 0..3.min(member.array_size as usize) {
                        vertex.position[i] = self.read_f32(pos + i * 4);
                    }
                }
                "BoneWeights" => {
                    for i in 0..4.min(member.array_size as usize) {
                        vertex.bone_weights[i] = self.data[pos + i];
                    }
                }
                "BoneIndices" => {
                    for i in 0..4.min(member.array_size as usize) {
                        vertex.bone_indices[i] = self.data[pos + i];
                    }
                }
                "QTangent" => {
                    for i in 0..4.min(member.array_size as usize) {
                        vertex.qtangent[i] = self.read_i16(pos + i * 2);
                    }
                }
                "DiffuseColor0" => {
                    for i in 0..4.min(member.array_size as usize) {
                        vertex.color[i] = self.data[pos + i];
                    }
                }
                "TextureCoordinates0" => {
                    match member.member_type {
                        MemberType::Real16 => {
                            vertex.uv[0] = self.read_f16(pos);
                            vertex.uv[1] = self.read_f16(pos + 2);
                        }
                        MemberType::Real32 => {
                            vertex.uv[0] = self.read_f32(pos);
                            vertex.uv[1] = self.read_f32(pos + 4);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            pos += member.total_size();
        }

        vertex
    }

    /// Parse meshes from the GR2 file.
    ///
    /// # Errors
    /// Returns an error if the mesh data cannot be read.
    pub fn parse_meshes(&self, file_data: &[u8]) -> Result<Vec<MeshData>> {
        let mut cursor = std::io::Cursor::new(&file_data[0x20..]);
        cursor.set_position(28);
        let root_section = cursor.read_u32::<LittleEndian>()? as usize;
        let root_offset = cursor.read_u32::<LittleEndian>()? as usize;

        let root_addr = self.section_offsets[root_section] + root_offset;
        let ptr_size = self.ptr_size();
        let array_size = 4 + ptr_size;

        let mut pos = root_addr;
        pos += ptr_size * 3; // Skip first 3 ptrs
        pos += array_size * 4; // Skip 4 arrays
        pos += array_size; // Skip topologies array

        let mesh_count = self.read_u32(pos) as usize;
        let meshes_ptr = self.read_ptr(pos + 4);

        let mut meshes = Vec::with_capacity(mesh_count);

        for i in 0..mesh_count {
            let mesh_ptr = self.read_ptr(meshes_ptr + i * ptr_size);
            if mesh_ptr == 0 || mesh_ptr >= self.data.len() {
                continue;
            }

            let name = self.read_string_ptr(mesh_ptr);
            let vertex_data_ptr = self.read_ptr(mesh_ptr + ptr_size);
            let topology_ptr = self.read_ptr(mesh_ptr + ptr_size * 2 + array_size);

            // Parse vertex data
            let vertices = if vertex_data_ptr > 0 && vertex_data_ptr < self.data.len() {
                let type_ptr = self.read_ptr(vertex_data_ptr);
                let count = self.read_u32(vertex_data_ptr + 8) as usize;
                let data_ptr = self.read_ptr(vertex_data_ptr + 12);

                let vt = if type_ptr > 0 {
                    self.parse_vertex_type(type_ptr)
                } else {
                    VertexType { members: vec![] }
                };

                let stride = vt.stride();
                let mut verts = Vec::with_capacity(count);

                if data_ptr > 0 && stride > 0 {
                    for j in 0..count {
                        let offset = data_ptr + j * stride;
                        if offset + stride <= self.data.len() {
                            verts.push(self.read_vertex(offset, &vt));
                        }
                    }
                }

                verts
            } else {
                Vec::new()
            };

            // Parse topology
            let (indices, is_32bit) = if topology_ptr > 0 && topology_ptr < self.data.len() {
                let idx32_count = self.read_u32(topology_ptr + 12) as usize;
                let idx32_ptr = self.read_ptr(topology_ptr + 16);
                let idx16_count = self.read_u32(topology_ptr + 24) as usize;
                let idx16_ptr = self.read_ptr(topology_ptr + 28);

                if idx32_count > 0 && idx32_ptr > 0 {
                    let mut inds = Vec::with_capacity(idx32_count);
                    for j in 0..idx32_count {
                        let idx_offset = idx32_ptr + j * 4;
                        if idx_offset + 4 <= self.data.len() {
                            inds.push(self.read_u32(idx_offset));
                        }
                    }
                    (inds, true)
                } else if idx16_count > 0 && idx16_ptr > 0 {
                    let mut inds = Vec::with_capacity(idx16_count);
                    for j in 0..idx16_count {
                        let idx_offset = idx16_ptr + j * 2;
                        if idx_offset + 2 <= self.data.len() {
                            inds.push(u32::from(self.read_u16(idx_offset)));
                        }
                    }
                    (inds, false)
                } else {
                    (Vec::new(), false)
                }
            } else {
                (Vec::new(), false)
            };

            if !vertices.is_empty() {
                meshes.push(MeshData {
                    name,
                    vertices,
                    indices,
                    is_32bit_indices: is_32bit,
                });
            }
        }

        Ok(meshes)
    }

    fn read_transform(&self, offset: usize) -> Transform {
        let mut transform = Transform::default();

        for i in 0..3 {
            transform.translation[i] = self.read_f32(offset + 4 + i * 4);
        }
        for i in 0..4 {
            transform.rotation[i] = self.read_f32(offset + 16 + i * 4);
        }
        for i in 0..9 {
            transform.scale_shear[i] = self.read_f32(offset + 32 + i * 4);
        }
        transform
    }

    /// Parse skeleton from the GR2 file.
    ///
    /// # Errors
    /// Returns an error if the skeleton data cannot be read.
    pub fn parse_skeleton(&self, file_data: &[u8]) -> Result<Option<Skeleton>> {
        let mut cursor = std::io::Cursor::new(&file_data[0x20..]);
        cursor.set_position(28);
        let root_section = cursor.read_u32::<LittleEndian>()? as usize;
        let root_offset = cursor.read_u32::<LittleEndian>()? as usize;

        let root_addr = self.section_offsets[root_section] + root_offset;
        let ptr_size = self.ptr_size();
        let array_size = 4 + ptr_size;

        let mut pos = root_addr;
        pos += ptr_size * 3; // Skip first 3 ptrs
        pos += array_size * 2; // Skip TextureInfos and Materials

        // Skeletons array
        let skeleton_count = self.read_u32(pos) as usize;
        let skeletons_ptr = self.read_ptr(pos + 4);

        if skeleton_count == 0 || skeletons_ptr == 0 {
            return Ok(None);
        }

        // Get first skeleton
        let skeleton_ptr = self.read_ptr(skeletons_ptr);
        if skeleton_ptr == 0 || skeleton_ptr >= self.data.len() {
            return Ok(None);
        }

        let name = self.read_string_ptr(skeleton_ptr);
        let bone_count = self.read_u32(skeleton_ptr + ptr_size) as usize;
        let bones_array_ptr = self.read_ptr(skeleton_ptr + ptr_size + 4);

        let mut bones = Vec::with_capacity(bone_count);

        let bone_size = ptr_size + 4 + 68 + 64 + 4 + 2 * ptr_size;

        for i in 0..bone_count {
            let bone_offset = bones_array_ptr + i * bone_size;
            if bone_offset + bone_size > self.data.len() {
                break;
            }

            let bone_name = self.read_string_ptr(bone_offset);
            let parent_index = self.read_u32(bone_offset + ptr_size) as i32;
            let transform = self.read_transform(bone_offset + ptr_size + 4);

            let iwt_offset = bone_offset + ptr_size + 4 + 68;
            let mut inverse_world_transform = [0.0f32; 16];
            for j in 0..16 {
                inverse_world_transform[j] = self.read_f32(iwt_offset + j * 4);
            }

            bones.push(Bone {
                name: bone_name,
                parent_index,
                transform,
                inverse_world_transform,
            });
        }

        Ok(Some(Skeleton { name, bones }))
    }

    /// Get a description of what data the GR2 file contains
    ///
    /// # Errors
    /// Returns an error if the content info cannot be read.
    pub fn get_content_info(&self, file_data: &[u8]) -> Result<Gr2ContentInfo> {
        let mut cursor = std::io::Cursor::new(&file_data[0x20..]);
        cursor.set_position(28);
        let root_section = cursor.read_u32::<LittleEndian>()? as usize;
        let root_offset = cursor.read_u32::<LittleEndian>()? as usize;

        let root_addr = self.section_offsets[root_section] + root_offset;
        let ptr_size = self.ptr_size();
        let array_size = 4 + ptr_size;

        let mut pos = root_addr + ptr_size * 3; // After 3 pointers

        // Read counts for each array type
        let texture_count = self.read_u32(pos) as usize;
        pos += array_size;
        let material_count = self.read_u32(pos) as usize;
        pos += array_size;
        let skeleton_count = self.read_u32(pos) as usize;
        pos += array_size;
        let vertex_data_count = self.read_u32(pos) as usize;
        pos += array_size;
        let topology_count = self.read_u32(pos) as usize;
        pos += array_size;
        let mesh_count = self.read_u32(pos) as usize;
        pos += array_size;
        let model_count = self.read_u32(pos) as usize;

        Ok(Gr2ContentInfo {
            texture_count,
            material_count,
            skeleton_count,
            vertex_data_count,
            topology_count,
            mesh_count,
            model_count,
        })
    }
}

/// Information about what data a GR2 file contains
#[derive(Debug, Clone)]
pub struct Gr2ContentInfo {
    pub texture_count: usize,
    pub material_count: usize,
    pub skeleton_count: usize,
    pub vertex_data_count: usize,
    pub topology_count: usize,
    pub mesh_count: usize,
    pub model_count: usize,
}

impl Gr2ContentInfo {
    /// Returns a human-readable description of the file contents
    #[must_use] 
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if self.skeleton_count > 0 {
            parts.push(format!("{} skeleton(s)", self.skeleton_count));
        }
        if self.mesh_count > 0 {
            parts.push(format!("{} mesh(es)", self.mesh_count));
        }
        if self.model_count > 0 {
            parts.push(format!("{} model(s)", self.model_count));
        }
        if self.material_count > 0 {
            parts.push(format!("{} material(s)", self.material_count));
        }
        if parts.is_empty() {
            "empty".to_string()
        } else {
            parts.join(", ")
        }
    }
}
