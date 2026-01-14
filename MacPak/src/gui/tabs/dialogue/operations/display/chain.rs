//! Chain traversal helpers for following node chains to find text

use MacLarian::dialog::{Dialog, DialogNode, NodeConstructor};

/// Result of following a node chain to find text
pub struct ChainResult {
    /// The text handle if found
    pub handle: Option<String>,
    /// Inline text value if available
    pub value: Option<String>,
    /// Fallback display (node ID or type) if no text found
    pub fallback: String,
    /// Whether we hit a VisualState node
    pub is_visual: bool,
}

/// Follow a chain of nodes (through Jump, Alias, or single children) to find text
/// Returns the first text found, or a fallback description
pub fn follow_chain_for_text(dialog: &Dialog, start_node: &DialogNode, max_depth: usize) -> ChainResult {
    let mut current = start_node;
    let mut depth = 0;

    loop {
        if depth >= max_depth {
            break;
        }
        depth += 1;

        // Check if current node has text
        if let Some(text_entry) = dialog.get_node_text(current) {
            return ChainResult {
                handle: Some(text_entry.handle.clone()),
                value: text_entry.value.clone(),
                fallback: String::new(),
                is_visual: false,
            };
        }

        // Check for VisualState - try to get text from first child before giving up
        if current.constructor == NodeConstructor::VisualState {
            // If VisualState has children, try to find text in the first child
            if !current.children.is_empty() {
                if let Some(first_child) = dialog.get_node(&current.children[0]) {
                    if let Some(text_entry) = dialog.get_node_text(first_child) {
                        return ChainResult {
                            handle: Some(text_entry.handle.clone()),
                            value: text_entry.value.clone(),
                            fallback: String::new(),
                            is_visual: true, // Still mark as visual for context
                        };
                    }
                }
            }
            // No text found in children, return [Visual] fallback
            return ChainResult {
                handle: None,
                value: None,
                fallback: "[Visual]".to_string(),
                is_visual: true,
            };
        }

        // Follow through RollResult to first child
        if current.constructor == NodeConstructor::RollResult {
            if !current.children.is_empty() {
                if let Some(child) = dialog.get_node(&current.children[0]) {
                    current = child;
                    continue;
                }
            }
            break;
        }

        // Try to follow through Jump
        if current.constructor == NodeConstructor::Jump {
            if let Some(ref target_uuid) = current.jump_target {
                if let Some(target) = dialog.get_node(target_uuid) {
                    // If jump_target_point is set, go to that child index, then keep following
                    // first children until we hit a Jump/Alias
                    if let Some(point) = current.jump_target_point {
                        let child_index = point as usize;
                        if child_index < target.children.len() {
                            if let Some(entry_child) = dialog.get_node(&target.children[child_index]) {
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
                                current = entry_node;
                                continue;
                            }
                        }
                    }
                    current = target;
                    continue;
                }
            }
            // Can't follow jump
            break;
        }

        // Try to follow through Alias
        if current.constructor == NodeConstructor::Alias {
            if let Some(ref source_uuid) = current.source_node {
                if let Some(source) = dialog.get_node(source_uuid) {
                    current = source;
                    continue;
                }
            }
            // Can't follow alias
            break;
        }

        // Only follow through single child if the child is a Jump or Alias (pass-through)
        // Don't follow through regular nodes with single children - those should display as-is
        if current.children.len() == 1 {
            if let Some(child) = dialog.get_node(&current.children[0]) {
                if matches!(child.constructor, NodeConstructor::Jump | NodeConstructor::Alias | NodeConstructor::VisualState) {
                    current = child;
                    continue;
                }
            }
        }

        // Stop here - either multiple children, no children, or single non-pass-through child
        break;
    }

    // No text found - return fallback based on where we ended up
    let fallback = current.editor_data.get("ID")
        .cloned()
        .unwrap_or_else(|| current.constructor.display_name().to_string());

    ChainResult {
        handle: None,
        value: None,
        fallback,
        is_visual: current.constructor == NodeConstructor::VisualState,
    }
}

/// Get effective children for a Jump node's target, handling VisualState pass-through
/// For display purposes, we show ALL content children (ignore jump_target_point for tree display)
/// The jump_target_point is only used for text resolution, not for determining children to display
pub fn get_jump_effective_children(dialog: &Dialog, target_node: &DialogNode, _jump_target_point: Option<i32>) -> Vec<String> {
    // If target is a VisualState, its children are the content nodes
    if target_node.constructor == NodeConstructor::VisualState {
        return target_node.children.clone();
    }

    // If target has a single VisualState child, follow through to that
    if target_node.children.len() == 1 {
        if let Some(child) = dialog.get_node(&target_node.children[0]) {
            if child.constructor == NodeConstructor::VisualState {
                return child.children.clone();
            }
        }
    }

    // Otherwise use target's children directly
    target_node.children.clone()
}

/// Resolve pass-through children to get the actual nodes that will be displayed
/// Pass-through nodes (no text, no flags, single Jump/Alias/VisualState child, or VisualState)
/// are followed through to find the actual content nodes that will appear in the tree
pub fn resolve_passthrough_children(dialog: &Dialog, children: &[String]) -> Vec<String> {
    let mut result = Vec::new();

    for child_uuid in children {
        let resolved = resolve_single_passthrough(dialog, child_uuid);
        result.extend(resolved);
    }

    result
}

/// Recursively resolve a single node through any pass-through chain
pub fn resolve_single_passthrough(dialog: &Dialog, uuid: &str) -> Vec<String> {
    let Some(node) = dialog.get_node(uuid) else {
        return vec![uuid.to_string()];
    };

    let has_text = dialog.get_node_text(node).is_some();
    // Check for actual flags, not just empty FlagGroups
    let has_flags = node.check_flags.iter().any(|fg| !fg.flags.is_empty())
        || node.set_flags.iter().any(|fg| !fg.flags.is_empty());

    // VisualState is always pass-through - return its children's resolutions
    if node.constructor == NodeConstructor::VisualState {
        let mut result = Vec::new();
        for child_uuid in &node.children {
            result.extend(resolve_single_passthrough(dialog, child_uuid));
        }
        return if result.is_empty() { vec![uuid.to_string()] } else { result };
    }

    // Check if this is a pass-through node: no text, no flags, single Alias/VisualState child
    // Note: We don't follow through to Jump here - let is_jump_alias_container handle it
    // so that the all-visited-content check can work properly
    if !has_text && !has_flags && node.children.len() == 1 {
        if let Some(child) = dialog.get_node(&node.children[0]) {
            if matches!(child.constructor, NodeConstructor::Alias | NodeConstructor::VisualState) {
                // Follow through to child
                return resolve_single_passthrough(dialog, &node.children[0]);
            }
        }
    }

    // Not pass-through, return this node
    vec![uuid.to_string()]
}
