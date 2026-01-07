//! Dialog loading and parsing operations

use std::path::PathBuf;
use std::sync::Arc;
use floem::reactive::{SignalGet, SignalUpdate};
use MacLarian::formats::dialog::{parse_dialog_file, parse_dialog_bytes, Dialog};
use MacLarian::formats::lsf::parse_lsf_bytes;
use MacLarian::converter::{to_lsx, to_lsj};
use MacLarian::formats::lsx::parse_lsx;
use MacLarian::pak::PakOperations;
use crate::gui::state::{DialogueState, DialogSource, DialogEntry, DisplayNode};
use super::display::{build_display_nodes, resolve_speaker_names, resolve_localized_text, resolve_flag_names};

/// Load a dialog from a PAK file (runs synchronously for UI updates)
/// Handles both .lsf (binary) and .lsj (JSON) formats
pub fn load_dialog_from_pak(state: DialogueState, pak_path: PathBuf, internal_path: String) {
    state.status_message.set("Loading dialog from PAK...".to_string());
    state.is_loading.set(true);

    match PakOperations::read_file_bytes(&pak_path, &internal_path) {
        Ok(data) => {
            let lower_path = internal_path.to_lowercase();
            let result = if lower_path.ends_with(".lsf") {
                // Parse LSF (binary) format - convert through LSX to LSJ
                parse_lsf_dialog(&data)
            } else {
                // Parse LSJ (JSON) format directly
                parse_dialog_bytes(&data).map_err(|e| e.to_string())
            };

            match result {
                Ok(dialog) => {
                    process_loaded_dialog(state.clone(), dialog);
                    state.status_message.set("Dialog loaded".to_string());
                }
                Err(e) => {
                    state.status_message.set(format!("Parse error: {}", e));
                    state.error_message.set(Some(e));
                }
            }
        }
        Err(e) => {
            state.status_message.set(format!("Load error: {}", e));
            state.error_message.set(Some(format!("{}", e)));
        }
    }

    state.is_loading.set(false);
}

/// Parse an LSF dialog file by converting through LSX → LSJ → Dialog
fn parse_lsf_dialog(data: &[u8]) -> Result<Dialog, String> {
    use MacLarian::formats::dialog::parse_dialog;

    // Parse LSF binary
    let lsf_doc = parse_lsf_bytes(data)
        .map_err(|e| format!("LSF parse error: {}", e))?;

    // Convert to LSX XML string
    let lsx_xml = to_lsx(&lsf_doc)
        .map_err(|e| format!("LSF→LSX error: {}", e))?;

    // Parse LSX XML to document
    let lsx_doc = parse_lsx(&lsx_xml)
        .map_err(|e| format!("LSX parse error: {}", e))?;

    // Convert to LSJ
    let lsj_doc = to_lsj(&lsx_doc)
        .map_err(|e| format!("LSX→LSJ error: {}", e))?;

    // Parse dialog from LSJ
    parse_dialog(&lsj_doc)
        .map_err(|e| format!("Dialog parse error: {}", e))
}

/// Load a dialog from a path (runs synchronously for UI updates)
pub fn load_dialog(state: DialogueState, path: String) {
    // Find the entry
    let entry = state.available_dialogs.get()
        .into_iter()
        .find(|e| e.path == path);

    let Some(entry) = entry else {
        state.status_message.set("Dialog not found".to_string());
        return;
    };

    load_dialog_entry(state, entry);
}

/// Load a dialog from an entry directly (avoids re-reading available_dialogs)
pub fn load_dialog_entry(state: DialogueState, entry: DialogEntry) {
    state.selected_dialog_path.set(Some(entry.path.clone()));

    match &entry.source {
        DialogSource::LocalFile(file_path) => {
            state.status_message.set("Loading dialog...".to_string());
            state.is_loading.set(true);

            match parse_dialog_file(file_path) {
                Ok(dialog) => {
                    process_loaded_dialog(state.clone(), dialog);
                    state.status_message.set("Dialog loaded".to_string());
                }
                Err(e) => {
                    state.status_message.set(format!("Error: {}", e));
                    state.error_message.set(Some(format!("{}", e)));
                }
            }

            state.is_loading.set(false);
        }
        DialogSource::PakFile { pak_path, internal_path } => {
            load_dialog_from_pak(state, pak_path.clone(), internal_path.clone());
        }
    }
}

/// Calculate the content width for a node (must match tree_view.rs calculation)
fn calculate_node_content_width(node: &DisplayNode) -> f32 {
    let indent = (node.depth * 20) as f32;
    let expand_icon = 16.0;
    let badge = 30.0;
    let speaker_width = if node.speaker_name.is_empty() { 0.0 } else { (node.speaker_name.len() as f32 * 8.0) + 20.0 };
    let text_width = (node.text.len() as f32 * 7.0).max(50.0);
    let end_indicator = if node.is_end_node { 40.0 } else { 0.0 };
    let padding_and_gaps = 60.0;
    indent + expand_icon + badge + speaker_width + text_width + end_indicator + padding_and_gaps
}

/// Process a loaded dialog and update the state
fn process_loaded_dialog(state: DialogueState, dialog: Dialog) {
    // Build display nodes from the dialog
    let mut display_nodes = build_display_nodes(&dialog);
    let visible_indices: Vec<usize> = (0..display_nodes.len()).collect();

    // Resolve speaker names (embedded DB + runtime loca)
    resolve_speaker_names(&state, &mut display_nodes);

    // Resolve localized text (runtime loca)
    resolve_localized_text(&state, &mut display_nodes);

    // Resolve flag UUIDs to names (flag cache)
    resolve_flag_names(&state, &mut display_nodes);

    // Calculate max content width for horizontal scroll
    let max_width = display_nodes.iter()
        .map(|node| calculate_node_content_width(node))
        .fold(0.0f32, |a, b| a.max(b));

    state.max_content_width.set(max_width);
    state.display_nodes.set(display_nodes);
    state.visible_node_indices.set(visible_indices);
    state.current_dialog.set(Some(Arc::new(dialog)));
    state.selected_node_index.set(None);
    state.selected_node_uuid.set(None);
}
