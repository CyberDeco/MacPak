//! Resolution functions for converting UUIDs to display names

use crate::dialog::NodeConstructor;
use crate::gui::state::{DialogueState, DisplayNode};

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
        // Check for UUID
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

        // Resolve jump/alias/link target text if there's a texthandle
        if let Some(ref handle) = node.jump_target_handle {
            let is_jump = node.constructor == NodeConstructor::Jump;
            let is_alias = node.constructor == NodeConstructor::Alias;
            let is_link = matches!(&node.constructor, NodeConstructor::Other(s) if s == "Link");

            if is_jump || is_alias || is_link {
                let prefix = if is_alias { "=" } else { "→" }; // Jump and Link both use →

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
                    // For Jump/Alias/Link, keep the prefix
                    let is_link = matches!(&node.constructor, NodeConstructor::Other(s) if s == "Link");
                    if node.constructor == NodeConstructor::Jump || is_link {
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
