//! Skeleton/bone visualization using gizmos

use std::collections::HashSet;

use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy::mesh::skinning::SkinnedMesh;

use crate::viewer::types::ViewSettings;

/// Draw skeleton bones as gizmo lines
pub fn draw_bones(
    view_settings: Res<ViewSettings>,
    mut gizmos: Gizmos,
    skinned_meshes: Query<&SkinnedMesh>,
    transforms: Query<&GlobalTransform>,
    children_query: Query<&Children>,
    bone_parent_query: Query<&ChildOf>,
) {
    if !view_settings.show_bones {
        return;
    }

    // Collect all joint entities from skinned meshes
    let mut joint_set = HashSet::new();
    for skinned_mesh in &skinned_meshes {
        for joint in &skinned_mesh.joints {
            joint_set.insert(*joint);
        }
    }

    // Draw bones: line from each joint to its parent (if parent is also a joint)
    for joint_entity in &joint_set {
        if let Ok(joint_transform) = transforms.get(*joint_entity) {
            let joint_pos = joint_transform.translation();

            // Check if this joint has a parent that's also a joint
            if let Ok(child_of) = bone_parent_query.get(*joint_entity) {
                if joint_set.contains(&child_of.parent()) {
                    if let Ok(parent_transform) = transforms.get(child_of.parent()) {
                        let parent_pos = parent_transform.translation();

                        // Draw bone line
                        gizmos.line(parent_pos, joint_pos, tailwind::YELLOW_400);

                        // Draw small sphere at joint
                        gizmos.sphere(
                            Isometry3d::from_translation(joint_pos),
                            0.01,
                            tailwind::ORANGE_500,
                        );
                    }
                }
            }

            // Draw sphere at root joints (joints without a joint parent)
            let is_root = if let Ok(child_of) = bone_parent_query.get(*joint_entity) {
                !joint_set.contains(&child_of.parent())
            } else {
                true
            };

            if is_root {
                gizmos.sphere(
                    Isometry3d::from_translation(joint_pos),
                    0.02,
                    tailwind::RED_500,
                );
            }
        }
    }

    // Also draw lines to children
    for joint_entity in &joint_set {
        if let Ok(joint_transform) = transforms.get(*joint_entity) {
            let joint_pos = joint_transform.translation();

            if let Ok(children) = children_query.get(*joint_entity) {
                for child in children.iter() {
                    if joint_set.contains(&child) {
                        if let Ok(child_transform) = transforms.get(child) {
                            let child_pos = child_transform.translation();
                            gizmos.line(joint_pos, child_pos, tailwind::YELLOW_400);
                        }
                    }
                }
            }
        }
    }
}
