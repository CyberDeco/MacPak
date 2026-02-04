//! Section building for GR2 files

use std::collections::HashMap;

use crate::error::Result;

use super::Gr2Writer;
use super::constants::{
    MEMBER_ARRAY_OF_REFS, MEMBER_BINORMAL_INT16, MEMBER_INT32, MEMBER_REAL16, MEMBER_REAL32,
    MEMBER_REF_TO_ARRAY, MEMBER_REFERENCE, MEMBER_STRING, MEMBER_TRANSFORM, MEMBER_UINT32,
    MEMBER_UINT8,
};
use super::section::Section;
use super::types::{MemberDef, write_type_def};

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
        let mut all_strings: Vec<String> = Vec::new();
        let static_strings = [
            "ArtToolInfo", "ExporterInfo", "FromFileName", "Textures", "Materials",
            "Skeletons", "VertexDatas", "TriTopologies", "Meshes", "Models",
            "TrackGroups", "Animations", "ExtendedData",
            // Vertex type member names
            "Position", "BoneWeights", "BoneIndices", "QTangent", "DiffuseColor0",
            "TextureCoordinates0",
            // Bone type member names
            "Name", "ParentIndex", "Transform", "InverseWorldTransform", "LODError",
            // Skeleton/Mesh member names
            "Bones", "LODType", "PrimaryVertexData", "MorphTargets", "PrimaryTopology",
            "MaterialBindings", "BoneBindings",
            // Mesh ExtendedData member names
            "MeshProxy", "Rigid", "Cloth", "Spring", "Occluder", "LOD",
            "UserDefinedProperties", "UserMeshProperties", "LSMVersion",
            // MeshPropertySet member names
            "Flags", "Lod", "FormatDescs", "LodDistance", "IsImpostor",
            // Topology member names
            "Groups", "Indices", "Indices16",
            // VertexData member names
            "Vertices", "VertexComponentNames", "VertexAnnotationSets",
            // TopologyGroup member names
            "MaterialIndex", "TriFirst", "TriCount",
            // BoneBinding member names
            "BoneName", "OBBMin", "OBBMax", "TriangleCount", "TriangleIndices",
            // Model member names
            "Skeleton", "MeshBindings", "InitialPlacement",
        ];
        for s in &static_strings {
            all_strings.push((*s).to_string());
        }

        // Add mesh and skeleton names
        for mesh in &self.meshes {
            all_strings.push(mesh.name.clone());
            // Add material binding names
            for mat_name in &mesh.material_binding_names {
                all_strings.push(mat_name.clone());
            }
            // Add bone binding bone names
            for bb in &mesh.bone_bindings {
                all_strings.push(bb.bone_name.clone());
            }
        }
        if let Some(ref skel) = self.skeleton {
            all_strings.push(skel.name.clone());
            for bone in &skel.bones {
                all_strings.push(bone.name.clone());
            }
        }
        if let Some(ref model) = self.model {
            all_strings.push(model.name.clone());
        }

        // Add user defined properties as strings
        for mesh in &self.meshes {
            if let Some(ref udp) = mesh.user_defined_properties {
                all_strings.push(udp.clone());
            }
        }

        // Write strings to section 0 and build offset map
        let mut string_offsets: HashMap<String, u32> = HashMap::new();
        for s in &all_strings {
            if !string_offsets.contains_key(s) {
                let offset = sections[0].write_string(s);
                string_offsets.insert(s.clone(), offset);
            }
        }
        sections[0].align(8);

        // Write type definitions
        let type_offsets = self.write_type_definitions(&mut sections, &string_offsets);

        // Track offsets
        let mut mesh_offsets: Vec<u32> = Vec::new();
        let mut vertex_data_offsets: Vec<u32> = Vec::new();
        let mut topology_offsets: Vec<u32> = Vec::new();
        let mut skeleton_offsets: Vec<u32> = Vec::new();

        // Write vertex and index data
        let index_offsets =
            self.write_vertex_and_index_data(&mut sections, &mut vertex_data_offsets);

        // Write skeleton data
        let _bone_array_offset =
            self.write_skeleton_data(&mut sections, &string_offsets, &mut skeleton_offsets);

        // Write mesh structures
        self.write_mesh_structures(
            &mut sections,
            &string_offsets,
            &type_offsets,
            &mut vertex_data_offsets,
            &index_offsets,
            &mut topology_offsets,
            &mut mesh_offsets,
        );

        // Write root object
        let root_offset = self.write_root_object(
            &mut sections,
            &string_offsets,
            &type_offsets,
            &mesh_offsets,
            &skeleton_offsets,
            &vertex_data_offsets,
            &topology_offsets,
        );

        Ok((sections, root_offset, type_offsets.root_type))
    }

    /// Returns all type definition offsets needed by other methods.
    fn write_type_definitions(
        &self,
        sections: &mut [Section],
        string_offsets: &HashMap<String, u32>,
    ) -> TypeOffsets {
        // Write vertex type definition to section 4
        let vertex_type = sections[4].pos();
        let vertex_members = [
            MemberDef::array("Position", MEMBER_REAL32, 3),
            MemberDef::array("BoneWeights", MEMBER_UINT8, 4),
            MemberDef::array("BoneIndices", MEMBER_UINT8, 4),
            MemberDef::array("QTangent", MEMBER_BINORMAL_INT16, 4),
            MemberDef::array("DiffuseColor0", MEMBER_UINT8, 4),
            MemberDef::array("TextureCoordinates0", MEMBER_REAL16, 2),
        ];
        write_type_def(&mut sections[4], &vertex_members, string_offsets);

        // Write TopologyGroup type definition
        let topology_group_type = sections[4].pos();
        let tg_members = [
            MemberDef::new("MaterialIndex", MEMBER_INT32),
            MemberDef::new("TriFirst", MEMBER_INT32),
            MemberDef::new("TriCount", MEMBER_INT32),
        ];
        write_type_def(&mut sections[4], &tg_members, string_offsets);

        // Write topology type definition
        let _topology_type = sections[4].pos();
        let topology_members = [
            MemberDef::with_def("Groups", MEMBER_REF_TO_ARRAY, topology_group_type),
            MemberDef::new("Indices", MEMBER_REF_TO_ARRAY),
            MemberDef::new("Indices16", MEMBER_REF_TO_ARRAY),
        ];
        write_type_def(&mut sections[4], &topology_members, string_offsets);

        // Write vertex data type definition
        let _vertex_data_type = sections[4].pos();
        let vertex_data_members = [
            MemberDef::new("Vertices", MEMBER_REF_TO_ARRAY),
            MemberDef::new("VertexComponentNames", MEMBER_REF_TO_ARRAY),
            MemberDef::new("VertexAnnotationSets", MEMBER_REF_TO_ARRAY),
        ];
        write_type_def(&mut sections[4], &vertex_data_members, string_offsets);

        // Write BoneBinding type definition
        let bone_binding_type = sections[4].pos();
        let bb_members = [
            MemberDef::new("BoneName", MEMBER_STRING),
            MemberDef::array("OBBMin", MEMBER_REAL32, 3),
            MemberDef::array("OBBMax", MEMBER_REAL32, 3),
            MemberDef::new("TriangleCount", MEMBER_INT32),
            MemberDef::new("TriangleIndices", MEMBER_REF_TO_ARRAY),
        ];
        write_type_def(&mut sections[4], &bb_members, string_offsets);

        // Write Material type definition (minimal: just Name)
        let material_type = sections[4].pos();
        let mat_members = [
            MemberDef::new("Name", MEMBER_STRING),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
        ];
        write_type_def(&mut sections[4], &mat_members, string_offsets);

        // Write bone type definition
        let _bone_type = sections[4].pos();
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
        let _skeleton_type = sections[4].pos();
        let skeleton_members = [
            MemberDef::new("Name", MEMBER_STRING),
            MemberDef::new("Bones", MEMBER_REF_TO_ARRAY),
            MemberDef::new("LODType", MEMBER_INT32),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
        ];
        write_type_def(&mut sections[4], &skeleton_members, string_offsets);

        // Write mesh type definition
        let _mesh_type = sections[4].pos();
        let mesh_members = [
            MemberDef::new("Name", MEMBER_STRING),
            MemberDef::new("PrimaryVertexData", MEMBER_REFERENCE),
            MemberDef::new("MorphTargets", MEMBER_REF_TO_ARRAY),
            MemberDef::new("PrimaryTopology", MEMBER_REFERENCE),
            MemberDef::with_def("MaterialBindings", MEMBER_REF_TO_ARRAY, material_type),
            MemberDef::with_def("BoneBindings", MEMBER_REF_TO_ARRAY, bone_binding_type),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
        ];
        write_type_def(&mut sections[4], &mesh_members, string_offsets);

        // Write Model type definition
        let model_type = sections[4].pos();
        let model_members = [
            MemberDef::new("Name", MEMBER_STRING),
            MemberDef::new("Skeleton", MEMBER_REFERENCE),
            MemberDef::new("MeshBindings", MEMBER_ARRAY_OF_REFS),
            MemberDef::new("InitialPlacement", MEMBER_TRANSFORM),
        ];
        write_type_def(&mut sections[4], &model_members, string_offsets);

        // Write MeshPropertySet type definition
        let mesh_props_type = sections[4].pos();
        let mesh_props_members = [
            MemberDef::array("Flags", MEMBER_UINT32, 4),
            MemberDef::array("Lod", MEMBER_INT32, 1),
            MemberDef::new("FormatDescs", MEMBER_REFERENCE),
            MemberDef::new("ExtendedData", MEMBER_REFERENCE),
            MemberDef::array("LodDistance", MEMBER_REAL32, 1),
            MemberDef::array("IsImpostor", MEMBER_INT32, 1),
        ];
        write_type_def(&mut sections[4], &mesh_props_members, string_offsets);

        // Write mesh ExtendedData type definition
        let ext_data_type = sections[4].pos();
        let ext_data_members = [
            MemberDef::new("MeshProxy", MEMBER_INT32),
            MemberDef::new("Rigid", MEMBER_INT32),
            MemberDef::new("Cloth", MEMBER_INT32),
            MemberDef::new("Spring", MEMBER_INT32),
            MemberDef::new("Occluder", MEMBER_INT32),
            MemberDef::new("LOD", MEMBER_INT32),
            MemberDef::new("UserDefinedProperties", MEMBER_STRING),
            MemberDef::with_def("UserMeshProperties", MEMBER_REFERENCE, mesh_props_type),
            MemberDef::new("LSMVersion", MEMBER_INT32),
        ];
        write_type_def(&mut sections[4], &ext_data_members, string_offsets);

        // Write root type definition
        let root_type = sections[4].pos();
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

        TypeOffsets {
            vertex_type,
            ext_data_type,
            _mesh_props_type: mesh_props_type,
            root_type,
            _model_type: model_type,
        }
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
        string_offsets: &HashMap<String, u32>,
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

                // LODError (4 bytes) - now using actual value
                sections[2].write_f32(bone.lod_error);

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

            // LODType - now using actual value
            sections[2].write_i32(skel.lod_type);

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
        string_offsets: &HashMap<String, u32>,
        type_offsets: &TypeOffsets,
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
            sections[3].write_ptr(4, type_offsets.vertex_type); // Type pointer
            sections[3].write_u32(mesh.vertices.len() as u32);
            sections[3].write_ptr(5, vertex_data_offsets[i]);

            // VertexComponentNames (empty)
            sections[3].write_null_array();

            // VertexAnnotationSets (empty)
            sections[3].write_null_array();

            // Store for mesh reference
            vertex_data_offsets[i] = vd_struct_offset;
        }

        // Write TopologyGroup data to section 3 (one array per mesh)
        let mut topo_group_offsets: Vec<Option<(u32, usize)>> = Vec::new(); // (offset, count)
        for mesh in &self.meshes {
            if mesh.topology_groups.is_empty() {
                topo_group_offsets.push(None);
            } else {
                sections[3].align(4);
                let group_data_offset = sections[3].pos();
                for tg in &mesh.topology_groups {
                    sections[3].write_i32(tg.material_index);
                    sections[3].write_i32(tg.tri_first);
                    sections[3].write_i32(tg.tri_count);
                }
                topo_group_offsets.push(Some((group_data_offset, mesh.topology_groups.len())));
            }
        }

        // Write TriTopology structs to section 3
        for (i, mesh) in self.meshes.iter().enumerate() {
            sections[3].align(8);
            let topo_offset = sections[3].pos();

            // Groups
            if let Some((group_offset, group_count)) = topo_group_offsets[i] {
                sections[3].write_array_ref(group_count as u32, 3, group_offset);
            } else {
                sections[3].write_null_array();
            }

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

        // Write BoneBinding data to section 3 (one array per mesh)
        let mut bone_binding_offsets: Vec<Option<(u32, usize)>> = Vec::new();
        for mesh in &self.meshes {
            if mesh.bone_bindings.is_empty() {
                bone_binding_offsets.push(None);
            } else {
                // First, write TriangleIndices arrays for each bone binding
                let mut tri_idx_offsets: Vec<Option<(u32, usize)>> = Vec::new();
                for bb in &mesh.bone_bindings {
                    if bb.tri_indices.is_empty() {
                        tri_idx_offsets.push(None);
                    } else {
                        sections[3].align(4);
                        let tri_offset = sections[3].pos();
                        for &idx in &bb.tri_indices {
                            sections[3].write_i32(idx);
                        }
                        tri_idx_offsets.push(Some((tri_offset, bb.tri_indices.len())));
                    }
                }

                // Now write BoneBinding structs
                sections[3].align(8);
                let bb_array_offset = sections[3].pos();
                for (j, bb) in mesh.bone_bindings.iter().enumerate() {
                    // BoneName (string ptr)
                    if let Some(&str_offset) = string_offsets.get(bb.bone_name.as_str()) {
                        sections[3].write_ptr(0, str_offset);
                    } else {
                        sections[3].write_null_ptr();
                    }
                    // OBBMin (3 x f32)
                    for &v in &bb.obb_min {
                        sections[3].write_f32(v);
                    }
                    // OBBMax (3 x f32)
                    for &v in &bb.obb_max {
                        sections[3].write_f32(v);
                    }
                    // TriangleCount (i32)
                    sections[3].write_i32(bb.tri_count);
                    // TriangleIndices (ref_to_array: count + ptr)
                    if let Some((tri_offset, tri_count)) = tri_idx_offsets[j] {
                        sections[3].write_array_ref(tri_count as u32, 3, tri_offset);
                    } else {
                        sections[3].write_null_array();
                    }
                }
                bone_binding_offsets.push(Some((bb_array_offset, mesh.bone_bindings.len())));
            }
        }

        // Write Material structs for MaterialBindings (one set per mesh)
        let mut material_binding_offsets: Vec<Option<(u32, usize)>> = Vec::new();
        for mesh in &self.meshes {
            if mesh.material_binding_names.is_empty() {
                material_binding_offsets.push(None);
            } else {
                // Write Material structs (minimal: Name + ExtendedData)
                let mut mat_struct_offsets: Vec<u32> = Vec::new();
                for mat_name in &mesh.material_binding_names {
                    sections[3].align(8);
                    let mat_offset = sections[3].pos();
                    // Name
                    if let Some(&str_offset) = string_offsets.get(mat_name.as_str()) {
                        sections[3].write_ptr(0, str_offset);
                    } else {
                        sections[3].write_null_ptr();
                    }
                    // ExtendedData (null variant)
                    sections[3].write_null_ptr();
                    sections[3].write_null_ptr();
                    mat_struct_offsets.push(mat_offset);
                }

                // Write MaterialBindings array (array of ptrs to Material structs)
                sections[3].align(8);
                let mb_array_offset = sections[3].pos();
                for &mat_off in &mat_struct_offsets {
                    sections[3].write_ptr(3, mat_off);
                }

                material_binding_offsets.push(Some((
                    mb_array_offset,
                    mesh.material_binding_names.len(),
                )));
            }
        }

        // Write MeshPropertySet and MeshExtendedData structs to section 3 (if profiles exist)
        let mut ext_data_offsets: Vec<Option<(u32, u32)>> = Vec::new(); // (type_offset, data_offset)
        for mesh in &self.meshes {
            if let Some(ref profile) = mesh.bg3_profile {
                // Write MeshPropertySet data
                sections[3].align(8);
                let props_offset = sections[3].pos();

                // Flags[4] (u32 x 4)
                let mut flags0 = 0u32;
                if profile.mesh_proxy == Some(true) { flags0 |= 0x01; }
                if profile.cloth == Some(true) { flags0 |= 0x02; }
                if profile.proxy_geometry == Some(true) { flags0 |= 0x04; }
                if profile.rigid == Some(true) { flags0 |= 0x20; }
                if profile.spring == Some(true) { flags0 |= 0x40; }
                if profile.occluder == Some(true) { flags0 |= 0x80; }
                sections[3].write_u32(flags0);
                sections[3].write_u32(0); // Flags[1]
                let mut flags2 = 0u32;
                if profile.cloth_01 == Some(true) { flags2 |= 0x01; }
                if profile.cloth_02 == Some(true) { flags2 |= 0x02; }
                if profile.cloth_04 == Some(true) { flags2 |= 0x04; }
                if profile.cloth_physics == Some(true) { flags2 |= 0x100; }
                sections[3].write_u32(flags2);
                sections[3].write_u32(0); // Flags[3]
                // Lod[1]
                sections[3].write_i32(profile.lod.unwrap_or(0));
                // FormatDescs (null ptr)
                sections[3].write_null_ptr();
                // ExtendedData (null ptr - nested, not used)
                sections[3].write_null_ptr();
                // LodDistance[1]
                sections[3].write_f32(profile.lod_distance.unwrap_or(f32::MAX));
                // IsImpostor[1]
                sections[3].write_i32(i32::from(profile.impostor == Some(true)));

                // Write MeshExtendedData
                sections[3].align(8);
                let ext_offset = sections[3].pos();
                // MeshProxy
                sections[3].write_i32(i32::from(profile.mesh_proxy == Some(true)));
                // Rigid
                sections[3].write_i32(i32::from(profile.rigid == Some(true)));
                // Cloth
                sections[3].write_i32(i32::from(profile.cloth == Some(true)));
                // Spring
                sections[3].write_i32(i32::from(profile.spring == Some(true)));
                // Occluder
                sections[3].write_i32(i32::from(profile.occluder == Some(true)));
                // LOD
                sections[3].write_i32(profile.lod.unwrap_or(0));
                // UserDefinedProperties
                if let Some(ref udp) = mesh.user_defined_properties {
                    if let Some(&str_offset) = string_offsets.get(udp.as_str()) {
                        sections[3].write_ptr(0, str_offset);
                    } else {
                        sections[3].write_null_ptr();
                    }
                } else {
                    sections[3].write_null_ptr();
                }
                // UserMeshProperties (ptr to MeshPropertySet)
                sections[3].write_ptr(3, props_offset);
                // LSMVersion
                sections[3].write_i32(3);

                ext_data_offsets.push(Some((type_offsets.ext_data_type, ext_offset)));
            } else {
                ext_data_offsets.push(None);
            }
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

            // MaterialBindings
            if let Some((mb_offset, mb_count)) = material_binding_offsets[i] {
                sections[3].write_array_ref(mb_count as u32, 3, mb_offset);
            } else {
                sections[3].write_null_array();
            }

            // BoneBindings
            if let Some((bb_offset, bb_count)) = bone_binding_offsets[i] {
                sections[3].write_array_ref(bb_count as u32, 3, bb_offset);
            } else {
                sections[3].write_null_array();
            }

            // ExtendedData (variant reference: type_ptr + data_ptr)
            if let Some((type_off, data_off)) = ext_data_offsets[i] {
                sections[3].write_ptr(4, type_off);
                sections[3].write_ptr(3, data_off);
            } else {
                sections[3].write_null_ptr();
                sections[3].write_null_ptr();
            }

            mesh_offsets.push(mesh_offset);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn write_root_object(
        &self,
        sections: &mut [Section],
        string_offsets: &HashMap<String, u32>,
        type_offsets: &TypeOffsets,
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

        // Write Model data and pointer array if model is present
        let (model_ptr_array_offset, model_count) =
            self.write_model_data(sections, string_offsets, type_offsets, mesh_offsets, skeleton_offsets);

        // Write material pointer array (collect unique materials from MaterialBindings)
        let (material_ptr_array_offset, material_count) =
            self.write_material_references(sections, mesh_offsets);

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
        // Materials
        if material_count > 0 {
            sections[0].write_array_ref(material_count as u32, 0, material_ptr_array_offset);
        } else {
            sections[0].write_null_array();
        }
        // Skeletons
        if skeleton_offsets.is_empty() {
            sections[0].write_null_array();
        } else {
            sections[0].write_array_ref(
                skeleton_offsets.len() as u32,
                0,
                skeleton_ptr_array_offset,
            );
        }
        // VertexDatas
        sections[0].write_array_ref(self.meshes.len() as u32, 0, vd_ptr_array_offset);
        // TriTopologies
        sections[0].write_array_ref(self.meshes.len() as u32, 0, topo_ptr_array_offset);
        // Meshes
        sections[0].write_array_ref(self.meshes.len() as u32, 0, mesh_ptr_array_offset);
        // Models
        if model_count > 0 {
            sections[0].write_array_ref(model_count as u32, 0, model_ptr_array_offset);
        } else {
            sections[0].write_null_array();
        }
        // TrackGroups (empty)
        sections[0].write_null_array();
        // Animations (empty)
        sections[0].write_null_array();
        // ExtendedData (null)
        sections[0].write_null_ptr();
        sections[0].write_null_ptr();

        root_offset
    }

    fn write_model_data(
        &self,
        sections: &mut [Section],
        string_offsets: &HashMap<String, u32>,
        _type_offsets: &TypeOffsets,
        mesh_offsets: &[u32],
        skeleton_offsets: &[u32],
    ) -> (u32, usize) {
        let Some(ref model) = self.model else {
            return (0, 0);
        };

        // Write MeshBindings pointer array (pointers to mesh structs)
        let mesh_bindings_ptr_offset = if model.mesh_binding_names.is_empty() {
            0u32
        } else {
            sections[0].align(8);
            let offset = sections[0].pos();
            // Map mesh binding names to mesh offsets
            for binding_name in &model.mesh_binding_names {
                // Find the mesh offset that matches this name
                let mesh_idx = self
                    .meshes
                    .iter()
                    .position(|m| m.name == *binding_name);
                if let Some(idx) = mesh_idx {
                    if idx < mesh_offsets.len() {
                        sections[0].write_ptr(3, mesh_offsets[idx]);
                    } else {
                        sections[0].write_null_ptr();
                    }
                } else {
                    sections[0].write_null_ptr();
                }
            }
            offset
        };

        // Write Model struct to section 0
        sections[0].align(8);
        let model_struct_offset = sections[0].pos();

        // Name
        if let Some(&str_offset) = string_offsets.get(model.name.as_str()) {
            sections[0].write_ptr(0, str_offset);
        } else {
            sections[0].write_null_ptr();
        }

        // Skeleton (ptr)
        if !skeleton_offsets.is_empty() {
            sections[0].write_ptr(2, skeleton_offsets[0]);
        } else {
            sections[0].write_null_ptr();
        }

        // MeshBindings (array of refs)
        if model.mesh_binding_names.is_empty() {
            sections[0].write_null_array();
        } else {
            sections[0].write_array_ref(
                model.mesh_binding_names.len() as u32,
                0,
                mesh_bindings_ptr_offset,
            );
        }

        // InitialPlacement (Transform = 68 bytes: flags + translation + rotation + scale_shear)
        sections[0].write_u32(0x1FF);
        for &t in &model.initial_placement.translation {
            sections[0].write_f32(t);
        }
        for &r in &model.initial_placement.rotation {
            sections[0].write_f32(r);
        }
        for &s in &model.initial_placement.scale_shear {
            sections[0].write_f32(s);
        }

        // Write model pointer array
        sections[0].align(8);
        let model_ptr_array_offset = sections[0].pos();
        sections[0].write_ptr(0, model_struct_offset);

        (model_ptr_array_offset, 1)
    }

    fn write_material_references(
        &self,
        _sections: &mut [Section],
        _mesh_offsets: &[u32],
    ) -> (u32, usize) {
        // Materials at root level are referenced by the MaterialBindings in each mesh.
        // For now, we don't write a separate root-level Materials array since BG3
        // reads materials from mesh MaterialBindings directly.
        (0, 0)
    }
}

/// Collected type definition offsets for use across writing methods.
#[allow(clippy::struct_field_names)]
struct TypeOffsets {
    vertex_type: u32,
    ext_data_type: u32,
    _mesh_props_type: u32,
    root_type: u32,
    _model_type: u32,
}
