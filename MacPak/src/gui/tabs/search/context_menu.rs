//! Context menu for search result operations

use floem::action::show_context_menu;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;

use crate::gui::state::{EditorTabsState, SearchResult, SearchState};

use super::operations::copy_to_clipboard;

/// Show context menu for a search result
pub fn show_search_result_context_menu(
    result: &SearchResult,
    state: SearchState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) {
    let result_clone = result.clone();
    let result_for_open = result.clone();
    let result_for_extract = result.clone();
    let result_for_matches = result.clone();
    let state_for_matches = state.clone();

    let mut menu = Menu::new("");

    // Open in Editor (text files only)
    let file_type = result.file_type.to_lowercase();
    if matches!(file_type.as_str(), "lsx" | "lsf" | "lsj" | "xml" | "json" | "txt") {
        let editor_tabs = editor_tabs_state.clone();
        menu = menu.entry(
            MenuItem::new("Open in Editor")
                .action(move || {
                    open_result_in_editor(&result_for_open, editor_tabs.clone(), active_tab);
                })
        );
    }

    // Show All Matches in File
    menu = menu.entry(
        MenuItem::new("Show All Matches")
            .action(move || {
                state_for_matches.all_matches_file.set(Some(result_for_matches.clone()));
                state_for_matches.show_all_matches.set(true);
            })
    );

    menu = menu.separator();

    // Extract File
    menu = menu.entry(
        MenuItem::new("Extract File...")
            .action(move || {
                extract_search_result(&result_for_extract);
            })
    );

    // Copy Path
    {
        let path = result_clone.path.clone();
        menu = menu.entry(
            MenuItem::new("Copy Path")
                .action(move || {
                    copy_to_clipboard(&path);
                })
        );
    }

    show_context_menu(menu, None);
}

/// Open a search result in the Editor tab
fn open_result_in_editor(
    result: &SearchResult,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) {
    use std::env::temp_dir;
    use floem::ext_event::create_ext_action;
    use floem_reactive::Scope;
    use MacLarian::pak::PakOperations;
    use crate::gui::tabs::load_file_in_tab;

    let result = result.clone();
    let pak_path = result.pak_path.clone();
    let file_path = result.path.clone();

    // Run extraction in background thread
    let send = create_ext_action(Scope::new(), move |extracted_path: Result<std::path::PathBuf, String>| {
        match extracted_path {
            Ok(path) => {
                load_file_in_tab(&path, editor_tabs_state.clone());
                active_tab.set(1); // Switch to Editor tab
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Extraction Failed")
                    .set_description(&e)
                    .show();
            }
        }
    });

    std::thread::spawn(move || {
        // Create temp directory for extracted file
        let temp_base = temp_dir().join("macpak_search_preview");
        let pak_name = pak_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let temp_dir = temp_base.join(&pak_name);

        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            send(Err(format!("Failed to create temp directory: {}", e)));
            return;
        }

        // Extract the single file
        match PakOperations::extract_files_with_progress(
            &pak_path,
            &temp_dir,
            &[file_path.as_str()],
            &|_, _, _| {},
        ) {
            Ok(_) => {
                let extracted = temp_dir.join(&file_path);
                send(Ok(extracted));
            }
            Err(e) => {
                send(Err(format!("Extraction failed: {}", e)));
            }
        }
    });
}

/// Extract a search result to a user-selected location
fn extract_search_result(result: &SearchResult) {
    use MacLarian::pak::PakOperations;

    let pak_path = result.pak_path.clone();
    let file_path = result.path.clone();

    // Get destination folder
    let dest = match rfd::FileDialog::new()
        .set_title("Extract File To...")
        .pick_folder()
    {
        Some(d) => d,
        None => return,
    };

    // Extract in background
    std::thread::spawn(move || {
        match PakOperations::extract_files_with_progress(
            &pak_path,
            &dest,
            &[file_path.as_str()],
            &|_, _, _| {},
        ) {
            Ok(_) => {
                rfd::MessageDialog::new()
                    .set_title("Extraction Complete")
                    .set_description(&format!("File extracted to:\n{}", dest.display()))
                    .show();
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Extraction Failed")
                    .set_description(&e.to_string())
                    .show();
            }
        }
    });
}
