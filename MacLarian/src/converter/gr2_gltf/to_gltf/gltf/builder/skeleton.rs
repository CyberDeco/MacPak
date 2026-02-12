//! Skeleton methods for `GltfBuilder`

use crate::converter::gr2_gltf::to_gltf::gr2_reader::Skeleton;

use super::super::types::{Bg3SkeletonProfile, GltfNode, GltfSkin, GltfSkinExtensions};
use super::GltfBuilder;

/// Result of adding a skeleton, including the remap table for vertex bone indices.
pub struct SkeletonResult {
    /// The skin index in the glTF document.
    pub skin_idx: usize,
    /// Maps old bone index → new bone index. Use this to remap vertex `bone_indices`.
    /// `remap[old_index] = new_index`
    pub bone_remap: Vec<u8>,
}

/// Compute a depth-first ordering of bones starting from root bones.
/// Returns a vec where `result[new_index] = old_index` and an inverse
/// map where `inverse[old_index] = new_index`.
fn depth_first_order(
    bones: &[crate::converter::gr2_gltf::to_gltf::gr2_reader::Bone],
) -> (Vec<usize>, Vec<usize>) {
    // Build children lists indexed by parent
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); bones.len()];
    let mut roots = Vec::new();
    for (i, bone) in bones.iter().enumerate() {
        if bone.parent_index < 0 {
            roots.push(i);
        } else {
            children[bone.parent_index as usize].push(i);
        }
    }

    // DFS traversal
    let mut order = Vec::with_capacity(bones.len());
    let mut stack: Vec<usize> = Vec::new();
    // Push roots in reverse so first root is processed first
    for &root in roots.iter().rev() {
        stack.push(root);
    }
    while let Some(idx) = stack.pop() {
        order.push(idx);
        // Push children in reverse so first child is processed first
        for &child in children[idx].iter().rev() {
            stack.push(child);
        }
    }

    // Build inverse map: inverse[old_index] = new_index
    let mut inverse = vec![0usize; bones.len()];
    for (new_idx, &old_idx) in order.iter().enumerate() {
        inverse[old_idx] = new_idx;
    }

    (order, inverse)
}

impl GltfBuilder {
    /// Add a skeleton with a BG3 profile, reordering bones to depth-first order
    /// for Blender compatibility. Returns a `SkeletonResult` with the skin index
    /// and a remap table for fixing vertex bone indices.
    pub fn add_skeleton_with_profile(
        &mut self,
        skeleton: &Skeleton,
        profile: Bg3SkeletonProfile,
    ) -> SkeletonResult {
        self.add_skeleton_internal(skeleton, Some(profile))
    }

    fn add_skeleton_internal(
        &mut self,
        skeleton: &Skeleton,
        profile: Option<Bg3SkeletonProfile>,
    ) -> SkeletonResult {
        self.bone_node_offset = self.nodes.len();

        let (dfs_order, dfs_inverse) = depth_first_order(&skeleton.bones);

        // Add bone nodes in depth-first order
        for &old_idx in &dfs_order {
            let bone = &skeleton.bones[old_idx];

            // Children: find original children, map to new indices, then to node indices
            let children: Vec<usize> = skeleton
                .bones
                .iter()
                .enumerate()
                .filter(|(_, b)| b.parent_index >= 0 && b.parent_index as usize == old_idx)
                .map(|(i, _)| self.bone_node_offset + dfs_inverse[i])
                .collect();

            // Apply X-axis reflection to match mesh vertex coordinate conversion.
            // Mesh positions negate X, so bone transforms must be reflected to match.
            let t = bone.transform.translation;
            let r = bone.transform.rotation;
            let translation = Some([-t[0], t[1], t[2]]);
            let rotation = Some([r[0], -r[1], -r[2], r[3]]);
            let scale = Some([
                bone.transform.scale_shear[0],
                bone.transform.scale_shear[4],
                bone.transform.scale_shear[8],
            ]);

            self.nodes.push(GltfNode {
                name: Some(bone.name.clone()),
                mesh: None,
                skin: None,
                children,
                translation,
                rotation,
                scale,
            });
        }

        // Inverse bind matrices in new depth-first order, with X-axis reflection applied.
        // For column-major 4x4: M' = S * M * S where S = diag(-1,1,1,1)
        // negates indices 1, 2, 3, 4, 8, 12.
        let ibm: Vec<[f32; 16]> = dfs_order
            .iter()
            .map(|&old_idx| {
                let mut m = skeleton.bones[old_idx].inverse_world_transform;
                m[1] = -m[1];
                m[2] = -m[2];
                m[3] = -m[3];
                m[4] = -m[4];
                m[8] = -m[8];
                m[12] = -m[12];
                m
            })
            .collect();

        let ibm_accessor = self.add_inverse_bind_matrices(&ibm);

        let joints: Vec<usize> = (0..skeleton.bones.len())
            .map(|i| self.bone_node_offset + i)
            .collect();

        // Root bone is always at new index 0 in depth-first order (first root found)
        let root_bone_idx = skeleton
            .bones
            .iter()
            .position(|b| b.parent_index < 0)
            .map(|_| self.bone_node_offset);

        // Reorder bone_lod_error in the profile to match depth-first order
        let extensions = profile.map(|mut p| {
            if let Some(ref lod_errors) = p.bone_lod_error {
                p.bone_lod_error = Some(
                    dfs_order
                        .iter()
                        .map(|&old_idx| lod_errors[old_idx])
                        .collect(),
                );
            }
            GltfSkinExtensions {
                bg3_profile: Some(p),
            }
        });

        let skin_idx = self.skins.len();
        self.skins.push(GltfSkin {
            name: Some(skeleton.name.clone()),
            inverse_bind_matrices: Some(ibm_accessor),
            joints,
            skeleton: root_bone_idx,
            extensions,
        });

        // Build remap table: remap[old_index] = new_index (as u8 for bone_indices)
        let bone_remap: Vec<u8> = dfs_inverse.iter().map(|&new_idx| new_idx as u8).collect();

        SkeletonResult {
            skin_idx,
            bone_remap,
        }
    }
}
