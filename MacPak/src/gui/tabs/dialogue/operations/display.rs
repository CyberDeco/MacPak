//! Display node building - converts Dialog to DisplayNode list

use std::collections::HashSet;
use floem::prelude::RwSignal;
use MacLarian::dialog::{Dialog, DialogNode, NodeConstructor};
use crate::gui::state::{DialogueState, DisplayNode, DisplayFlag};

/// Result of following a node chain to find text
struct ChainResult {
    /// The text handle if found
    handle: Option<String>,
    /// Inline text value if available
    value: Option<String>,
    /// Fallback display (node ID or type) if no text found
    fallback: String,
    /// Whether we hit a VisualState node
    is_visual: bool,
}

/// Follow a chain of nodes (through Jump, Alias, or single children) to find text
/// Returns the first text found, or a fallback description
fn follow_chain_for_text(dialog: &Dialog, start_node: &DialogNode, max_depth: usize) -> ChainResult {
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

        // Check for VisualState
        if current.constructor == NodeConstructor::VisualState {
            return ChainResult {
                handle: None,
                value: None,
                fallback: "[Visual]".to_string(),
                is_visual: true,
            };
        }

        // Try to follow through Jump
        if current.constructor == NodeConstructor::Jump {
            if let Some(ref target_uuid) = current.jump_target {
                if let Some(target) = dialog.get_node(target_uuid) {
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

        // Try to follow through single child
        if current.children.len() == 1 {
            if let Some(child) = dialog.get_node(&current.children[0]) {
                current = child;
                continue;
            }
        }

        // Multiple children - stop here, don't trace (children shown as separate rows)
        if current.children.len() > 1 {
            break;
        }

        // No children and no text - stop here
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

/// Build display nodes from a dialog
pub fn build_display_nodes(dialog: &Dialog) -> Vec<DisplayNode> {
    let mut nodes = Vec::new();
    let mut visited = HashSet::new();

    // Start from root nodes (parent_expanded = true for root level)
    for root_uuid in &dialog.root_nodes {
        build_node_tree(dialog, root_uuid, None, 0, true, &mut nodes, &mut visited);
    }

    // Add any orphaned nodes
    for uuid in &dialog.node_order {
        if !visited.contains(uuid) {
            build_node_tree(dialog, uuid, None, 0, true, &mut nodes, &mut visited);
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
) {
    if visited.contains(uuid) {
        return;
    }
    visited.insert(uuid.to_string());

    let Some(node) = dialog.get_node(uuid) else {
        return;
    };

    // Use original children - no flattening
    let effective_children = node.children.clone();
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

    // Get speaker name - store speaker_list UUIDs for later resolution
    if let Some(speaker_idx) = node.speaker {
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
        // No text - check if this is a single Jump/Alias child container
        // (those should show the target text since they're effectively links)
        let mut resolved = false;

        if node.children.len() == 1 {
            if let Some(child_node) = dialog.get_node(&node.children[0]) {
                if child_node.constructor == NodeConstructor::Jump {
                    if let Some(ref target_uuid) = child_node.jump_target {
                        if let Some(target_node) = dialog.get_node(target_uuid) {
                            let result = follow_chain_for_text(dialog, target_node, 10);
                            if let Some(ref handle) = result.handle {
                                display.jump_target_handle = Some(handle.clone());
                                if let Some(ref value) = result.value {
                                    if !value.is_empty() {
                                        display.text = format!("↳ {}", value);
                                        resolved = true;
                                    }
                                }
                                if !resolved {
                                    display.text = format!("↳ Handle: {}", handle);
                                    resolved = true;
                                }
                            } else if result.is_visual {
                                display.text = "↳ [Visual]".to_string();
                                resolved = true;
                            }
                        }
                    }
                } else if child_node.constructor == NodeConstructor::Alias {
                    if let Some(ref source_uuid) = child_node.source_node {
                        if let Some(source_node) = dialog.get_node(source_uuid) {
                            let result = follow_chain_for_text(dialog, source_node, 10);
                            if let Some(ref handle) = result.handle {
                                display.jump_target_handle = Some(handle.clone());
                                if let Some(ref value) = result.value {
                                    if !value.is_empty() {
                                        display.text = format!("↳ {}", value);
                                        resolved = true;
                                    }
                                }
                                if !resolved {
                                    display.text = format!("↳ Handle: {}", handle);
                                    resolved = true;
                                }
                            } else if result.is_visual {
                                display.text = "↳ [Visual]".to_string();
                                resolved = true;
                            }
                        }
                    }
                }
            }
        }

        if !resolved {
            // Multi-child containers or unresolved - children shown as separate rows
            display.text = String::new();
        }
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
                // Copy speaker from target if Jump node doesn't have its own
                if display.speaker_name.is_empty() {
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
                }
                // Follow chain from target to find text
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
    for child_uuid in &effective_children {
        build_node_tree(dialog, child_uuid, Some(uuid), depth + 1, children_visible, nodes, visited);
    }
}

/// Resolve speaker names using dynamic speaker cache + runtime localization
pub fn resolve_speaker_names(state: &DialogueState, nodes: &mut [DisplayNode]) {
    let loca_cache = state.localization_cache.clone();
    let speaker_cache = state.speaker_cache.clone();

    let loca_cache = match loca_cache.read() {
        Ok(c) => c,
        Err(_) => return,
    };

    let speaker_cache = match speaker_cache.read() {
        Ok(c) => c,
        Err(_) => return,
    };

    for node in nodes.iter_mut() {
        // Check for our special UUID marker
        if node.speaker_name.starts_with("__UUID__:") {
            let uuids_str = &node.speaker_name[9..]; // Skip "__UUID__:" prefix

            // Handle multiple UUIDs separated by semicolons
            let uuids: Vec<&str> = uuids_str.split(';').collect();
            let mut resolved_names: Vec<String> = Vec::new();

            for uuid in &uuids {
                // Look up in dynamic speaker cache (loaded from PAK files)
                if let Some(handle) = speaker_cache.get_handle(uuid) {
                    // Check for hardcoded direct names (prefixed with __DIRECT__:)
                    if let Some(direct_name) = handle.strip_prefix("__DIRECT__:") {
                        resolved_names.push(direct_name.to_string());
                    } else {
                        // Resolve the handle to localized text
                        if let Some(localized) = loca_cache.get_text_opt(handle) {
                            resolved_names.push(localized);
                        }
                    }
                }
            }

            if !resolved_names.is_empty() {
                node.speaker_name = resolved_names.join(", ");
            } else {
                // Fallback to shortened first UUID
                let first_uuid = uuids.first().unwrap_or(&"");
                let short_id = &first_uuid[..8.min(first_uuid.len())];
                node.speaker_name = format!("({}...)", short_id);
            }
        }
    }
}

/// Resolve localized text using runtime localization cache
pub fn resolve_localized_text(state: &DialogueState, nodes: &mut [DisplayNode]) {
    let loca_cache = state.localization_cache.clone();

    let loca_cache = match loca_cache.read() {
        Ok(c) => c,
        Err(_) => return,
    };

    for node in nodes.iter_mut() {
        // If text shows "Handle: xxx", try to resolve it
        if node.text.starts_with("Handle: ") {
            if let Some(handle) = &node.text_handle {
                if let Some(text) = loca_cache.get_text_opt(handle) {
                    node.text = text;
                }
            }
        }

        // Resolve jump/alias target text if we have a handle
        if let Some(ref handle) = node.jump_target_handle {
            let is_jump = node.constructor == NodeConstructor::Jump;
            let is_alias = node.constructor == NodeConstructor::Alias;

            if is_jump || is_alias {
                let prefix = if is_jump { "→" } else { "=" };

                // Try to resolve if text shows UUID reference, handle, or node type placeholder
                let needs_resolution = node.text.starts_with(&format!("{} (", prefix))
                    || node.text.starts_with(&format!("{} Handle:", prefix))
                    || node.text == "[Jump node]"
                    || node.text == "[Alias node]";

                if needs_resolution {
                    if let Some(text) = loca_cache.get_text_opt(handle) {
                        node.text = format!("{} {}", prefix, text);
                    }
                }
            } else {
                // Container node (TagAnswer, etc.) that links through a Jump/Alias child
                let needs_resolution = node.text.starts_with("↳ Handle:")
                    || node.text.starts_with("↳ (");

                if needs_resolution {
                    if let Some(text) = loca_cache.get_text_opt(handle) {
                        node.text = format!("↳ {}", text);
                    }
                }
            }
        }

        // Also try to resolve the primary text handle if text is still showing as a placeholder
        if node.text.contains("Handle:") || node.text.starts_with("[") {
            if let Some(handle) = &node.text_handle {
                if let Some(text) = loca_cache.get_text_opt(handle) {
                    // For Jump/Alias, keep the prefix
                    if node.constructor == NodeConstructor::Jump {
                        node.text = format!("→ {}", text);
                    } else if node.constructor == NodeConstructor::Alias {
                        node.text = format!("= {}", text);
                    } else {
                        node.text = text;
                    }
                }
            }
        }
    }
}

/// Resolve flag UUIDs to human-readable names using the flag cache
/// Uses pre-indexed lookups (O(1) per flag)
pub fn resolve_flag_names(state: &DialogueState, nodes: &mut [DisplayNode]) {
    let flag_cache = state.flag_cache.clone();

    let flag_cache = match flag_cache.read() {
        Ok(c) => c,
        Err(_) => return,
    };

    for node in nodes.iter_mut() {
        // Resolve check_flags
        for flag in node.check_flags.iter_mut() {
            if flag.name.starts_with("__UUID__:") {
                let uuid = flag.name[9..].to_string(); // Skip "__UUID__:" prefix
                if let Some(name) = flag_cache.get_name(&uuid) {
                    flag.name = name.to_string();
                } else {
                    // Fallback to shortened UUID
                    let short_id = &uuid[..8.min(uuid.len())];
                    flag.name = format!("({}...)", short_id);
                }
            }
        }

        // Resolve set_flags
        for flag in node.set_flags.iter_mut() {
            if flag.name.starts_with("__UUID__:") {
                let uuid = flag.name[9..].to_string(); // Skip "__UUID__:" prefix
                if let Some(name) = flag_cache.get_name(&uuid) {
                    flag.name = name.to_string();
                } else {
                    // Fallback to shortened UUID
                    let short_id = &uuid[..8.min(uuid.len())];
                    flag.name = format!("({}...)", short_id);
                }
            }
        }
    }
}

/// Resolve difficulty class UUIDs in roll_info to numeric DC values
/// Uses pre-indexed lookups (O(1) per DC)
pub fn resolve_difficulty_classes(state: &DialogueState, nodes: &mut [DisplayNode]) {
    let dc_cache = state.difficulty_class_cache.clone();

    let dc_cache = match dc_cache.read() {
        Ok(c) => c,
        Err(_) => return,
    };

    for node in nodes.iter_mut() {
        // Resolve DC UUIDs in roll_info
        if let Some(ref mut roll_info) = node.roll_info {
            // Check for DC:__UUID__: pattern
            if roll_info.contains("DC:__UUID__:") {
                // Find and replace the UUID with resolved DC value
                if let Some(start) = roll_info.find("DC:__UUID__:") {
                    let uuid_start = start + "DC:__UUID__:".len();
                    // UUID is 36 chars (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)
                    if uuid_start + 36 <= roll_info.len() {
                        let uuid = &roll_info[uuid_start..uuid_start + 36];
                        if let Some(formatted) = dc_cache.get_formatted(uuid) {
                            // Replace "DC:__UUID__:uuid" with resolved DC
                            let old_pattern = format!("DC:__UUID__:{}", uuid);
                            *roll_info = roll_info.replace(&old_pattern, &formatted);
                        } else {
                            // Fallback: just show "DC ?"
                            let old_pattern = format!("DC:__UUID__:{}", uuid);
                            *roll_info = roll_info.replace(&old_pattern, "DC ?");
                        }
                    }
                }
            }
        }
    }
}
