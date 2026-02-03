//! Three-way merge for LSX documents
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`
//!
//! SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;
use crate::formats::lsf;
use crate::formats::lsx::{self, LsxAttribute, LsxDocument, LsxNode, LsxRegion};

use super::types::{Conflict, ConflictType, MergeOptions, MergeResult, NodePath};

/// Merge three files (base, ours, theirs)
///
/// Automatically detects file format and converts to LSX for merging.
///
/// # Errors
/// Returns an error if files cannot be read or parsed.
pub fn merge_files<P: AsRef<Path>>(
    base: P,
    ours: P,
    theirs: P,
    options: &MergeOptions,
) -> Result<MergeResult> {
    let base_doc = load_as_lsx(base)?;
    let ours_doc = load_as_lsx(ours)?;
    let theirs_doc = load_as_lsx(theirs)?;
    Ok(merge_documents(&base_doc, &ours_doc, &theirs_doc, options))
}

/// Merge three LSX documents
pub fn merge_documents(
    base: &LsxDocument,
    ours: &LsxDocument,
    theirs: &LsxDocument,
    options: &MergeOptions,
) -> MergeResult {
    let mut conflicts = Vec::new();
    let mut ours_applied = 0;
    let mut theirs_applied = 0;

    // Start with base and apply changes
    let mut merged = base.clone();

    // Use "ours" version info (arbitrary choice)
    merged.major = ours.major;
    merged.minor = ours.minor;
    merged.revision = ours.revision;
    merged.build = ours.build;

    // Merge regions
    merged.regions = merge_regions(
        &base.regions,
        &ours.regions,
        &theirs.regions,
        options,
        &mut conflicts,
        &mut ours_applied,
        &mut theirs_applied,
    );

    MergeResult {
        merged,
        conflicts,
        ours_applied,
        theirs_applied,
    }
}

/// Load a file as LSX document (converting from LSF if needed)
fn load_as_lsx<P: AsRef<Path>>(path: P) -> Result<LsxDocument> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);

    match ext.as_deref() {
        Some("lsf") => {
            let lsf_doc = lsf::read_lsf(path)?;
            let xml = crate::converter::lsf_lsx_lsj::to_lsx(&lsf_doc)?;
            lsx::parse_lsx(&xml)
        }
        _ => lsx::read_lsx(path),
    }
}

/// Merge region lists
fn merge_regions(
    base: &[LsxRegion],
    ours: &[LsxRegion],
    theirs: &[LsxRegion],
    options: &MergeOptions,
    conflicts: &mut Vec<Conflict>,
    ours_applied: &mut usize,
    theirs_applied: &mut usize,
) -> Vec<LsxRegion> {
    let base_map: HashMap<&str, &LsxRegion> = base.iter().map(|r| (r.id.as_str(), r)).collect();
    let ours_map: HashMap<&str, &LsxRegion> = ours.iter().map(|r| (r.id.as_str(), r)).collect();
    let theirs_map: HashMap<&str, &LsxRegion> =
        theirs.iter().map(|r| (r.id.as_str(), r)).collect();

    let mut result = Vec::new();

    // All region IDs
    let all_ids: std::collections::HashSet<&str> = base_map
        .keys()
        .chain(ours_map.keys())
        .chain(theirs_map.keys())
        .copied()
        .collect();

    for id in all_ids {
        let base_region = base_map.get(id);
        let ours_region = ours_map.get(id);
        let theirs_region = theirs_map.get(id);
        let path = NodePath::new(id);

        match (base_region, ours_region, theirs_region) {
            // In all three - merge contents
            (Some(b), Some(o), Some(t)) => {
                let merged_nodes = merge_node_lists(
                    &b.nodes,
                    &o.nodes,
                    &t.nodes,
                    &path,
                    options,
                    conflicts,
                    ours_applied,
                    theirs_applied,
                );
                result.push(LsxRegion {
                    id: id.to_string(),
                    nodes: merged_nodes,
                });
            }

            // Only in base - both deleted it (no conflict)
            (Some(_), None, None) => {
                // Region was deleted by both sides
            }

            // In base and ours, not theirs - theirs deleted it
            (Some(b), Some(o), None) => {
                if regions_equal(b, o) {
                    // Ours didn't change, theirs deleted - accept deletion
                    *theirs_applied += 1;
                } else if options.prefer_ours {
                    result.push((*o).clone());
                    *ours_applied += 1;
                } else if options.prefer_theirs {
                    *theirs_applied += 1;
                } else {
                    // Conflict: ours modified, theirs deleted
                    conflicts.push(Conflict {
                        path,
                        conflict_type: ConflictType::DeleteModifyConflict,
                        description: "Region modified by ours, deleted by theirs".to_string(),
                        ours: Some(format!("(modified region {})", id)),
                        theirs: Some("(deleted)".to_string()),
                    });
                    // Keep ours on conflict
                    result.push((*o).clone());
                }
            }

            // In base and theirs, not ours - ours deleted it
            (Some(b), None, Some(t)) => {
                if regions_equal(b, t) {
                    // Theirs didn't change, ours deleted - accept deletion
                    *ours_applied += 1;
                } else if options.prefer_ours {
                    *ours_applied += 1;
                } else if options.prefer_theirs {
                    result.push((*t).clone());
                    *theirs_applied += 1;
                } else {
                    // Conflict: theirs modified, ours deleted
                    conflicts.push(Conflict {
                        path,
                        conflict_type: ConflictType::DeleteModifyConflict,
                        description: "Region deleted by ours, modified by theirs".to_string(),
                        ours: Some("(deleted)".to_string()),
                        theirs: Some(format!("(modified region {})", id)),
                    });
                    // Keep theirs on conflict
                    result.push((*t).clone());
                }
            }

            // Only in ours - ours added it
            (None, Some(o), None) => {
                result.push((*o).clone());
                *ours_applied += 1;
            }

            // Only in theirs - theirs added it
            (None, None, Some(t)) => {
                result.push((*t).clone());
                *theirs_applied += 1;
            }

            // In ours and theirs but not base - both added
            (None, Some(o), Some(t)) => {
                if regions_equal(o, t) {
                    // Same addition - no conflict
                    result.push((*o).clone());
                } else if options.prefer_ours {
                    result.push((*o).clone());
                    *ours_applied += 1;
                } else if options.prefer_theirs {
                    result.push((*t).clone());
                    *theirs_applied += 1;
                } else {
                    // Conflict: both added different content
                    conflicts.push(Conflict {
                        path,
                        conflict_type: ConflictType::AddAddConflict,
                        description: "Region added by both with different content".to_string(),
                        ours: Some(format!("(added region {})", id)),
                        theirs: Some(format!("(added region {})", id)),
                    });
                    // Keep ours on conflict
                    result.push((*o).clone());
                }
            }

            // Not in any - shouldn't happen
            (None, None, None) => {}
        }
    }

    result
}

/// Merge node lists
#[allow(clippy::too_many_arguments)]
fn merge_node_lists(
    base: &[LsxNode],
    ours: &[LsxNode],
    theirs: &[LsxNode],
    parent_path: &NodePath,
    options: &MergeOptions,
    conflicts: &mut Vec<Conflict>,
    ours_applied: &mut usize,
    theirs_applied: &mut usize,
) -> Vec<LsxNode> {
    // Group nodes by (id, key)
    type NodeKey<'a> = (&'a str, Option<&'a str>);

    fn group_nodes(nodes: &[LsxNode]) -> HashMap<(&str, Option<&str>), Vec<&LsxNode>> {
        let mut map: HashMap<NodeKey, Vec<&LsxNode>> = HashMap::new();
        for node in nodes {
            map.entry((node.id.as_str(), node.key.as_deref()))
                .or_default()
                .push(node);
        }
        map
    }

    let base_map = group_nodes(base);
    let ours_map = group_nodes(ours);
    let theirs_map = group_nodes(theirs);

    let mut result = Vec::new();

    // All node keys
    let all_keys: std::collections::HashSet<NodeKey> = base_map
        .keys()
        .chain(ours_map.keys())
        .chain(theirs_map.keys())
        .copied()
        .collect();

    for key @ (id, node_key) in all_keys {
        let base_nodes = base_map.get(&key).map_or(&[][..], |v| v.as_slice());
        let ours_nodes = ours_map.get(&key).map_or(&[][..], |v| v.as_slice());
        let theirs_nodes = theirs_map.get(&key).map_or(&[][..], |v| v.as_slice());

        let path = parent_path.with_segment(id).with_key(node_key);

        // Merge by position within same key group
        let max_len = base_nodes.len().max(ours_nodes.len()).max(theirs_nodes.len());

        for i in 0..max_len {
            let base_node = base_nodes.get(i).copied();
            let ours_node = ours_nodes.get(i).copied();
            let theirs_node = theirs_nodes.get(i).copied();

            if let Some(merged) = merge_single_node(
                base_node,
                ours_node,
                theirs_node,
                &path,
                options,
                conflicts,
                ours_applied,
                theirs_applied,
            ) {
                result.push(merged);
            }
        }
    }

    result
}

/// Merge a single node position
#[allow(clippy::too_many_arguments)]
fn merge_single_node(
    base: Option<&LsxNode>,
    ours: Option<&LsxNode>,
    theirs: Option<&LsxNode>,
    path: &NodePath,
    options: &MergeOptions,
    conflicts: &mut Vec<Conflict>,
    ours_applied: &mut usize,
    theirs_applied: &mut usize,
) -> Option<LsxNode> {
    match (base, ours, theirs) {
        // In all three - merge contents
        (Some(b), Some(o), Some(t)) => {
            Some(merge_node_contents(
                b,
                o,
                t,
                path,
                options,
                conflicts,
                ours_applied,
                theirs_applied,
            ))
        }

        // Both deleted
        (Some(_), None, None) => None,

        // Ours deleted, theirs has it
        (Some(b), None, Some(t)) => {
            if nodes_equal(b, t) {
                *ours_applied += 1;
                None
            } else if options.prefer_ours {
                *ours_applied += 1;
                None
            } else if options.prefer_theirs {
                *theirs_applied += 1;
                Some(t.clone())
            } else {
                conflicts.push(Conflict {
                    path: path.clone(),
                    conflict_type: ConflictType::DeleteModifyConflict,
                    description: "Node deleted by ours, modified by theirs".to_string(),
                    ours: Some("(deleted)".to_string()),
                    theirs: Some(format!("(node {})", t.id)),
                });
                Some(t.clone())
            }
        }

        // Theirs deleted, ours has it
        (Some(b), Some(o), None) => {
            if nodes_equal(b, o) {
                *theirs_applied += 1;
                None
            } else if options.prefer_ours {
                *ours_applied += 1;
                Some(o.clone())
            } else if options.prefer_theirs {
                *theirs_applied += 1;
                None
            } else {
                conflicts.push(Conflict {
                    path: path.clone(),
                    conflict_type: ConflictType::DeleteModifyConflict,
                    description: "Node modified by ours, deleted by theirs".to_string(),
                    ours: Some(format!("(node {})", o.id)),
                    theirs: Some("(deleted)".to_string()),
                });
                Some(o.clone())
            }
        }

        // Only ours added
        (None, Some(o), None) => {
            *ours_applied += 1;
            Some(o.clone())
        }

        // Only theirs added
        (None, None, Some(t)) => {
            *theirs_applied += 1;
            Some(t.clone())
        }

        // Both added
        (None, Some(o), Some(t)) => {
            if nodes_equal(o, t) {
                Some(o.clone())
            } else if options.prefer_ours {
                *ours_applied += 1;
                Some(o.clone())
            } else if options.prefer_theirs {
                *theirs_applied += 1;
                Some(t.clone())
            } else {
                conflicts.push(Conflict {
                    path: path.clone(),
                    conflict_type: ConflictType::AddAddConflict,
                    description: "Node added by both with different content".to_string(),
                    ours: Some(format!("(node {})", o.id)),
                    theirs: Some(format!("(node {})", t.id)),
                });
                Some(o.clone())
            }
        }

        // Not in any
        (None, None, None) => None,
    }
}

/// Merge node contents (attributes and children)
#[allow(clippy::too_many_arguments)]
fn merge_node_contents(
    base: &LsxNode,
    ours: &LsxNode,
    theirs: &LsxNode,
    path: &NodePath,
    options: &MergeOptions,
    conflicts: &mut Vec<Conflict>,
    ours_applied: &mut usize,
    theirs_applied: &mut usize,
) -> LsxNode {
    // Merge attributes
    let merged_attrs = merge_attributes(
        &base.attributes,
        &ours.attributes,
        &theirs.attributes,
        path,
        options,
        conflicts,
        ours_applied,
        theirs_applied,
    );

    // Merge children recursively
    let merged_children = merge_node_lists(
        &base.children,
        &ours.children,
        &theirs.children,
        path,
        options,
        conflicts,
        ours_applied,
        theirs_applied,
    );

    LsxNode {
        id: ours.id.clone(),
        key: ours.key.clone(),
        attributes: merged_attrs,
        children: merged_children,
    }
}

/// Merge attribute lists
#[allow(clippy::too_many_arguments)]
fn merge_attributes(
    base: &[LsxAttribute],
    ours: &[LsxAttribute],
    theirs: &[LsxAttribute],
    path: &NodePath,
    options: &MergeOptions,
    conflicts: &mut Vec<Conflict>,
    ours_applied: &mut usize,
    theirs_applied: &mut usize,
) -> Vec<LsxAttribute> {
    let base_map: HashMap<&str, &LsxAttribute> =
        base.iter().map(|a| (a.id.as_str(), a)).collect();
    let ours_map: HashMap<&str, &LsxAttribute> =
        ours.iter().map(|a| (a.id.as_str(), a)).collect();
    let theirs_map: HashMap<&str, &LsxAttribute> =
        theirs.iter().map(|a| (a.id.as_str(), a)).collect();

    let mut result = Vec::new();

    let all_ids: std::collections::HashSet<&str> = base_map
        .keys()
        .chain(ours_map.keys())
        .chain(theirs_map.keys())
        .copied()
        .collect();

    for id in all_ids {
        let base_attr = base_map.get(id);
        let ours_attr = ours_map.get(id);
        let theirs_attr = theirs_map.get(id);

        match (base_attr, ours_attr, theirs_attr) {
            // In all three
            (Some(b), Some(o), Some(t)) => {
                let ours_changed = !attrs_equal(b, o);
                let theirs_changed = !attrs_equal(b, t);

                if !ours_changed && !theirs_changed {
                    // No changes
                    result.push((*b).clone());
                } else if ours_changed && !theirs_changed {
                    // Only ours changed
                    result.push((*o).clone());
                    *ours_applied += 1;
                } else if !ours_changed && theirs_changed {
                    // Only theirs changed
                    result.push((*t).clone());
                    *theirs_applied += 1;
                } else if attrs_equal(o, t) {
                    // Both changed to same value
                    result.push((*o).clone());
                } else if options.prefer_ours {
                    result.push((*o).clone());
                    *ours_applied += 1;
                } else if options.prefer_theirs {
                    result.push((*t).clone());
                    *theirs_applied += 1;
                } else {
                    // Conflict
                    conflicts.push(Conflict {
                        path: path.clone(),
                        conflict_type: ConflictType::AttributeConflict,
                        description: format!("Attribute '{}' modified differently", id),
                        ours: Some(get_attr_value(o)),
                        theirs: Some(get_attr_value(t)),
                    });
                    result.push((*o).clone());
                }
            }

            // Both deleted
            (Some(_), None, None) => {
                // Deleted by both
            }

            // Ours deleted, theirs modified
            (Some(b), None, Some(t)) => {
                if attrs_equal(b, t) {
                    *ours_applied += 1;
                } else if options.prefer_ours {
                    *ours_applied += 1;
                } else if options.prefer_theirs {
                    result.push((*t).clone());
                    *theirs_applied += 1;
                } else {
                    conflicts.push(Conflict {
                        path: path.clone(),
                        conflict_type: ConflictType::DeleteModifyConflict,
                        description: format!("Attribute '{}' deleted by ours, modified by theirs", id),
                        ours: Some("(deleted)".to_string()),
                        theirs: Some(get_attr_value(t)),
                    });
                    result.push((*t).clone());
                }
            }

            // Theirs deleted, ours modified
            (Some(b), Some(o), None) => {
                if attrs_equal(b, o) {
                    *theirs_applied += 1;
                } else if options.prefer_ours {
                    result.push((*o).clone());
                    *ours_applied += 1;
                } else if options.prefer_theirs {
                    *theirs_applied += 1;
                } else {
                    conflicts.push(Conflict {
                        path: path.clone(),
                        conflict_type: ConflictType::DeleteModifyConflict,
                        description: format!("Attribute '{}' modified by ours, deleted by theirs", id),
                        ours: Some(get_attr_value(o)),
                        theirs: Some("(deleted)".to_string()),
                    });
                    result.push((*o).clone());
                }
            }

            // Only ours added
            (None, Some(o), None) => {
                result.push((*o).clone());
                *ours_applied += 1;
            }

            // Only theirs added
            (None, None, Some(t)) => {
                result.push((*t).clone());
                *theirs_applied += 1;
            }

            // Both added
            (None, Some(o), Some(t)) => {
                if attrs_equal(o, t) {
                    result.push((*o).clone());
                } else if options.prefer_ours {
                    result.push((*o).clone());
                    *ours_applied += 1;
                } else if options.prefer_theirs {
                    result.push((*t).clone());
                    *theirs_applied += 1;
                } else {
                    conflicts.push(Conflict {
                        path: path.clone(),
                        conflict_type: ConflictType::AddAddConflict,
                        description: format!("Attribute '{}' added by both with different values", id),
                        ours: Some(get_attr_value(o)),
                        theirs: Some(get_attr_value(t)),
                    });
                    result.push((*o).clone());
                }
            }

            // Not in any
            (None, None, None) => {}
        }
    }

    result
}

/// Check if two regions are equal
fn regions_equal(a: &LsxRegion, b: &LsxRegion) -> bool {
    a.id == b.id && a.nodes.len() == b.nodes.len() && a.nodes.iter().zip(&b.nodes).all(|(x, y)| nodes_equal(x, y))
}

/// Check if two nodes are equal
fn nodes_equal(a: &LsxNode, b: &LsxNode) -> bool {
    a.id == b.id
        && a.key == b.key
        && a.attributes.len() == b.attributes.len()
        && a.children.len() == b.children.len()
        && a.attributes.iter().zip(&b.attributes).all(|(x, y)| attrs_equal(x, y))
        && a.children.iter().zip(&b.children).all(|(x, y)| nodes_equal(x, y))
}

/// Check if two attributes are equal
fn attrs_equal(a: &LsxAttribute, b: &LsxAttribute) -> bool {
    a.id == b.id
        && a.type_name == b.type_name
        && a.value == b.value
        && a.handle == b.handle
        && a.version == b.version
}

/// Get attribute value for display
fn get_attr_value(attr: &LsxAttribute) -> String {
    if let Some(handle) = &attr.handle {
        if let Some(version) = attr.version {
            format!("{}:{}", handle, version)
        } else {
            handle.clone()
        }
    } else {
        attr.value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(regions: Vec<LsxRegion>) -> LsxDocument {
        LsxDocument {
            major: 4,
            minor: 0,
            revision: 0,
            build: 0,
            regions,
        }
    }

    fn make_region(id: &str, nodes: Vec<LsxNode>) -> LsxRegion {
        LsxRegion {
            id: id.to_string(),
            nodes,
        }
    }

    fn make_node(id: &str, attrs: Vec<LsxAttribute>) -> LsxNode {
        LsxNode {
            id: id.to_string(),
            key: None,
            attributes: attrs,
            children: Vec::new(),
        }
    }

    fn make_attr(id: &str, value: &str) -> LsxAttribute {
        LsxAttribute {
            id: id.to_string(),
            type_name: "FixedString".to_string(),
            value: value.to_string(),
            handle: None,
            version: None,
        }
    }

    #[test]
    fn test_no_changes() {
        let doc = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Test")])],
        )]);

        let result = merge_documents(&doc, &doc, &doc, &MergeOptions::default());
        assert!(!result.has_conflicts());
        assert_eq!(result.ours_applied, 0);
        assert_eq!(result.theirs_applied, 0);
    }

    #[test]
    fn test_ours_only_change() {
        let base = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Original")])],
        )]);
        let ours = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Ours")])],
        )]);

        let result = merge_documents(&base, &ours, &base, &MergeOptions::default());
        assert!(!result.has_conflicts());
        assert_eq!(result.ours_applied, 1);
        assert_eq!(result.theirs_applied, 0);
    }

    #[test]
    fn test_conflict() {
        let base = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Original")])],
        )]);
        let ours = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Ours")])],
        )]);
        let theirs = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Theirs")])],
        )]);

        let result = merge_documents(&base, &ours, &theirs, &MergeOptions::default());
        assert!(result.has_conflicts());
        assert_eq!(result.conflicts.len(), 1);
    }

    #[test]
    fn test_prefer_ours() {
        let base = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Original")])],
        )]);
        let ours = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Ours")])],
        )]);
        let theirs = make_doc(vec![make_region(
            "Config",
            vec![make_node("root", vec![make_attr("Name", "Theirs")])],
        )]);

        let result = merge_documents(
            &base,
            &ours,
            &theirs,
            &MergeOptions {
                prefer_ours: true,
                ..Default::default()
            },
        );
        assert!(!result.has_conflicts());
        assert_eq!(result.ours_applied, 1);
    }
}
