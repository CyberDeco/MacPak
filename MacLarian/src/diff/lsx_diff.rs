//! LSX document diffing
//!

use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;
use crate::formats::lsf;
use crate::formats::lsx::{self, LsxAttribute, LsxDocument, LsxNode, LsxRegion};

use super::types::{
    AttributeChange, Change, ChangeType, DiffOptions, DiffResult, NodeChange, NodePath,
    RegionChange,
};

/// Diff two files (LSF or LSX)
///
/// Automatically detects file format and converts to LSX for comparison.
///
/// # Errors
/// Returns an error if files cannot be read or parsed.
pub fn diff_files<P: AsRef<Path>>(old: P, new: P, options: &DiffOptions) -> Result<DiffResult> {
    let old_doc = load_as_lsx(old)?;
    let new_doc = load_as_lsx(new)?;
    Ok(diff_documents(&old_doc, &new_doc, options))
}

/// Diff two LSX documents
pub fn diff_documents(old: &LsxDocument, new: &LsxDocument, options: &DiffOptions) -> DiffResult {
    let mut result = DiffResult::default();

    // Compare version
    if !options.ignore_version {
        let old_ver = format!(
            "{}.{}.{}.{}",
            old.major, old.minor, old.revision, old.build
        );
        let new_ver = format!(
            "{}.{}.{}.{}",
            new.major, new.minor, new.revision, new.build
        );
        if old_ver != new_ver {
            result.changes.push(Change::Version {
                old: old_ver,
                new: new_ver,
            });
        }
    }

    // Build maps of regions by ID
    let old_regions: HashMap<&str, &LsxRegion> = old.regions.iter().map(|r| (r.id.as_str(), r)).collect();
    let new_regions: HashMap<&str, &LsxRegion> = new.regions.iter().map(|r| (r.id.as_str(), r)).collect();

    // Find removed regions
    for (id, region) in &old_regions {
        if !new_regions.contains_key(id) {
            result.changes.push(Change::Region(RegionChange {
                id: (*id).to_string(),
                change_type: ChangeType::Removed,
                node_changes: collect_all_nodes_as_removed(region, &NodePath::new(id)),
            }));
        }
    }

    // Find added and modified regions
    for (id, new_region) in &new_regions {
        if let Some(old_region) = old_regions.get(id) {
            // Region exists in both - compare contents
            let region_change = diff_region(old_region, new_region, options);
            if !region_change.is_empty() {
                result.changes.push(Change::Region(region_change));
            }
        } else {
            // New region
            result.changes.push(Change::Region(RegionChange {
                id: (*id).to_string(),
                change_type: ChangeType::Added,
                node_changes: collect_all_nodes_as_added(new_region, &NodePath::new(id)),
            }));
        }
    }

    result
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
            // Read LSF and convert to LSX
            let lsf_doc = lsf::read_lsf(path)?;
            let xml = crate::converter::lsf_lsx_lsj::to_lsx(&lsf_doc)?;
            lsx::parse_lsx(&xml)
        }
        _ => {
            // Assume LSX
            lsx::read_lsx(path)
        }
    }
}

/// Compare two regions
fn diff_region(old: &LsxRegion, new: &LsxRegion, options: &DiffOptions) -> RegionChange {
    let path = NodePath::new(&old.id);
    let node_changes = diff_node_lists(&old.nodes, &new.nodes, &path, options);

    RegionChange {
        id: old.id.clone(),
        change_type: ChangeType::Modified,
        node_changes,
    }
}

/// Compare two lists of nodes
fn diff_node_lists(
    old_nodes: &[LsxNode],
    new_nodes: &[LsxNode],
    parent_path: &NodePath,
    options: &DiffOptions,
) -> Vec<NodeChange> {
    let mut changes = Vec::new();

    if options.match_by_key {
        // Match nodes by key attribute if available
        diff_nodes_by_key(old_nodes, new_nodes, parent_path, options, &mut changes);
    } else {
        // Match nodes by id and position
        diff_nodes_by_position(old_nodes, new_nodes, parent_path, options, &mut changes);
    }

    changes
}

/// Match and diff nodes by key attribute
fn diff_nodes_by_key(
    old_nodes: &[LsxNode],
    new_nodes: &[LsxNode],
    parent_path: &NodePath,
    options: &DiffOptions,
    changes: &mut Vec<NodeChange>,
) {
    // Group nodes by (id, key)
    let old_by_key: HashMap<(&str, Option<&str>), Vec<&LsxNode>> = {
        let mut map: HashMap<(&str, Option<&str>), Vec<&LsxNode>> = HashMap::new();
        for node in old_nodes {
            map.entry((node.id.as_str(), node.key.as_deref()))
                .or_default()
                .push(node);
        }
        map
    };

    let new_by_key: HashMap<(&str, Option<&str>), Vec<&LsxNode>> = {
        let mut map: HashMap<(&str, Option<&str>), Vec<&LsxNode>> = HashMap::new();
        for node in new_nodes {
            map.entry((node.id.as_str(), node.key.as_deref()))
                .or_default()
                .push(node);
        }
        map
    };

    // Track which old nodes have been matched
    let mut matched_old: std::collections::HashSet<*const LsxNode> = std::collections::HashSet::new();

    // Process new nodes
    for ((id, key), new_list) in &new_by_key {
        if let Some(old_list) = old_by_key.get(&(*id, *key)) {
            // Match by position within same (id, key) group
            for (i, new_node) in new_list.iter().enumerate() {
                let path = parent_path.with_segment(id).with_key(*key);
                if let Some(old_node) = old_list.get(i) {
                    matched_old.insert(*old_node as *const LsxNode);
                    let node_change = diff_single_node(old_node, new_node, &path, options);
                    if !node_change.is_empty() {
                        changes.push(node_change);
                    }
                } else {
                    // New node (more new than old with same key)
                    changes.push(NodeChange::added(path));
                }
            }
        } else {
            // All new nodes with this key are additions
            for new_node in new_list {
                let path = parent_path.with_segment(id).with_key(new_node.key.as_deref());
                changes.push(collect_node_as_added(new_node, &path));
            }
        }
    }

    // Find removed nodes (in old but not matched)
    for node in old_nodes {
        let ptr = node as *const LsxNode;
        if !matched_old.contains(&ptr) {
            let path = parent_path
                .with_segment(&node.id)
                .with_key(node.key.as_deref());
            changes.push(collect_node_as_removed(node, &path));
        }
    }
}

/// Match and diff nodes by position within same id groups
fn diff_nodes_by_position(
    old_nodes: &[LsxNode],
    new_nodes: &[LsxNode],
    parent_path: &NodePath,
    options: &DiffOptions,
    changes: &mut Vec<NodeChange>,
) {
    // Group nodes by id
    let old_by_id: HashMap<&str, Vec<&LsxNode>> = {
        let mut map: HashMap<&str, Vec<&LsxNode>> = HashMap::new();
        for node in old_nodes {
            map.entry(node.id.as_str()).or_default().push(node);
        }
        map
    };

    let new_by_id: HashMap<&str, Vec<&LsxNode>> = {
        let mut map: HashMap<&str, Vec<&LsxNode>> = HashMap::new();
        for node in new_nodes {
            map.entry(node.id.as_str()).or_default().push(node);
        }
        map
    };

    // All unique IDs
    let all_ids: std::collections::HashSet<&str> = old_by_id
        .keys()
        .chain(new_by_id.keys())
        .copied()
        .collect();

    for id in all_ids {
        let old_list = old_by_id.get(id).map_or(&[][..], |v| v.as_slice());
        let new_list = new_by_id.get(id).map_or(&[][..], |v| v.as_slice());

        let max_len = old_list.len().max(new_list.len());
        for i in 0..max_len {
            let path = parent_path.with_segment(id);
            match (old_list.get(i), new_list.get(i)) {
                (Some(old_node), Some(new_node)) => {
                    let node_change = diff_single_node(old_node, new_node, &path, options);
                    if !node_change.is_empty() {
                        changes.push(node_change);
                    }
                }
                (Some(old_node), None) => {
                    changes.push(collect_node_as_removed(old_node, &path));
                }
                (None, Some(new_node)) => {
                    changes.push(collect_node_as_added(new_node, &path));
                }
                (None, None) => unreachable!(),
            }
        }
    }
}

/// Compare two individual nodes
fn diff_single_node(
    old: &LsxNode,
    new: &LsxNode,
    path: &NodePath,
    options: &DiffOptions,
) -> NodeChange {
    let mut node_change = NodeChange::modified(path.clone());

    // Compare attributes
    node_change.attribute_changes = diff_attributes(&old.attributes, &new.attributes, options);

    // Compare children recursively
    node_change.child_changes = diff_node_lists(&old.children, &new.children, path, options);

    node_change
}

/// Compare attribute lists
fn diff_attributes(
    old_attrs: &[LsxAttribute],
    new_attrs: &[LsxAttribute],
    options: &DiffOptions,
) -> Vec<AttributeChange> {
    let mut changes = Vec::new();

    // Build maps by id
    let old_map: HashMap<&str, &LsxAttribute> =
        old_attrs.iter().map(|a| (a.id.as_str(), a)).collect();
    let new_map: HashMap<&str, &LsxAttribute> =
        new_attrs.iter().map(|a| (a.id.as_str(), a)).collect();

    // Find removed
    for (id, old_attr) in &old_map {
        if !new_map.contains_key(id) {
            changes.push(AttributeChange {
                id: (*id).to_string(),
                change_type: ChangeType::Removed,
                old_value: Some(get_attr_display_value(old_attr)),
                new_value: None,
                old_type: Some(old_attr.type_name.clone()),
                new_type: None,
            });
        }
    }

    // Find added and modified
    for (id, new_attr) in &new_map {
        if let Some(old_attr) = old_map.get(id) {
            // Check if changed
            if !attrs_equal(old_attr, new_attr, options) {
                changes.push(AttributeChange {
                    id: (*id).to_string(),
                    change_type: ChangeType::Modified,
                    old_value: Some(get_attr_display_value(old_attr)),
                    new_value: Some(get_attr_display_value(new_attr)),
                    old_type: Some(old_attr.type_name.clone()),
                    new_type: Some(new_attr.type_name.clone()),
                });
            }
        } else {
            changes.push(AttributeChange {
                id: (*id).to_string(),
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(get_attr_display_value(new_attr)),
                old_type: None,
                new_type: Some(new_attr.type_name.clone()),
            });
        }
    }

    changes
}

/// Check if two attributes are equal
fn attrs_equal(a: &LsxAttribute, b: &LsxAttribute, options: &DiffOptions) -> bool {
    if a.type_name != b.type_name {
        return false;
    }

    let a_val = get_attr_compare_value(a, options);
    let b_val = get_attr_compare_value(b, options);

    a_val == b_val
}

/// Get attribute value for comparison (applying options)
fn get_attr_compare_value(attr: &LsxAttribute, options: &DiffOptions) -> String {
    let value = get_attr_display_value(attr);
    if options.ignore_whitespace {
        // Normalize whitespace
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        value
    }
}

/// Get attribute value for display
fn get_attr_display_value(attr: &LsxAttribute) -> String {
    if let Some(handle) = &attr.handle {
        // TranslatedString - show handle
        if let Some(version) = attr.version {
            format!("{}:{}", handle, version)
        } else {
            handle.clone()
        }
    } else {
        attr.value.clone()
    }
}

/// Collect all nodes in a region as removed
fn collect_all_nodes_as_removed(region: &LsxRegion, path: &NodePath) -> Vec<NodeChange> {
    region
        .nodes
        .iter()
        .map(|n| collect_node_as_removed(n, &path.with_segment(&n.id)))
        .collect()
}

/// Collect all nodes in a region as added
fn collect_all_nodes_as_added(region: &LsxRegion, path: &NodePath) -> Vec<NodeChange> {
    region
        .nodes
        .iter()
        .map(|n| collect_node_as_added(n, &path.with_segment(&n.id)))
        .collect()
}

/// Collect a node and all its children as removed
fn collect_node_as_removed(node: &LsxNode, path: &NodePath) -> NodeChange {
    let mut change = NodeChange::removed(path.clone());
    change.child_changes = node
        .children
        .iter()
        .map(|c| collect_node_as_removed(c, &path.with_segment(&c.id)))
        .collect();
    change
}

/// Collect a node and all its children as added
fn collect_node_as_added(node: &LsxNode, path: &NodePath) -> NodeChange {
    let mut change = NodeChange::added(path.clone());
    change.child_changes = node
        .children
        .iter()
        .map(|c| collect_node_as_added(c, &path.with_segment(&c.id)))
        .collect();
    change
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

    fn make_node(id: &str, attrs: Vec<LsxAttribute>, children: Vec<LsxNode>) -> LsxNode {
        LsxNode {
            id: id.to_string(),
            key: None,
            attributes: attrs,
            children,
        }
    }

    fn make_attr(id: &str, type_name: &str, value: &str) -> LsxAttribute {
        LsxAttribute {
            id: id.to_string(),
            type_name: type_name.to_string(),
            value: value.to_string(),
            handle: None,
            version: None,
        }
    }

    #[test]
    fn test_identical_docs() {
        let doc = make_doc(vec![make_region(
            "Config",
            vec![make_node(
                "root",
                vec![make_attr("Name", "FixedString", "Test")],
                vec![],
            )],
        )]);

        let result = diff_documents(&doc, &doc, &DiffOptions::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_version_change() {
        let old = make_doc(vec![]);
        let mut new = old.clone();
        new.major = 5;

        let result = diff_documents(&old, &new, &DiffOptions::default());
        assert_eq!(result.change_count(), 1);

        let result_ignore = diff_documents(
            &old,
            &new,
            &DiffOptions {
                ignore_version: true,
                ..Default::default()
            },
        );
        assert!(result_ignore.is_empty());
    }

    #[test]
    fn test_attribute_change() {
        let old = make_doc(vec![make_region(
            "Config",
            vec![make_node(
                "root",
                vec![make_attr("Name", "FixedString", "OldName")],
                vec![],
            )],
        )]);

        let new = make_doc(vec![make_region(
            "Config",
            vec![make_node(
                "root",
                vec![make_attr("Name", "FixedString", "NewName")],
                vec![],
            )],
        )]);

        let result = diff_documents(&old, &new, &DiffOptions::default());
        assert_eq!(result.change_count(), 1);
    }

    #[test]
    fn test_node_added() {
        let old = make_doc(vec![make_region("Config", vec![])]);
        let new = make_doc(vec![make_region(
            "Config",
            vec![make_node("NewNode", vec![], vec![])],
        )]);

        let result = diff_documents(&old, &new, &DiffOptions::default());
        assert_eq!(result.change_count(), 1);
    }
}
