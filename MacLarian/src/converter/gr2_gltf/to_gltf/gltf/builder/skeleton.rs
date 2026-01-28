//! Skeleton methods for `GltfBuilder`

use crate::converter::gr2_gltf::to_gltf::gr2_reader::Skeleton;

use super::super::types::{GltfNode, GltfSkin};
use super::GltfBuilder;

impl GltfBuilder {
    pub fn add_skeleton(&mut self, skeleton: &Skeleton) -> usize {
        self.bone_node_offset = self.nodes.len();

        // Add bone nodes
        for (bone_idx, bone) in skeleton.bones.iter().enumerate() {
            let children: Vec<usize> = skeleton
                .bones
                .iter()
                .enumerate()
                .filter(|(_, b)| b.parent_index >= 0 && b.parent_index as usize == bone_idx)
                .map(|(i, _)| self.bone_node_offset + i)
                .collect();

            let translation = Some(bone.transform.translation);
            let rotation = Some(bone.transform.rotation);
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

        let ibm: Vec<[f32; 16]> = skeleton
            .bones
            .iter()
            .map(|b| b.inverse_world_transform)
            .collect();

        let ibm_accessor = self.add_inverse_bind_matrices(&ibm);

        let joints: Vec<usize> = (0..skeleton.bones.len())
            .map(|i| self.bone_node_offset + i)
            .collect();

        let root_bone_idx = skeleton
            .bones
            .iter()
            .position(|b| b.parent_index < 0)
            .map(|i| self.bone_node_offset + i);

        let skin_idx = self.skins.len();
        self.skins.push(GltfSkin {
            name: Some(skeleton.name.clone()),
            inverse_bind_matrices: Some(ibm_accessor),
            joints,
            skeleton: root_bone_idx,
        });

        skin_idx
    }
}
