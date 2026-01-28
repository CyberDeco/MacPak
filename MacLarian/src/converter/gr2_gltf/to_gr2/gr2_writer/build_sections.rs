//! Section building for GR2 files

use std::collections::HashMap;

use crate::error::Result;

use super::constants::{MEMBER_REAL32, MEMBER_UINT8, MEMBER_BINORMAL_INT16, MEMBER_REAL16, MEMBER_REF_TO_ARRAY, MEMBER_STRING, MEMBER_INT32, MEMBER_TRANSFORM, MEMBER_REFERENCE, MEMBER_ARRAY_OF_REFS};
use super::section::Section;
use super::types::{MemberDef, write_type_def};
use super::Gr2Writer;

impl Gr2Writer {
    pub(super) fn build_sections(&self) -> Result<(Vec<Section>, u32, u32)> {
        // Initialize sections
        // Section 0: Main (root object, strings, misc data)
        // Section 1: TrackGroups (animations) - empty for static meshes
        // Section 2: Skeleton data
        // Section 3: Mesh structs
        // Section 4: Type definitions
        // Section 5: Vertex data
        // Section 6: Index data
        let mut sections: Vec<Section> = (0..7).map(|_| Section::new()).collect();

        // Collect all necessary strings
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

        // Write type definitions
        let (vertex_type_offset, root_type_offset) = self.write_type_definitions(&mut sections, &string_offsets);

        // Track offsets
        let mut mesh_offsets: Vec<u32> = Vec::new();
        let mut vertex_data_offsets: Vec<u32> = Vec::new();
        let mut topology_offsets: Vec<u32> = Vec::new();
        let mut skeleton_offsets: Vec<u32> = Vec::new();

        // Write vertex and index data
        let index_offsets = self.write_vertex_and_index_data(&mut sections, &mut vertex_data_offsets);

        // Write skeleton data
        let _bone_array_offset = self.write_skeleton_data(&mut sections, &string_offsets, &mut skeleton_offsets);

        // Write mesh structures
        self.write_mesh_structures(
            &mut sections,
            &string_offsets,
            vertex_type_offset,
            &mut vertex_data_offsets,
            &index_offsets,
            &mut topology_offsets,
            &mut mesh_offsets,
        );

        // Write root object
        let root_offset = self.write_root_object(
            &mut sections,
            &mesh_offsets,
            &skeleton_offsets,
            &vertex_data_offsets,
            &topology_offsets,
        );

        Ok((sections, root_offset, root_type_offset))
    }

    fn write_type_definitions(
        &self,
        sections: &mut [Section],
        string_offsets: &HashMap<&str, u32>,
    ) -> (u32, u32) {
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
        write_type_def(&mut sections[4], &vertex_members, string_offsets);

        // Write topology type definition
        let _topology_type_offset = sections[4].pos();
        let topology_members = [
            MemberDef::new("Groups", MEMBER_REF_TO_ARRAY),
            MemberDef::new("Indices", MEMBER_REF_TO_ARRAY),
            MemberDef::new("Indices16", MEMBER_REF_TO_ARRAY),
        ];
        write_type_def(&mut sections[4], &topology_members, string_offsets);

        // Write vertex data type definition
        let _vertex_data_type_offset = sections[4].pos();
        let vertex_data_members = [
            MemberDef::new("Vertices", MEMBER_REF_TO_ARRAY),
            MemberDef::new("VertexComponentNames", MEMBER_REF_TO_ARRAY),
            MemberDef::new("VertexAnnotationSets", MEMBER_REF_TO_ARRAY),
        ];
        write_type_def(&mut sections[4], &vertex_data_members, string_offsets);

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
        write_type_def(&mut sections[4], &bone_members, string_offsets);

        // Write skeleton type definition
        let _skeleton_type_offset = sections[4].pos();
        let skeleton_members = [
            MemberDef::new("Name", MEMBER_STRING),
            MemberDef::new("Bones", MEMBER_REF_TO_ARRAY),
            MemberDef::new("LODType", MEMBER_INT32),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
        ];
        write_type_def(&mut sections[4], &skeleton_members, string_offsets);

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
        write_type_def(&mut sections[4], &mesh_members, string_offsets);

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
        write_type_def(&mut sections[4], &root_members, string_offsets);

        (vertex_type_offset, root_type_offset)
    }

    fn write_vertex_and_index_data(
        &self,
        sections: &mut [Section],
        vertex_data_offsets: &mut Vec<u32>,
    ) -> Vec<u32> {
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

        // Write indices to section 6 (separate from vertex data)
        let mut index_offsets: Vec<u32> = Vec::new();
        for mesh in &self.meshes {
            sections[6].align(4);
            let index_offset = sections[6].pos();
            // Use 16-bit indices if possible
            let use_16bit = mesh.indices.iter().all(|&i| i <= 65535);
            if use_16bit {
                for &idx in &mesh.indices {
                    sections[6].write_u16(idx as u16);
                }
            } else {
                for &idx in &mesh.indices {
                    sections[6].write_u32(idx);
                }
            }
            index_offsets.push(index_offset);
        }

        index_offsets
    }

    fn write_skeleton_data(
        &self,
        sections: &mut [Section],
        string_offsets: &HashMap<&str, u32>,
        skeleton_offsets: &mut Vec<u32>,
    ) -> u32 {
        // Write bones to section 2 if there's a skeleton
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

        // Write skeleton struct to section 2
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

        bone_array_offset
    }

    #[allow(clippy::too_many_arguments)]
    fn write_mesh_structures(
        &self,
        sections: &mut [Section],
        string_offsets: &HashMap<&str, u32>,
        vertex_type_offset: u32,
        vertex_data_offsets: &mut [u32],
        index_offsets: &[u32],
        topology_offsets: &mut Vec<u32>,
        mesh_offsets: &mut Vec<u32>,
    ) {
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
                sections[3].write_array_ref(mesh.indices.len() as u32, 6, index_offsets[i]);
            } else {
                // Indices (32-bit)
                sections[3].write_array_ref(mesh.indices.len() as u32, 6, index_offsets[i]);
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
    }

    fn write_root_object(
        &self,
        sections: &mut [Section],
        mesh_offsets: &[u32],
        skeleton_offsets: &[u32],
        vertex_data_offsets: &[u32],
        topology_offsets: &[u32],
    ) -> u32 {
        // Write mesh pointer array to section 0
        sections[0].align(8);
        let mesh_ptr_array_offset = sections[0].pos();
        for &offset in mesh_offsets {
            sections[0].write_ptr(3, offset);
        }

        // Write skeleton pointer array to section 0
        let skeleton_ptr_array_offset = if skeleton_offsets.is_empty() {
            0
        } else {
            sections[0].align(8);
            let offset = sections[0].pos();
            for &skel_offset in skeleton_offsets {
                sections[0].write_ptr(2, skel_offset);
            }
            offset
        };

        // Write vertex data pointer array to section 0
        sections[0].align(8);
        let vd_ptr_array_offset = sections[0].pos();
        for &offset in vertex_data_offsets {
            sections[0].write_ptr(3, offset);
        }

        // Write topology pointer array to section 0
        sections[0].align(8);
        let topo_ptr_array_offset = sections[0].pos();
        for &offset in topology_offsets {
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

        root_offset
    }
}
