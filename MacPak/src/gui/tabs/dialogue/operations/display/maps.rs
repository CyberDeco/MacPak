//! Map building helpers for dialogue tree construction

use std::collections::{HashMap, HashSet};
use crate::dialog::{Dialog, NodeConstructor};

/// Build a map from node UUIDs to logicalnames inherited from Alias nodes that reference them
pub fn build_alias_logicalname_map(dialog: &Dialog) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for node in dialog.nodes.values() {
        if node.constructor == NodeConstructor::Alias {
            if let Some(ref source_uuid) = node.source_node {
                // Check if this Alias has a logicalname with INCLUSION pattern
                if let Some(logicalname) = node.editor_data.get("logicalname") {
                    let display_name = if logicalname.starts_with("Alias (") && logicalname.ends_with(")") {
                        // Extract inner name: "Alias (INCLUSION_LAE'ZEL)" -> "INCLUSION_LAE'ZEL"
                        Some(logicalname[7..logicalname.len()-1].to_string())
                    } else if logicalname.starts_with("INCLUSION_") {
                        Some(logicalname.clone())
                    } else {
                        None
                    };

                    if let Some(name) = display_name {
                        map.insert(source_uuid.clone(), name);
                    }
                }
            }
        }
    }

    map
}

/// Build a map of grandparent siblings for each node
/// This helps detect when a node appears as both a direct sibling and a descendant of its sibling
/// Key: node UUID, Value: set of UUIDs that are siblings of this node's parent (i.e., grandparent's other children)
pub fn build_ancestor_sibling_map(dialog: &Dialog) -> HashMap<String, HashSet<String>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();

    // For each node, find its children's siblings (which are the node's own children)
    for node in dialog.nodes.values() {
        let siblings_set: HashSet<String> = node.children.iter().cloned().collect();

        // For each child, record that its siblings are the other children of this node
        for child_uuid in &node.children {
            map.insert(child_uuid.clone(), siblings_set.clone());
        }
    }

    map
}
