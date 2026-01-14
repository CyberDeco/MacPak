//! Tree building logic for constructing the display node tree

use std::collections::{HashMap, HashSet};
use floem::prelude::RwSignal;
use MacLarian::dialog::{Dialog, NodeConstructor};
use crate::gui::state::{DisplayNode, DisplayFlag};
use super::chain::{follow_chain_for_text, get_jump_effective_children, resolve_passthrough_children};
use super::maps::{build_alias_logicalname_map, build_ancestor_sibling_map};

/// Build display nodes from a dialog
pub fn build_display_nodes(dialog: &Dialog) -> Vec<DisplayNode> {
    let mut nodes = Vec::new();
    let mut visited = HashSet::new();

    // Build reverse lookup for logicalnames from Alias nodes
    let alias_logicalnames = build_alias_logicalname_map(dialog);

    // Build map of each node's siblings (children of the same parent)
    // Used to detect when a node appears as both sibling and descendant of sibling
    let sibling_map = build_ancestor_sibling_map(dialog);

    // Start from root nodes (parent_expanded = true for root level)
    for root_uuid in &dialog.root_nodes {
        build_node_tree(dialog, root_uuid, None, 0, true, &mut nodes, &mut visited, &alias_logicalnames, &sibling_map, &HashSet::new());
    }

    // Add any orphaned nodes
    for uuid in &dialog.node_order {
        if !visited.contains(uuid) {
            build_node_tree(dialog, uuid, None, 0, true, &mut nodes, &mut visited, &alias_logicalnames, &sibling_map, &HashSet::new());
        }
    }

    nodes
}

/// Recursively build the node tree
fn build_node_tree(
    dialog: &Dialog,
    uuid: &str,
    parent_uuid: Option<&str>,
    depth: usize,
    parent_expanded: bool, // Whether the parent node is expanded
    nodes: &mut Vec<DisplayNode>,
    visited: &mut HashSet<String>,
    alias_logicalnames: &HashMap<String, String>,
    sibling_map: &HashMap<String, HashSet<String>>,
    ancestor_siblings: &HashSet<String>, // Siblings of ancestors - if we're in this set, we should be a link
) {
    // Check if this node is a sibling of one of our ancestors
    // This handles the case where a node appears as both a direct sibling AND a descendant of that sibling
    // In that case, we show the full node at the sibling level and a link at the descendant level
    if ancestor_siblings.contains(uuid) {
        // Only create a link if the node has text - otherwise linking to it is not useful
        if let Some(node) = dialog.get_node(uuid) {
            if dialog.get_node_text(node).is_some() {
                build_link_node(dialog, uuid, parent_uuid, depth, parent_expanded, nodes);
            }
        }
        return;
    }

    if visited.contains(uuid) {
        return;
    }
    visited.insert(uuid.to_string());

    let Some(node) = dialog.get_node(uuid) else {
        return;
    };

    // Check if this is a "pass-through" node that should be skipped:
    // 1. No text + no flags + single Jump/Alias/VisualState child - let the child display the content
    // 2. VisualState nodes - hoist children up, these are just visual context containers
    let has_text = dialog.get_node_text(node).is_some();
    // Check for actual flags, not just empty FlagGroups (parser may create empty groups from [{}])
    let has_flags = node.check_flags.iter().any(|fg| !fg.flags.is_empty())
        || node.set_flags.iter().any(|fg| !fg.flags.is_empty());
    let is_visual_state = node.constructor == NodeConstructor::VisualState;
    let is_jump_alias_container = !has_text && !has_flags && node.children.len() == 1 && {
        if let Some(child) = dialog.get_node(&node.children[0]) {
            matches!(child.constructor, NodeConstructor::Jump | NodeConstructor::Alias | NodeConstructor::VisualState)
        } else {
            false
        }
    };

    if is_visual_state {
        // Skip VisualState nodes, but process their children at the same depth with the same parent
        // If ALL children are already visited, skip entirely to avoid redundant links
        let all_visited = node.children.iter().all(|c| visited.contains(c));
        if all_visited {
            return;
        }

        for child_uuid in &node.children {
            if visited.contains(child_uuid) {
                build_link_node(dialog, child_uuid, parent_uuid, depth, parent_expanded, nodes);
            } else {
                build_node_tree(dialog, child_uuid, parent_uuid, depth, parent_expanded, nodes, visited, alias_logicalnames, sibling_map, ancestor_siblings);
            }
        }
        return;
    }

    // Jump nodes with no flags are pass-through - they just point to target content
    // Mark the Jump as visited but hoist the target's children directly
    // Note: check for actual flags, not just empty FlagGroups (parser may create empty groups)
    let has_actual_flags = node.check_flags.iter().any(|fg| !fg.flags.is_empty())
        || node.set_flags.iter().any(|fg| !fg.flags.is_empty());
    let is_passthrough_jump = node.constructor == NodeConstructor::Jump
        && !has_actual_flags
        && node.children.is_empty();

    if is_passthrough_jump {
        visited.insert(uuid.to_string());
        if let Some(ref target_uuid) = node.jump_target {
            if let Some(target_node) = dialog.get_node(target_uuid) {
                // Get effective children: follow through VisualState if needed
                let effective_children = get_jump_effective_children(dialog, target_node, node.jump_target_point);

                // If target is already visited AND has children, link to target directly
                // This prevents cascading into children's children which creates too many links
                // If target has no children, do nothing (matches old behavior)
                if visited.contains(target_uuid) {
                    if !effective_children.is_empty() {
                        build_link_node(dialog, target_uuid, parent_uuid, depth, parent_expanded, nodes);
                    }
                    // else: target has no children, nothing to show
                } else {
                    for child_uuid in &effective_children {
                        if visited.contains(child_uuid) {
                            build_link_node(dialog, child_uuid, parent_uuid, depth, parent_expanded, nodes);
                        } else {
                            build_node_tree(dialog, child_uuid, parent_uuid, depth, parent_expanded, nodes, visited, alias_logicalnames, sibling_map, ancestor_siblings);
                        }
                    }
                }
            }
        }
        return;
    }

    if is_jump_alias_container {
        // This is a pass-through node (no text + single Jump/Alias/VisualState child).
        // Check if this node has an inherited logicalname from an Alias - if so, keep it visible
        // because the logicalname provides meaningful context (e.g., INCLUSION_LAE'ZEL for companion branches)
        let has_inherited_logicalname = alias_logicalnames.contains_key(uuid);

        if !has_inherited_logicalname {
            // No meaningful context - hoist the child up to replace this node
            for child_uuid in &node.children {
                if visited.contains(child_uuid) {
                    // Child already visited - check if it's a pass-through type
                    if let Some(child_node) = dialog.get_node(child_uuid) {
                        if child_node.constructor == NodeConstructor::VisualState {
                            // VisualState is pass-through - follow through to its content children
                            for grandchild_uuid in &child_node.children {
                                if visited.contains(grandchild_uuid) {
                                    build_link_node(dialog, grandchild_uuid, parent_uuid, depth, parent_expanded, nodes);
                                } else {
                                    build_node_tree(dialog, grandchild_uuid, parent_uuid, depth, parent_expanded, nodes, visited, alias_logicalnames, sibling_map, ancestor_siblings);
                                }
                            }
                            continue;
                        }
                        // Check for pass-through Jump (no actual flags, no children)
                        let has_actual_flags = child_node.check_flags.iter().any(|fg| !fg.flags.is_empty())
                            || child_node.set_flags.iter().any(|fg| !fg.flags.is_empty());
                        let is_passthrough_jump = child_node.constructor == NodeConstructor::Jump
                            && !has_actual_flags
                            && child_node.children.is_empty();
                        if is_passthrough_jump {
                            // Skip - content already shown through another path
                            continue;
                        }
                    }
                    // Not pass-through, create link
                    build_link_node(dialog, child_uuid, parent_uuid, depth, parent_expanded, nodes);
                } else {
                    // Child not visited yet - but if it's a pass-through Jump whose target
                    // content is all visited, skip it. Otherwise we'd create links at the
                    // wrong level (as siblings to the parent's other children).
                    if let Some(child_node) = dialog.get_node(child_uuid) {
                        // Check for actual flags (not just empty FlagGroups)
                        let has_actual_check_flags = child_node.check_flags.iter().any(|fg| !fg.flags.is_empty());
                        let has_actual_set_flags = child_node.set_flags.iter().any(|fg| !fg.flags.is_empty());

                        let is_child_passthrough_jump = child_node.constructor == NodeConstructor::Jump
                            && !has_actual_check_flags
                            && !has_actual_set_flags
                            && child_node.children.is_empty();

                        if is_child_passthrough_jump {
                            if let Some(ref target_uuid) = child_node.jump_target {
                                if let Some(target_node) = dialog.get_node(target_uuid) {
                                    let effective = get_jump_effective_children(dialog, target_node, child_node.jump_target_point);
                                    let all_visited = !effective.is_empty() && effective.iter().all(|c| visited.contains(c));
                                    if all_visited {
                                        // Content already shown elsewhere - skip to avoid duplicate links
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    build_node_tree(dialog, child_uuid, parent_uuid, depth, parent_expanded, nodes, visited, alias_logicalnames, sibling_map, ancestor_siblings);
                }
            }
            return;
        }
        // If has_inherited_logicalname, fall through to display this node with the logicalname
    }

    // Skip if this node resolves to the same text as one of its children (redundant parent)
    // BUT preserve nodes with inherited logicalnames (they provide meaningful context)
    // AND preserve RollResult nodes (they display success/failure indicator and roll info)
    let has_inherited_logicalname = alias_logicalnames.contains_key(uuid);
    let is_roll_result = node.constructor == NodeConstructor::RollResult;
    if !has_inherited_logicalname && !is_roll_result && !node.children.is_empty() {
        let parent_result = follow_chain_for_text(dialog, node, 10);
        if let Some(ref parent_handle) = parent_result.handle {
            for child_uuid in &node.children {
                if let Some(child_node) = dialog.get_node(child_uuid) {
                    let child_result = follow_chain_for_text(dialog, child_node, 10);
                    if child_result.handle.as_ref() == Some(parent_handle) {
                        // Parent resolves to same text as child - skip parent, process children
                        // If child is VisualState, follow through to its content children
                        for child_uuid in &node.children {
                            if let Some(child_node) = dialog.get_node(child_uuid) {
                                if child_node.constructor == NodeConstructor::VisualState {
                                    // Follow through VisualState to content children
                                    for grandchild_uuid in &child_node.children {
                                        if visited.contains(grandchild_uuid) {
                                            build_link_node(dialog, grandchild_uuid, parent_uuid, depth, parent_expanded, nodes);
                                        } else {
                                            build_node_tree(dialog, grandchild_uuid, parent_uuid, depth, parent_expanded, nodes, visited, alias_logicalnames, sibling_map, ancestor_siblings);
                                        }
                                    }
                                    continue;
                                }
                            }

                            if visited.contains(child_uuid) {
                                build_link_node(dialog, child_uuid, parent_uuid, depth, parent_expanded, nodes);
                            } else {
                                build_node_tree(dialog, child_uuid, parent_uuid, depth, parent_expanded, nodes, visited, alias_logicalnames, sibling_map, ancestor_siblings);
                            }
                        }
                        return;
                    }
                }
            }
        }
    }

    // Use original children - no flattening
    // For Jump nodes with no direct children (but with flags, so not pass-through),
    // use the target's children via the helper function
    let base_children = if node.constructor == NodeConstructor::Jump && node.children.is_empty() {
        if let Some(ref target_uuid) = node.jump_target {
            if let Some(target_node) = dialog.get_node(target_uuid) {
                get_jump_effective_children(dialog, target_node, node.jump_target_point)
            } else {
                node.children.clone()
            }
        } else {
            node.children.clone()
        }
    } else {
        node.children.clone()
    };

    // Resolve pass-through children to get the actual nodes that will be displayed
    // This ensures display.children reflects what's actually shown in the tree
    let effective_children = resolve_passthrough_children(dialog, &base_children);

    let inherited_state_contexts: Vec<String> = Vec::new();

    let index = nodes.len();
    let mut display = DisplayNode::new(index, uuid.to_string(), node.constructor.clone());

    display.parent_uuid = parent_uuid.map(|s| s.to_string());
    display.children = effective_children.clone();
    display.depth = depth;
    display.child_count = effective_children.len();

    display.is_end_node = node.end_node;
    display.has_flags = !node.check_flags.is_empty() || !node.set_flags.is_empty();

    // Copy check_flags - store UUIDs for later resolution
    for flag_group in &node.check_flags {
        for flag in &flag_group.flags {
            display.check_flags.push(DisplayFlag {
                // Store UUID for now, will be resolved to name later
                name: flag.name.clone().unwrap_or_else(|| format!("__UUID__:{}", flag.uuid)),
                value: flag.value,
                param_val: flag.param_val,
            });
        }
    }

    // Copy set_flags - store UUIDs for later resolution
    for flag_group in &node.set_flags {
        for flag in &flag_group.flags {
            display.set_flags.push(DisplayFlag {
                name: flag.name.clone().unwrap_or_else(|| format!("__UUID__:{}", flag.uuid)),
                value: flag.value,
                param_val: flag.param_val,
            });
        }
    }

    // Add logicalname as a pseudo-flag for nodes with "Alias (XXX)" or "INCLUSION_XXX" style names
    // This helps distinguish companion-specific branches
    // First, check the node's own logicalname
    let own_logicalname = node.editor_data.get("logicalname").and_then(|logicalname| {
        if logicalname.starts_with("Alias (") && logicalname.ends_with(")") {
            // Extract inner name: "Alias (INCLUSION_LAE'ZEL)" -> "INCLUSION_LAE'ZEL"
            Some(logicalname[7..logicalname.len()-1].to_string())
        } else if logicalname.starts_with("INCLUSION_") {
            Some(logicalname.clone())
        } else {
            None
        }
    });

    // Also check if any Alias node references this node and has an INCLUSION logicalname
    let inherited_logicalname = alias_logicalnames.get(uuid).cloned();

    // Prefer inherited logicalname (from Alias that references this node), fallback to own
    let display_name = inherited_logicalname.or(own_logicalname);

    if let Some(name) = display_name {
        display.check_flags.insert(0, DisplayFlag {
            name,
            value: true,
            param_val: None,
        });
        display.has_flags = true;
    }

    // Set visibility based on parent's expansion state
    // Root nodes (depth 0) are always visible
    display.is_visible = RwSignal::new(depth == 0 || parent_expanded);

    // Start collapsed for nodes with children (except root nodes)
    let this_expanded = if depth > 0 && !node.children.is_empty() {
        display.is_expanded = RwSignal::new(false);
        false
    } else {
        true
    };

    // Detect "logic nodes" - structural branch points with no dialogue text
    // These should only show flags, not speaker or text
    let is_logic_node = !has_text && node.children.len() > 1;

    // Get speaker name - store speaker_list UUIDs for later resolution
    // Skip for logic nodes (they're structural, not dialogue)
    if !is_logic_node && let Some(speaker_idx) = node.speaker {
        if speaker_idx == -666 {
            display.speaker_name = "Narrator".to_string();
        } else if speaker_idx >= 0 {
            if let Some(speaker) = dialog.get_speaker(speaker_idx) {
                // The speaker_list contains GlobalTemplate UUIDs that map to RootTemplates
                // Store them as special marker for resolution in resolve_speaker_names()
                // Filter out invalid entries like "@" or empty strings
                let valid_uuids: Vec<_> = speaker.speaker_list.iter()
                    .filter(|s| !s.is_empty() && *s != "@" && s.len() > 8)
                    .cloned()
                    .collect();

                if !valid_uuids.is_empty() {
                    // Join multiple UUIDs with semicolon for resolution
                    display.speaker_name = format!("__UUID__:{}", valid_uuids.join(";"));
                } else if !speaker.speaker_mapping_id.is_empty()
                    && speaker.speaker_mapping_id != "@"
                    && speaker.speaker_mapping_id.len() > 8
                {
                    // Fallback to speaker_mapping_id if no list (and it's a valid UUID)
                    display.speaker_name = format!("__UUID__:{}", speaker.speaker_mapping_id);
                } else {
                    // No valid speaker info, leave empty (don't show placeholder)
                    display.speaker_name = String::new();
                }
            } else {
                display.speaker_name = format!("Speaker {}", speaker_idx);
            }
        }
    }

    // Get primary text
    if let Some(text_entry) = dialog.get_node_text(node) {
        display.text_handle = Some(text_entry.handle.clone());
        if let Some(ref value) = text_entry.value {
            if !value.is_empty() {
                display.text = value.clone();
            } else {
                // Show handle for lookup
                display.text = format!("Handle: {}", text_entry.handle);
            }
        } else {
            // Show handle - this needs localization lookup
            display.text = format!("Handle: {}", text_entry.handle);
        }
    } else {
        // No text - leave empty, let children (Jump/Alias) display their own content
        display.text = String::new();
    }

    // Build roll info if applicable
    if node.is_roll() {
        let mut parts = Vec::new();
        if let Some(ref skill) = node.skill {
            parts.push(skill.clone());
        }
        if let Some(ref ability) = node.ability {
            parts.push(ability.clone());
        }
        if let Some(ref dc) = node.difficulty_class_id {
            parts.push(format!("DC: {}", dc));
        }
        if !parts.is_empty() {
            display.roll_info = Some(parts.join(" / "));
        }
    }

    // Copy editor data (dev notes) from the dialog node
    display.editor_data = node.editor_data.clone();

    // Add inherited stateContext from flattened VisualState children
    if !inherited_state_contexts.is_empty() {
        display.editor_data.insert(
            "stateContext".to_string(),
            inherited_state_contexts.join(" | ")
        );
    }

    // For RollResult nodes, capture the success/failure flag and copy roll info from parent
    if node.constructor == NodeConstructor::RollResult {
        display.roll_success = node.success;

        // Look up parent node to get roll details
        if let Some(parent_id) = parent_uuid {
            if let Some(parent_node) = dialog.get_node(parent_id) {
                // Copy roll info from parent if it's a roll node
                if parent_node.is_roll() {
                    let mut parts = Vec::new();
                    if let Some(ref skill) = parent_node.skill {
                        parts.push(skill.clone());
                    }
                    if let Some(ref ability) = parent_node.ability {
                        parts.push(format!("({})", ability));
                    }
                    if let Some(ref dc_id) = parent_node.difficulty_class_id {
                        // Store DC UUID for later resolution
                        parts.push(format!("DC:__UUID__:{}", dc_id));
                    }
                    if !parts.is_empty() {
                        display.roll_info = Some(parts.join(" "));
                    }
                }
            }
        }
    }

    // Handle Jump nodes - store target info for resolution
    if node.constructor == NodeConstructor::Jump {
        if let Some(ref target_uuid) = node.jump_target {
            display.jump_target_uuid = Some(target_uuid.clone());
            // Try to get the target node's text and speaker
            if let Some(target_node) = dialog.get_node(target_uuid) {
                // If jump_target_point is set, go to that child index, then keep following
                // first children until we hit a Jump/Alias
                let effective_target = if let Some(point) = node.jump_target_point {
                    let child_index = point as usize;
                    if child_index < target_node.children.len() {
                        if let Some(entry_child) = dialog.get_node(&target_node.children[child_index]) {
                            // Keep following first children until we hit a Jump/Alias
                            let mut entry_node = entry_child;
                            loop {
                                if matches!(entry_node.constructor, NodeConstructor::Jump | NodeConstructor::Alias) {
                                    break;
                                }
                                if entry_node.children.is_empty() {
                                    break;
                                }
                                if let Some(child) = dialog.get_node(&entry_node.children[0]) {
                                    entry_node = child;
                                } else {
                                    break;
                                }
                            }
                            entry_node
                        } else {
                            target_node
                        }
                    } else {
                        target_node
                    }
                } else {
                    target_node
                };

                // Copy speaker from effective target if Jump node doesn't have its own
                if display.speaker_name.is_empty() {
                    if let Some(speaker_idx) = effective_target.speaker {
                        if speaker_idx == -666 {
                            display.speaker_name = "Narrator".to_string();
                        } else if speaker_idx >= 0 {
                            if let Some(speaker) = dialog.get_speaker(speaker_idx) {
                                let valid_uuids: Vec<_> = speaker.speaker_list.iter()
                                    .filter(|s| !s.is_empty() && *s != "@" && s.len() > 8)
                                    .cloned()
                                    .collect();
                                if !valid_uuids.is_empty() {
                                    display.speaker_name = format!("__UUID__:{}", valid_uuids.join(";"));
                                } else if !speaker.speaker_mapping_id.is_empty()
                                    && speaker.speaker_mapping_id != "@"
                                    && speaker.speaker_mapping_id.len() > 8
                                {
                                    display.speaker_name = format!("__UUID__:{}", speaker.speaker_mapping_id);
                                }
                            }
                        }
                    }
                }
                // Follow chain from effective target to find text
                let result = follow_chain_for_text(dialog, effective_target, 10);

                if let Some(ref handle) = result.handle {
                    display.jump_target_handle = Some(handle.clone());
                    if let Some(ref value) = result.value {
                        if !value.is_empty() {
                            display.text = format!("→ {}", value);
                        } else {
                            display.text = format!("→ Handle: {}", handle);
                        }
                    } else {
                        display.text = format!("→ Handle: {}", handle);
                    }
                } else if result.is_visual {
                    display.text = "→ [Visual]".to_string();
                } else if !result.fallback.is_empty() {
                    display.text = format!("→ {}", result.fallback);
                }
            }
            // If target not found or had no text, show shortened UUID as fallback
            if !display.text.starts_with("→") {
                let short_uuid = &target_uuid[..8.min(target_uuid.len())];
                display.text = format!("→ ({}...)", short_uuid);
            }
        }
    }

    // Handle Alias nodes - store source info for resolution
    if node.constructor == NodeConstructor::Alias {
        if let Some(ref source_uuid) = node.source_node {
            display.jump_target_uuid = Some(source_uuid.clone());
            // Try to get the source node's text and speaker
            if let Some(source_node) = dialog.get_node(source_uuid) {
                // Copy speaker from source if Alias node doesn't have its own
                if display.speaker_name.is_empty() {
                    if let Some(speaker_idx) = source_node.speaker {
                        if speaker_idx == -666 {
                            display.speaker_name = "Narrator".to_string();
                        } else if speaker_idx >= 0 {
                            if let Some(speaker) = dialog.get_speaker(speaker_idx) {
                                let valid_uuids: Vec<_> = speaker.speaker_list.iter()
                                    .filter(|s| !s.is_empty() && *s != "@" && s.len() > 8)
                                    .cloned()
                                    .collect();
                                if !valid_uuids.is_empty() {
                                    display.speaker_name = format!("__UUID__:{}", valid_uuids.join(";"));
                                } else if !speaker.speaker_mapping_id.is_empty()
                                    && speaker.speaker_mapping_id != "@"
                                    && speaker.speaker_mapping_id.len() > 8
                                {
                                    display.speaker_name = format!("__UUID__:{}", speaker.speaker_mapping_id);
                                }
                            }
                        }
                    }
                }
                // Follow chain from source to find text
                let result = follow_chain_for_text(dialog, source_node, 10);

                if let Some(ref handle) = result.handle {
                    display.jump_target_handle = Some(handle.clone());
                    if let Some(ref value) = result.value {
                        if !value.is_empty() {
                            display.text = format!("= {}", value);
                        } else {
                            display.text = format!("= Handle: {}", handle);
                        }
                    } else {
                        display.text = format!("= Handle: {}", handle);
                    }
                } else if result.is_visual {
                    display.text = "= [Visual]".to_string();
                } else if !result.fallback.is_empty() {
                    display.text = format!("= {}", result.fallback);
                }
            }
            // If source not found or had no text, show shortened UUID as fallback
            if !display.text.starts_with("=") {
                let short_uuid = &source_uuid[..8.min(source_uuid.len())];
                display.text = format!("= ({}...)", short_uuid);
            }
        }
    }

    nodes.push(display);

    // Process children - they're visible only if this node is expanded AND visible
    // Use effective_children which has VisualState nodes flattened out
    let children_visible = this_expanded && (depth == 0 || parent_expanded);

    // Build new ancestor_siblings for children: merge current node's siblings with existing
    // This tracks siblings at all ancestor levels so we can detect when a node appears
    // as both a sibling and a descendant of that sibling
    let mut child_ancestor_siblings = ancestor_siblings.clone();
    if let Some(my_siblings) = sibling_map.get(uuid) {
        child_ancestor_siblings.extend(my_siblings.iter().cloned());
    }

    // Process each effective child
    // effective_children already has pass-through nodes resolved to content nodes
    for child_uuid in &effective_children {
        // Check if this child was already visited (shared node / link)
        if visited.contains(child_uuid) {
            // Get the effective content nodes, following through any pass-through chain
            let content_nodes = get_content_nodes_for_link(dialog, child_uuid);

            // Create links to each content node
            for content_uuid in &content_nodes {
                build_link_node(dialog, content_uuid, Some(uuid), depth + 1, children_visible, nodes);
            }
        } else {
            build_node_tree(dialog, child_uuid, Some(uuid), depth + 1, children_visible, nodes, visited, alias_logicalnames, sibling_map, &child_ancestor_siblings);
        }
    }

    // FALLBACK: If effective_children differs from original children, also process
    // the content nodes that should be under this node. This handles the case where
    // resolve_passthrough_children returned empty or wrong results.
    if effective_children != base_children {
        // Already handled by effective_children processing above
    } else if base_children.len() == 1 {
        // If we have a single child that might be pass-through, ensure its content is linked
        let child_uuid = &base_children[0];
        if let Some(child_node) = dialog.get_node(child_uuid) {
            let child_has_text = dialog.get_node_text(child_node).is_some();
            let child_has_flags = child_node.check_flags.iter().any(|fg| !fg.flags.is_empty())
                || child_node.set_flags.iter().any(|fg| !fg.flags.is_empty());

            // If child is pass-through (no text, no flags, leads to VisualState or Jump/Alias)
            if !child_has_text && !child_has_flags {
                if child_node.constructor == NodeConstructor::VisualState
                    || (child_node.children.len() == 1 && dialog.get_node(&child_node.children[0]).map_or(false, |gc| gc.constructor == NodeConstructor::VisualState))
                {
                    // Get content nodes through this pass-through chain
                    let content_nodes = get_content_nodes_for_link(dialog, child_uuid);
                    for content_uuid in &content_nodes {
                        if !effective_children.contains(content_uuid) {
                            if visited.contains(content_uuid) {
                                build_link_node(dialog, content_uuid, Some(uuid), depth + 1, children_visible, nodes);
                            } else {
                                build_node_tree(dialog, content_uuid, Some(uuid), depth + 1, children_visible, nodes, visited, alias_logicalnames, sibling_map, &child_ancestor_siblings);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Get the actual content nodes to link to, following through pass-through chains
/// Pass-through nodes include: VisualState, flagless Jumps, and nodes with no text + single VisualState child
fn get_content_nodes_for_link(dialog: &Dialog, uuid: &str) -> Vec<String> {
    get_content_nodes_for_link_depth(dialog, uuid, 0)
}

/// Internal helper with depth tracking to prevent deep recursion
fn get_content_nodes_for_link_depth(dialog: &Dialog, uuid: &str, depth: usize) -> Vec<String> {
    // Limit expansion depth to prevent cascading through many levels
    const MAX_DEPTH: usize = 2;

    let Some(node) = dialog.get_node(uuid) else {
        return vec![uuid.to_string()];
    };

    let has_text = dialog.get_node_text(node).is_some();

    // VisualState is always pass-through (but respect depth limit)
    if node.constructor == NodeConstructor::VisualState {
        if depth >= MAX_DEPTH {
            return node.children.clone();
        }
        let mut result = Vec::new();
        for child_uuid in &node.children {
            result.extend(get_content_nodes_for_link_depth(dialog, child_uuid, depth + 1));
        }
        return if result.is_empty() { vec![uuid.to_string()] } else { result };
    }

    // Jump without flags and empty children is pass-through
    let has_actual_flags = node.check_flags.iter().any(|fg| !fg.flags.is_empty())
        || node.set_flags.iter().any(|fg| !fg.flags.is_empty());
    if node.constructor == NodeConstructor::Jump
        && !has_actual_flags
        && node.children.is_empty()
    {
        if depth >= MAX_DEPTH {
            return vec![uuid.to_string()];
        }
        if let Some(ref target_uuid) = node.jump_target {
            if let Some(target_node) = dialog.get_node(target_uuid) {
                let effective = get_jump_effective_children(dialog, target_node, node.jump_target_point);
                let mut result = Vec::new();
                for child_uuid in &effective {
                    result.extend(get_content_nodes_for_link_depth(dialog, child_uuid, depth + 1));
                }
                return if result.is_empty() { vec![uuid.to_string()] } else { result };
            }
        }
        return vec![uuid.to_string()];
    }

    // Node with no text and single child - follow through if child is pass-through
    if !has_text && node.children.len() == 1 {
        if depth >= MAX_DEPTH {
            return vec![uuid.to_string()];
        }
        if let Some(child) = dialog.get_node(&node.children[0]) {
            // Check if child is VisualState
            if child.constructor == NodeConstructor::VisualState {
                return get_content_nodes_for_link_depth(dialog, &node.children[0], depth + 1);
            }
            // Check if child is a pass-through Jump (flagless, no children of its own)
            let child_has_actual_flags = child.check_flags.iter().any(|fg| !fg.flags.is_empty())
                || child.set_flags.iter().any(|fg| !fg.flags.is_empty());
            if child.constructor == NodeConstructor::Jump
                && !child_has_actual_flags
                && child.children.is_empty()
            {
                // Follow through the Jump
                return get_content_nodes_for_link_depth(dialog, &node.children[0], depth + 1);
            }
            // Don't follow through any single child here - the node should be linked to directly
            // This ensures links show the appropriate level of content, not grandchildren
        }
    }

    // Node with no text and multiple children - just show the children (skip the parent)
    // The parent has no content, so linking to it is redundant when we show all children
    // Only expand one level - don't recurse into children's children
    if !has_text && node.children.len() > 1 {
        // Return children directly without recursive expansion
        return node.children.clone();
    }

    // Not pass-through, return this node
    vec![uuid.to_string()]
}

/// Build a link node for an already-visited child (shared node)
/// This creates a display node that shows the target's speaker and text with → prefix
fn build_link_node(
    dialog: &Dialog,
    target_uuid: &str,
    parent_uuid: Option<&str>,
    depth: usize,
    parent_expanded: bool,
    nodes: &mut Vec<DisplayNode>,
) {
    // Don't create links at the top level - only real nodes should be root nodes
    if parent_uuid.is_none() {
        return;
    }

    let Some(target_node) = dialog.get_node(target_uuid) else {
        return;
    };

    let index = nodes.len();
    // Use a unique UUID for this link node (original UUID + "_link" + index to ensure uniqueness)
    let link_uuid = format!("{}_link_{}", target_uuid, index);
    let mut display = DisplayNode::new(index, link_uuid, NodeConstructor::Other("Link".to_string()));

    display.parent_uuid = parent_uuid.map(|s| s.to_string());
    display.depth = depth;
    display.child_count = 0; // Links don't show children (they're displayed under the original)
    display.children = Vec::new();
    display.jump_target_uuid = Some(target_uuid.to_string());

    // Set visibility based on parent's expansion state
    display.is_visible = RwSignal::new(depth == 0 || parent_expanded);
    display.is_expanded = RwSignal::new(false); // Links are not expandable

    // Copy speaker from target node
    if let Some(speaker_idx) = target_node.speaker {
        if speaker_idx == -666 {
            display.speaker_name = "Narrator".to_string();
        } else if speaker_idx >= 0 {
            if let Some(speaker) = dialog.get_speaker(speaker_idx) {
                let valid_uuids: Vec<_> = speaker.speaker_list.iter()
                    .filter(|s| !s.is_empty() && *s != "@" && s.len() > 8)
                    .cloned()
                    .collect();
                if !valid_uuids.is_empty() {
                    display.speaker_name = format!("__UUID__:{}", valid_uuids.join(";"));
                } else if !speaker.speaker_mapping_id.is_empty()
                    && speaker.speaker_mapping_id != "@"
                    && speaker.speaker_mapping_id.len() > 8
                {
                    display.speaker_name = format!("__UUID__:{}", speaker.speaker_mapping_id);
                }
            }
        }
    }

    // Get text from target node using follow_chain_for_text
    let result = follow_chain_for_text(dialog, target_node, 10);
    if let Some(ref handle) = result.handle {
        display.jump_target_handle = Some(handle.clone());
        if let Some(ref value) = result.value {
            if !value.is_empty() {
                display.text = format!("→ {}", value);
            } else {
                display.text = format!("→ Handle: {}", handle);
            }
        } else {
            display.text = format!("→ Handle: {}", handle);
        }
    } else if result.is_visual {
        display.text = "→ [Visual]".to_string();
    } else if !result.fallback.is_empty() {
        display.text = format!("→ {}", result.fallback);
    } else {
        // Fallback to shortened UUID
        let short_uuid = &target_uuid[..8.min(target_uuid.len())];
        display.text = format!("→ ({}...)", short_uuid);
    }

    nodes.push(display);
}
