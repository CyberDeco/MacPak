//! GR2 file writer.
//!
//! Writes meshes and skeletons to GR2 format compatible with
//! Baldur's Gate 3 and Divinity: Original Sin 2.
//!
//! Note: Compression is currently disabled. Files are written uncompressed.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::Result;
use super::gltf_loader::{MeshData, Skeleton};
use super::utils::{crc32, f32_to_half};

// ============================================================================
// Constants
// ============================================================================

/// 64-bit little-endian magic signature
const MAGIC_LE64: [u8; 16] = [
    0xE5, 0x9B, 0x49, 0x5E, 0x6F, 0x63, 0x1F, 0x14,
    0x1E, 0x13, 0xEB, 0xA9, 0x90, 0xBE, 0xED, 0xC4,
];

/// Game tag for BG3/DOS2
const TAG_BG3: u32 = 0xE57F0039;

/// GR2 format version
const VERSION: u32 = 7;

/// Section count (we use 6 sections like typical GR2 files)
const NUM_SECTIONS: u32 = 6;

// Member types
const MEMBER_NONE: u32 = 0;
const MEMBER_INLINE: u32 = 1;
const MEMBER_REFERENCE: u32 = 2;
const MEMBER_REF_TO_ARRAY: u32 = 3;
const MEMBER_ARRAY_OF_REFS: u32 = 4;
const MEMBER_STRING: u32 = 8;
const MEMBER_TRANSFORM: u32 = 9;
const MEMBER_REAL32: u32 = 10;
const MEMBER_INT8: u32 = 11;
const MEMBER_UINT8: u32 = 12;
const MEMBER_BINORMAL_INT16: u32 = 17;
const MEMBER_UINT16: u32 = 16;
const MEMBER_INT32: u32 = 19;
const MEMBER_UINT32: u32 = 20;
const MEMBER_REAL16: u32 = 21;

// ============================================================================
// Section Management
// ============================================================================

/// A fixup (relocation) to be applied.
#[derive(Debug, Clone)]
struct Fixup {
    /// Offset in the source section where the pointer is written
    offset_in_section: u32,
    /// Target section index
    target_section: u32,
    /// Offset within target section
    target_offset: u32,
}

/// A section of data being built.
struct Section {
    data: Vec<u8>,
    fixups: Vec<Fixup>,
}

impl Section {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            fixups: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn pos(&self) -> u32 {
        self.data.len() as u32
    }

    fn align(&mut self, alignment: usize) {
        let padding = (alignment - (self.data.len() % alignment)) % alignment;
        self.data.extend(std::iter::repeat_n(0u8, padding));
    }

    fn write_u8(&mut self, v: u8) {
        self.data.push(v);
    }

    fn write_u16(&mut self, v: u16) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    fn write_i16(&mut self, v: i16) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u32(&mut self, v: u32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    fn write_i32(&mut self, v: i32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u64(&mut self, v: u64) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    fn write_f32(&mut self, v: f32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    fn write_f16(&mut self, v: f32) {
        self.data.extend_from_slice(&f32_to_half(v).to_le_bytes());
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Write a pointer (64-bit) and record a fixup.
    fn write_ptr(&mut self, target_section: u32, target_offset: u32) {
        let offset = self.pos();
        self.fixups.push(Fixup {
            offset_in_section: offset,
            target_section,
            target_offset,
        });
        // Write placeholder (will be resolved later)
        self.write_u64(0);
    }

    /// Write an array reference (count + pointer).
    fn write_array_ref(&mut self, count: u32, target_section: u32, target_offset: u32) {
        self.write_u32(count);
        self.write_ptr(target_section, target_offset);
    }

    /// Write a null pointer.
    fn write_null_ptr(&mut self) {
        self.write_u64(0);
    }

    /// Write a null array reference.
    fn write_null_array(&mut self) {
        self.write_u32(0);
        self.write_u64(0);
    }

    /// Write a string and return its offset.
    fn write_string(&mut self, s: &str) -> u32 {
        let offset = self.pos();
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0); // Null terminator
        offset
    }
}

// ============================================================================
// Type Definitions
// ============================================================================

/// A member definition for struct serialization.
struct MemberDef {
    name: &'static str,
    member_type: u32,
    array_size: u32,
}

impl MemberDef {
    fn new(name: &'static str, member_type: u32) -> Self {
        Self { name, member_type, array_size: 0 }
    }

    fn array(name: &'static str, member_type: u32, size: u32) -> Self {
        Self { name, member_type, array_size: size }
    }
}

/// Write a type definition to a section.
fn write_type_def(section: &mut Section, members: &[MemberDef], string_offsets: &HashMap<&str, u32>) {
    // Member size in 64-bit mode: 44 bytes
    // [type:4][name_ptr:8][def_ptr:8][array_size:4][extra:12][unknown:8]
    for member in members {
        section.write_u32(member.member_type);
        if let Some(&offset) = string_offsets.get(member.name) {
            section.write_ptr(0, offset); // Name string in section 0
        } else {
            section.write_null_ptr();
        }
        section.write_null_ptr(); // Definition pointer (for nested types)
        section.write_u32(member.array_size);
        section.write_u32(0); // Extra[0]
        section.write_u32(0); // Extra[1]
        section.write_u32(0); // Extra[2]
        section.write_u64(0); // Unknown
    }
    // End marker
    section.write_u32(MEMBER_NONE);
    section.write_null_ptr();
    section.write_null_ptr();
    section.write_u32(0);
    section.write_u32(0);
    section.write_u32(0);
    section.write_u32(0);
    section.write_u64(0);
}

// ============================================================================
// GR2 Writer
// ============================================================================

pub struct Gr2Writer {
    meshes: Vec<MeshData>,
    skeleton: Option<Skeleton>,
}

impl Gr2Writer {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            skeleton: None,
        }
    }

    pub fn add_mesh(&mut self, mesh: &MeshData) {
        self.meshes.push(MeshData {
            name: mesh.name.clone(),
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
        });
    }

    pub fn add_skeleton(&mut self, skeleton: &Skeleton) {
        self.skeleton = Some(skeleton.clone());
    }

    /// Build the GR2 file and return as bytes.
    ///
    /// # Errors
    /// Returns an error if building the GR2 data fails.
    pub fn build(&self) -> Result<Vec<u8>> {
        let sections = self.build_sections()?;
        self.build_file_bytes(&sections)
    }

    /// Write the GR2 file to disk.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub fn write(&self, path: &Path) -> Result<()> {
        let data = self.build()?;
        let mut file = File::create(path)?;
        file.write_all(&data)?;
        Ok(())
    }

    fn build_sections(&self) -> Result<(Vec<Section>, u32, u32)> {
        // Initialize sections
        // Section 0: Main (root object, strings, misc data)
        // Section 1: TrackGroups (animations) - empty for now
        // Section 2: Skeleton
        // Section 3: Mesh
        // Section 4: Type definitions
        // Section 5: Vertex data
        let mut sections: Vec<Section> = (0..6).map(|_| Section::new()).collect();

        // Collect all strings we need
        let mut all_strings: Vec<&str> = Vec::new();
        all_strings.push("ArtToolInfo");
        all_strings.push("ExporterInfo");
        all_strings.push("FromFileName");
        all_strings.push("Textures");
        all_strings.push("Materials");
        all_strings.push("Skeletons");
        all_strings.push("VertexDatas");
        all_strings.push("TriTopologies");
        all_strings.push("Meshes");
        all_strings.push("Models");
        all_strings.push("TrackGroups");
        all_strings.push("Animations");
        all_strings.push("ExtendedData");

        // Vertex type member names
        all_strings.push("Position");
        all_strings.push("BoneWeights");
        all_strings.push("BoneIndices");
        all_strings.push("QTangent");
        all_strings.push("DiffuseColor0");
        all_strings.push("TextureCoordinates0");

        // Bone type member names
        all_strings.push("Name");
        all_strings.push("ParentIndex");
        all_strings.push("Transform");
        all_strings.push("InverseWorldTransform");
        all_strings.push("LODError");

        // Skeleton/Mesh member names
        all_strings.push("Bones");
        all_strings.push("LODType");
        all_strings.push("PrimaryVertexData");
        all_strings.push("MorphTargets");
        all_strings.push("PrimaryTopology");
        all_strings.push("MaterialBindings");
        all_strings.push("BoneBindings");

        // Topology member names
        all_strings.push("Groups");
        all_strings.push("Indices");
        all_strings.push("Indices16");

        // VertexData member names
        all_strings.push("Vertices");
        all_strings.push("VertexComponentNames");
        all_strings.push("VertexAnnotationSets");

        // Add mesh and skeleton names
        for mesh in &self.meshes {
            all_strings.push(&mesh.name);
        }
        if let Some(ref skel) = self.skeleton {
            all_strings.push(&skel.name);
            for bone in &skel.bones {
                all_strings.push(&bone.name);
            }
        }

        // Write strings to section 0 and build offset map
        let mut string_offsets: HashMap<&str, u32> = HashMap::new();
        for s in &all_strings {
            if !string_offsets.contains_key(*s) {
                let offset = sections[0].write_string(s);
                string_offsets.insert(*s, offset);
            }
        }
        sections[0].align(8);

        // Write vertex type definition to section 4
        let vertex_type_offset = sections[4].pos();
        let vertex_members = [
            MemberDef::array("Position", MEMBER_REAL32, 3),
            MemberDef::array("BoneWeights", MEMBER_UINT8, 4),
            MemberDef::array("BoneIndices", MEMBER_UINT8, 4),
            MemberDef::array("QTangent", MEMBER_BINORMAL_INT16, 4),
            MemberDef::array("DiffuseColor0", MEMBER_UINT8, 4),
            MemberDef::array("TextureCoordinates0", MEMBER_REAL16, 2),
        ];
        write_type_def(&mut sections[4], &vertex_members, &string_offsets);

        // Write topology type definition
        let _topology_type_offset = sections[4].pos();
        let topology_members = [
            MemberDef::new("Groups", MEMBER_REF_TO_ARRAY),
            MemberDef::new("Indices", MEMBER_REF_TO_ARRAY),
            MemberDef::new("Indices16", MEMBER_REF_TO_ARRAY),
        ];
        write_type_def(&mut sections[4], &topology_members, &string_offsets);

        // Write vertex data type definition
        let _vertex_data_type_offset = sections[4].pos();
        let vertex_data_members = [
            MemberDef::new("Vertices", MEMBER_REF_TO_ARRAY),
            MemberDef::new("VertexComponentNames", MEMBER_REF_TO_ARRAY),
            MemberDef::new("VertexAnnotationSets", MEMBER_REF_TO_ARRAY),
        ];
        write_type_def(&mut sections[4], &vertex_data_members, &string_offsets);

        // Write bone type definition
        let _bone_type_offset = sections[4].pos();
        let bone_members = [
            MemberDef::new("Name", MEMBER_STRING),
            MemberDef::new("ParentIndex", MEMBER_INT32),
            MemberDef::new("Transform", MEMBER_TRANSFORM),
            MemberDef::array("InverseWorldTransform", MEMBER_REAL32, 16),
            MemberDef::new("LODError", MEMBER_REAL32),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
        ];
        write_type_def(&mut sections[4], &bone_members, &string_offsets);

        // Write skeleton type definition
        let _skeleton_type_offset = sections[4].pos();
        let skeleton_members = [
            MemberDef::new("Name", MEMBER_STRING),
            MemberDef::new("Bones", MEMBER_REF_TO_ARRAY),
            MemberDef::new("LODType", MEMBER_INT32),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
        ];
        write_type_def(&mut sections[4], &skeleton_members, &string_offsets);

        // Write mesh type definition
        let _mesh_type_offset = sections[4].pos();
        let mesh_members = [
            MemberDef::new("Name", MEMBER_STRING),
            MemberDef::new("PrimaryVertexData", MEMBER_REFERENCE),
            MemberDef::new("MorphTargets", MEMBER_REF_TO_ARRAY),
            MemberDef::new("PrimaryTopology", MEMBER_REFERENCE),
            MemberDef::new("MaterialBindings", MEMBER_REF_TO_ARRAY),
            MemberDef::new("BoneBindings", MEMBER_REF_TO_ARRAY),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
        ];
        write_type_def(&mut sections[4], &mesh_members, &string_offsets);

        // Write root type definition
        let root_type_offset = sections[4].pos();
        let root_members = [
            MemberDef::new("ArtToolInfo", MEMBER_REFERENCE),
            MemberDef::new("ExporterInfo", MEMBER_REFERENCE),
            MemberDef::new("FromFileName", MEMBER_STRING),
            MemberDef::new("Textures", MEMBER_REF_TO_ARRAY),
            MemberDef::new("Materials", MEMBER_REF_TO_ARRAY),
            MemberDef::new("Skeletons", MEMBER_ARRAY_OF_REFS),
            MemberDef::new("VertexDatas", MEMBER_ARRAY_OF_REFS),
            MemberDef::new("TriTopologies", MEMBER_ARRAY_OF_REFS),
            MemberDef::new("Meshes", MEMBER_ARRAY_OF_REFS),
            MemberDef::new("Models", MEMBER_ARRAY_OF_REFS),
            MemberDef::new("TrackGroups", MEMBER_ARRAY_OF_REFS),
            MemberDef::new("Animations", MEMBER_ARRAY_OF_REFS),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
        ];
        write_type_def(&mut sections[4], &root_members, &string_offsets);

        // Track offsets for data we write
        let mut mesh_offsets: Vec<u32> = Vec::new();
        let mut vertex_data_offsets: Vec<u32> = Vec::new();
        let mut topology_offsets: Vec<u32> = Vec::new();
        let mut skeleton_offsets: Vec<u32> = Vec::new();

        // Write vertex data to section 5
        for mesh in &self.meshes {
            let vertex_data_offset = sections[5].pos();
            for v in &mesh.vertices {
                // Position: 3 x f32 = 12 bytes
                sections[5].write_f32(v.position[0]);
                sections[5].write_f32(v.position[1]);
                sections[5].write_f32(v.position[2]);
                // BoneWeights: 4 x u8 = 4 bytes
                for &w in &v.bone_weights {
                    sections[5].write_u8(w);
                }
                // BoneIndices: 4 x u8 = 4 bytes
                for &i in &v.bone_indices {
                    sections[5].write_u8(i);
                }
                // QTangent: 4 x i16 = 8 bytes
                for &q in &v.qtangent {
                    sections[5].write_i16(q);
                }
                // DiffuseColor0: 4 x u8 = 4 bytes
                for &c in &v.color {
                    sections[5].write_u8(c);
                }
                // TextureCoordinates0: 2 x f16 = 4 bytes
                sections[5].write_f16(v.uv[0]);
                sections[5].write_f16(v.uv[1]);
            }
            vertex_data_offsets.push(vertex_data_offset);
        }

        // Write indices to section 5
        let mut index_offsets: Vec<u32> = Vec::new();
        for mesh in &self.meshes {
            sections[5].align(4);
            let index_offset = sections[5].pos();
            // Use 16-bit indices if possible
            let use_16bit = mesh.indices.iter().all(|&i| i <= 65535);
            if use_16bit {
                for &idx in &mesh.indices {
                    sections[5].write_u16(idx as u16);
                }
            } else {
                for &idx in &mesh.indices {
                    sections[5].write_u32(idx);
                }
            }
            index_offsets.push(index_offset);
        }

        // Write bones to section 2 if we have a skeleton
        let mut bone_array_offset = 0u32;
        if let Some(ref skel) = self.skeleton {
            sections[2].align(8);
            bone_array_offset = sections[2].pos();

            for bone in &skel.bones {
                // Name string pointer
                if let Some(&str_offset) = string_offsets.get(bone.name.as_str()) {
                    sections[2].write_ptr(0, str_offset);
                } else {
                    sections[2].write_null_ptr();
                }

                // Parent index
                sections[2].write_i32(bone.parent_index);

                // Transform (68 bytes)
                // Flags (4 bytes) - 0x1FF means all components present
                sections[2].write_u32(0x1FF);
                // Translation (12 bytes)
                for &t in &bone.transform.translation {
                    sections[2].write_f32(t);
                }
                // Rotation quaternion (16 bytes)
                for &r in &bone.transform.rotation {
                    sections[2].write_f32(r);
                }
                // Scale/shear matrix (36 bytes)
                for &s in &bone.transform.scale_shear {
                    sections[2].write_f32(s);
                }

                // Inverse world transform (64 bytes)
                for &m in &bone.inverse_world_transform {
                    sections[2].write_f32(m);
                }

                // LODError (4 bytes)
                sections[2].write_f32(0.0);

                // ExtendedData (16 bytes for variant reference in 64-bit)
                sections[2].write_null_ptr();
                sections[2].write_null_ptr();
            }
        }

        // Write skeleton to section 2
        if let Some(ref skel) = self.skeleton {
            sections[2].align(8);
            let skel_offset = sections[2].pos();

            // Name
            if let Some(&str_offset) = string_offsets.get(skel.name.as_str()) {
                sections[2].write_ptr(0, str_offset);
            } else {
                sections[2].write_null_ptr();
            }

            // Bones array
            sections[2].write_array_ref(skel.bones.len() as u32, 2, bone_array_offset);

            // LODType
            sections[2].write_i32(0);

            // ExtendedData
            sections[2].write_null_ptr();
            sections[2].write_null_ptr();

            skeleton_offsets.push(skel_offset);
        }

        // Write VertexData structs to section 3
        for (i, mesh) in self.meshes.iter().enumerate() {
            sections[3].align(8);
            let vd_struct_offset = sections[3].pos();

            // Vertices array ref (type + count + data)
            sections[3].write_ptr(4, vertex_type_offset); // Type pointer
            sections[3].write_u32(mesh.vertices.len() as u32);
            sections[3].write_ptr(5, vertex_data_offsets[i]);

            // VertexComponentNames (empty)
            sections[3].write_null_array();

            // VertexAnnotationSets (empty)
            sections[3].write_null_array();

            // Store for mesh reference
            vertex_data_offsets[i] = vd_struct_offset;
        }

        // Write TriTopology structs to section 3
        for (i, mesh) in self.meshes.iter().enumerate() {
            sections[3].align(8);
            let topo_offset = sections[3].pos();

            // Groups (empty)
            sections[3].write_null_array();

            // Determine if 16-bit or 32-bit indices
            let use_16bit = mesh.indices.iter().all(|&idx| idx <= 65535);

            if use_16bit {
                // Indices (32-bit, empty)
                sections[3].write_null_array();
                // Indices16
                sections[3].write_array_ref(mesh.indices.len() as u32, 5, index_offsets[i]);
            } else {
                // Indices (32-bit)
                sections[3].write_array_ref(mesh.indices.len() as u32, 5, index_offsets[i]);
                // Indices16 (empty)
                sections[3].write_null_array();
            }

            topology_offsets.push(topo_offset);
        }

        // Write Mesh structs to section 3
        for (i, mesh) in self.meshes.iter().enumerate() {
            sections[3].align(8);
            let mesh_offset = sections[3].pos();

            // Name
            if let Some(&str_offset) = string_offsets.get(mesh.name.as_str()) {
                sections[3].write_ptr(0, str_offset);
            } else {
                sections[3].write_null_ptr();
            }

            // PrimaryVertexData
            sections[3].write_ptr(3, vertex_data_offsets[i]);

            // MorphTargets (empty)
            sections[3].write_null_array();

            // PrimaryTopology
            sections[3].write_ptr(3, topology_offsets[i]);

            // MaterialBindings (empty)
            sections[3].write_null_array();

            // BoneBindings (empty for now)
            sections[3].write_null_array();

            // ExtendedData
            sections[3].write_null_ptr();
            sections[3].write_null_ptr();

            mesh_offsets.push(mesh_offset);
        }

        // Write mesh pointer array to section 0
        sections[0].align(8);
        let mesh_ptr_array_offset = sections[0].pos();
        for &offset in &mesh_offsets {
            sections[0].write_ptr(3, offset);
        }

        // Write skeleton pointer array to section 0
        let skeleton_ptr_array_offset = if skeleton_offsets.is_empty() {
            0
        } else {
            sections[0].align(8);
            let offset = sections[0].pos();
            for &skel_offset in &skeleton_offsets {
                sections[0].write_ptr(2, skel_offset);
            }
            offset
        };

        // Write vertex data pointer array to section 0
        sections[0].align(8);
        let vd_ptr_array_offset = sections[0].pos();
        for &offset in &vertex_data_offsets {
            sections[0].write_ptr(3, offset);
        }

        // Write topology pointer array to section 0
        sections[0].align(8);
        let topo_ptr_array_offset = sections[0].pos();
        for &offset in &topology_offsets {
            sections[0].write_ptr(3, offset);
        }

        // Write root object to section 0
        sections[0].align(8);
        let root_offset = sections[0].pos();

        // ArtToolInfo (null)
        sections[0].write_null_ptr();
        // ExporterInfo (null)
        sections[0].write_null_ptr();
        // FromFileName (null)
        sections[0].write_null_ptr();
        // Textures (empty)
        sections[0].write_null_array();
        // Materials (empty)
        sections[0].write_null_array();
        // Skeletons
        if skeleton_offsets.is_empty() {
            sections[0].write_null_array();
        } else {
            sections[0].write_array_ref(skeleton_offsets.len() as u32, 0, skeleton_ptr_array_offset);
        }
        // VertexDatas
        sections[0].write_array_ref(self.meshes.len() as u32, 0, vd_ptr_array_offset);
        // TriTopologies
        sections[0].write_array_ref(self.meshes.len() as u32, 0, topo_ptr_array_offset);
        // Meshes
        sections[0].write_array_ref(self.meshes.len() as u32, 0, mesh_ptr_array_offset);
        // Models (empty)
        sections[0].write_null_array();
        // TrackGroups (empty)
        sections[0].write_null_array();
        // Animations (empty)
        sections[0].write_null_array();
        // ExtendedData (null)
        sections[0].write_null_ptr();
        sections[0].write_null_ptr();

        Ok((sections, root_offset, root_type_offset))
    }

    fn build_file_bytes(&self, sections_data: &(Vec<Section>, u32, u32)) -> Result<Vec<u8>> {
        let (sections, root_offset, root_type_offset) = sections_data;

        // Calculate offsets - NO COMPRESSION, write uncompressed
        let magic_size = 32;
        let header_size = 72; // v7 header
        let section_header_size = 44 * NUM_SECTIONS as usize;
        let headers_total = magic_size + header_size + section_header_size;

        // Align to 16 bytes
        let data_start = (headers_total + 15) & !15;

        // Calculate section offsets (uncompressed)
        let mut section_offsets = Vec::new();
        let mut current_offset = data_start;
        for section in sections {
            section_offsets.push(current_offset);
            current_offset += section.len();
            // Align each section to 4 bytes
            current_offset = (current_offset + 3) & !3;
        }

        // Calculate relocation table offsets (uncompressed)
        let mut reloc_offsets = Vec::new();
        for section in sections {
            reloc_offsets.push(current_offset);
            if !section.fixups.is_empty() {
                current_offset += section.fixups.len() * 12;
            }
        }

        let file_size = current_offset;

        // Build output buffer
        let mut output = Vec::with_capacity(file_size);

        // Write magic block (32 bytes)
        output.extend_from_slice(&MAGIC_LE64);
        output.extend_from_slice(&(data_start as u32).to_le_bytes());
        output.extend_from_slice(&0u32.to_le_bytes()); // header_format (uncompressed)
        output.extend_from_slice(&[0u8; 8]);

        // Write header (72 bytes for v7)
        output.extend_from_slice(&VERSION.to_le_bytes());
        output.extend_from_slice(&(file_size as u32).to_le_bytes());
        output.extend_from_slice(&0u32.to_le_bytes()); // CRC placeholder
        output.extend_from_slice(&(header_size as u32).to_le_bytes());
        output.extend_from_slice(&NUM_SECTIONS.to_le_bytes());
        // root_type reference
        output.extend_from_slice(&4u32.to_le_bytes()); // section 4
        output.extend_from_slice(&root_type_offset.to_le_bytes());
        // root_node reference
        output.extend_from_slice(&0u32.to_le_bytes()); // section 0
        output.extend_from_slice(&root_offset.to_le_bytes());
        // tag
        output.extend_from_slice(&TAG_BG3.to_le_bytes());
        // extra_tags
        output.extend_from_slice(&[0u8; 16]);
        // string_table_crc
        output.extend_from_slice(&0u32.to_le_bytes());
        // reserved (12 bytes)
        output.extend_from_slice(&[0u8; 12]);

        // Write section headers (uncompressed format)
        for (i, section) in sections.iter().enumerate() {
            output.extend_from_slice(&0u32.to_le_bytes()); // compression = 0 (none)
            output.extend_from_slice(&(section_offsets[i] as u32).to_le_bytes());
            output.extend_from_slice(&(section.len() as u32).to_le_bytes()); // compressed = uncompressed
            output.extend_from_slice(&(section.len() as u32).to_le_bytes());
            output.extend_from_slice(&4u32.to_le_bytes()); // alignment
            output.extend_from_slice(&0u32.to_le_bytes()); // first_16bit
            output.extend_from_slice(&0u32.to_le_bytes()); // first_8bit
            output.extend_from_slice(&(reloc_offsets[i] as u32).to_le_bytes());
            output.extend_from_slice(&(section.fixups.len() as u32).to_le_bytes());
            output.extend_from_slice(&0u32.to_le_bytes()); // mixed_marshalling_offset
            output.extend_from_slice(&0u32.to_le_bytes()); // num_mixed_marshalling
        }

        // Pad to data_start
        while output.len() < data_start {
            output.push(0);
        }

        // Write section data (uncompressed)
        for (i, section) in sections.iter().enumerate() {
            while output.len() < section_offsets[i] {
                output.push(0);
            }
            output.extend_from_slice(&section.data);
            // Align to 4 bytes
            while output.len() % 4 != 0 {
                output.push(0);
            }
        }

        // Write relocation tables (uncompressed)
        for (i, section) in sections.iter().enumerate() {
            if !section.fixups.is_empty() {
                while output.len() < reloc_offsets[i] {
                    output.push(0);
                }
                for fixup in &section.fixups {
                    output.extend_from_slice(&fixup.offset_in_section.to_le_bytes());
                    output.extend_from_slice(&fixup.target_section.to_le_bytes());
                    output.extend_from_slice(&fixup.target_offset.to_le_bytes());
                }
            }
        }

        // Calculate and update CRC
        let crc = crc32(&output[0x20 + 8..]);
        output[0x20 + 8..0x20 + 12].copy_from_slice(&crc.to_le_bytes());

        Ok(output)
    }
}

impl Default for Gr2Writer {
    fn default() -> Self {
        Self::new()
    }
}
