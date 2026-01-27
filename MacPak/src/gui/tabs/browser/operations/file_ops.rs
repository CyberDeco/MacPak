//! File operations: rename, delete, open

use std::path::Path;

use floem::prelude::*;

use crate::gui::state::{BrowserState, EditorTabsState, FileEntry};
use crate::gui::tabs::load_file_in_tab;
use crate::gui::utils::show_file_error;

use super::directory::{load_directory, refresh};
use super::utils::is_text_file;

/// Open file or folder, but only open text files in editor (not images, audio, etc.)
pub fn open_file_or_folder_filtered(
    file: &FileEntry,
    state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: floem::prelude::RwSignal<usize>,
) {
    if file.is_dir {
        load_directory(&file.path, state);
    } else if is_text_file(&file.extension) {
        // Only open text files in Editor tab (opens in new tab or switches to existing)
        let path = Path::new(&file.path);
        load_file_in_tab(path, editor_tabs_state);
        active_tab.set(1);
    }
    // Non-text files: do nothing on double-click (preview is already shown)
}

/// Perform the actual file rename
pub fn perform_rename(old_path: &str, new_name: &str, state: BrowserState) {
    let old_path_obj = Path::new(old_path);
    let parent = old_path_obj.parent().unwrap_or(Path::new("/"));
    let new_path = parent.join(new_name);

    match std::fs::rename(old_path, &new_path) {
        Ok(_) => {
            state.status_message.set("Renamed".to_string());
            refresh(state);
        }
        Err(e) => {
            show_file_error(old_path_obj, "Renaming", &e.to_string());
        }
    }
}

/// Delete a file or folder
pub fn delete_file(path: &str, state: BrowserState) {
    let path_obj = Path::new(path);

    let result = if path_obj.is_dir() {
        std::fs::remove_dir_all(path_obj)
    } else {
        std::fs::remove_file(path_obj)
    };

    match result {
        Ok(_) => {
            state.status_message.set("Deleted".to_string());
            // Refresh to update file list
            refresh(state);
        }
        Err(e) => {
            show_file_error(path_obj, "Deleting", &e.to_string());
        }
    }
}
