//! File open dialog and loading operations

use std::path::Path;

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

use crate::gui::state::{EditorTab, EditorTabsState};

use super::config::track_recent_file;
use super::loading::{handle_phase1_result, load_file_phase1};
use super::types::FileLoadPhase1;

/// Open file dialog - creates a new tab or uses empty existing tab
pub fn open_file_dialog(tabs_state: EditorTabsState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Open File")
        .add_filter("Larian Files", &["lsx", "lsf", "lsj", "lsfx", "loca"])
        .add_filter("LSX (XML)", &["lsx"])
        .add_filter("LSF (Binary)", &["lsf"])
        .add_filter("LSJ (JSON)", &["lsj"])
        .add_filter("LOCA (XML)", &["loca"])
        .add_filter("XML", &["xml"])
        .add_filter("TXT", &["txt", "scene"])
        .add_filter("JSON", &["json"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.pick_file() {
        let path_str = path.to_string_lossy().to_string();

        // Track in recent files
        track_recent_file(&path_str);

        // Check if file is already open
        if tabs_state.switch_to_file(&path_str) {
            return;
        }

        // Check if current tab is empty (new, unmodified, no content)
        // If so, reuse it; otherwise create a new tab
        let tab = match tabs_state.active_tab() {
            Some(active)
                if active.file_path.get().is_none()
                    && active.content.get().is_empty()
                    && !active.modified.get() =>
            {
                active
            }
            _ => tabs_state.new_tab(),
        };

        load_file(&path, tab);
    }
}

/// Open a file by path string (used by recent files menu)
pub fn open_file_at_path(tabs_state: EditorTabsState, path: &str) {
    let path_buf = std::path::PathBuf::from(path);

    // Verify file exists
    if !path_buf.exists() {
        rfd::MessageDialog::new()
            .set_title("File Not Found")
            .set_description(&format!("The file no longer exists:\n{path}"))
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
        return;
    }

    // Track in recent files
    track_recent_file(path);

    // Check if file is already open
    if tabs_state.switch_to_file(path) {
        return;
    }

    // Check if current tab is empty
    let tab = match tabs_state.active_tab() {
        Some(active)
            if active.file_path.get().is_none()
                && active.content.get().is_empty()
                && !active.modified.get() =>
        {
            active
        }
        _ => tabs_state.new_tab(),
    };

    load_file(&path_buf, tab);
}

/// Load a file into a specific tab (used by browser and other components)
pub fn load_file_in_tab(path: &Path, tabs_state: EditorTabsState) {
    let path_str = path.to_string_lossy().to_string();

    // Track in recent files
    track_recent_file(&path_str);

    // Check if file is already open
    if tabs_state.switch_to_file(&path_str) {
        return;
    }

    // Check if current tab is empty (new, unmodified, no content)
    // If so, reuse it; otherwise create a new tab
    let tab = match tabs_state.active_tab() {
        Some(active)
            if active.file_path.get().is_none()
                && active.content.get().is_empty()
                && !active.modified.get() =>
        {
            active
        }
        _ => tabs_state.new_tab(),
    };

    load_file(path, tab);
}

/// Load file contents into the given tab (async with rayon)
pub fn load_file(path: &Path, tab: EditorTab) {
    let path_buf = path.to_path_buf();

    // Don't show loading overlay for phase 1 - it's fast (just node count check for LSF)
    // The overlay will be shown in phase 2 after user confirms large file warning

    // Create callback for phase 1 result
    let send = create_ext_action(Scope::new(), move |result: FileLoadPhase1| {
        handle_phase1_result(tab, result);
    });

    // Spawn background work using rayon - phase 1 is fast (no LSF conversion for large files)
    rayon::spawn(move || {
        let result = load_file_phase1(&path_buf);
        send(result);
    });
}
