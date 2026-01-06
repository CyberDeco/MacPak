//! Display node building - converts Dialog to DisplayNode list

use std::collections::HashSet;
use floem::prelude::RwSignal;
use MacLarian::formats::dialog::{Dialog, NodeConstructor, embedded_speakers};
use crate::gui::state::{DialogueState, DisplayNode};

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

    let index = nodes.len();
    let mut display = DisplayNode::new(index, uuid.to_string(), node.constructor.clone());

    display.parent_uuid = parent_uuid.map(|s| s.to_string());
    display.children = node.children.clone();
    display.depth = depth;
    display.child_count = node.children.len();
    display.is_end_node = node.end_node;
    display.has_flags = !node.check_flags.is_empty() || !node.set_flags.is_empty();

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
        // No text - show node type info instead
        display.text = format!("[{} node]", node.constructor.display_name());
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

    // For RollResult nodes, capture the success/failure flag
    if node.constructor == NodeConstructor::RollResult {
        display.roll_success = node.success;
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
                // Copy text from target
                if let Some(text_entry) = dialog.get_node_text(target_node) {
                    display.jump_target_handle = Some(text_entry.handle.clone());
                    // If we have inline text, use it directly
                    if let Some(ref value) = text_entry.value {
                        if !value.is_empty() {
                            display.text = format!("→ {}", value);
                        }
                    }
                }
            }
            // If no text resolved, show target UUID
            if display.text.is_empty() || display.text == "[Jump node]" {
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
                // Copy text from source
                if let Some(text_entry) = dialog.get_node_text(source_node) {
                    display.jump_target_handle = Some(text_entry.handle.clone());
                    // If we have inline text, use it directly
                    if let Some(ref value) = text_entry.value {
                        if !value.is_empty() {
                            display.text = format!("= {}", value);
                        }
                    }
                }
            }
            // If no text resolved, show source UUID
            if display.text.is_empty() || display.text == "[Alias node]" {
                let short_uuid = &source_uuid[..8.min(source_uuid.len())];
                display.text = format!("= ({}...)", short_uuid);
            }
        }
    }

    nodes.push(display);

    // Process children - they're visible only if this node is expanded AND visible
    let children_visible = this_expanded && (depth == 0 || parent_expanded);
    for child_uuid in &node.children {
        build_node_tree(dialog, child_uuid, Some(uuid), depth + 1, children_visible, nodes, visited);
    }
}

/// Resolve speaker names using embedded speaker database + runtime localization
pub fn resolve_speaker_names(state: &DialogueState, nodes: &mut [DisplayNode]) {
    let speakers = embedded_speakers();
    let loca_cache = state.localization_cache.clone();

    let loca_cache = match loca_cache.read() {
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
                // First check embedded companion names
                if let Some(name) = speakers.get_companion_name(uuid) {
                    resolved_names.push(name.to_string());
                }
                // Then try embedded handle + runtime localization
                else if let Some(handle) = speakers.get_display_handle(uuid) {
                    if let Some(localized) = loca_cache.get_text_opt(handle) {
                        resolved_names.push(localized);
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
