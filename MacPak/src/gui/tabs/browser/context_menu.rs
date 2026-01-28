//! Context menu for file operations

use std::io::Write as IoWrite;
use std::path::Path;
use std::process::Command;

use floem::action::show_context_menu;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;

use super::operations::{convert_file_quick, delete_file, is_text_file};
use crate::gui::state::{BrowserState, EditorTabsState, FileEntry};
use crate::gui::tabs::load_file_in_tab;

/// Show context menu for a file entry
pub fn show_file_context_menu(
    file: &FileEntry,
    state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) {
    let file_path = file.path.clone();
    let file_ext = file.extension.clone();
    let is_dir = file.is_dir;
    let file_name = file.name.clone();

    let mut menu = Menu::new("");

    // Open in Editor (text files only)
    if !is_dir && is_text_file(&file_ext) {
        let path = file_path.clone();
        let editor_tabs = editor_tabs_state.clone();
        menu = menu.entry(MenuItem::new("Open in Editor").action(move || {
            load_file_in_tab(Path::new(&path), editor_tabs.clone());
            active_tab.set(1);
        }));
        menu = menu.separator();
    }

    // Show in Finder
    {
        let path = file_path.clone();
        menu = menu.entry(MenuItem::new("Show in Finder").action(move || {
            let _ = Command::new("open").arg("-R").arg(&path).spawn();
        }));
    }

    // Copy Path
    {
        let path = file_path.clone();
        menu = menu.entry(MenuItem::new("Copy Path").action(move || {
            if let Ok(mut child) = Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = stdin.write_all(path.as_bytes());
                }
            }
        }));
    }

    menu = menu.separator();

    // Rename (inline)
    {
        let path = file_path.clone();
        let name = file_name.clone();
        let browser_state = state.clone();
        menu = menu.entry(MenuItem::new("Rename").action(move || {
            // Start inline rename
            browser_state.rename_text.set(name.clone());
            browser_state.renaming_path.set(Some(path.clone()));
        }));
    }

    // Convert options (for LSX/LSF/LSJ files only)
    if !is_dir {
        let ext_lower = file_ext.to_lowercase();
        if matches!(ext_lower.as_str(), "lsx" | "lsf" | "lsj") {
            menu = menu.separator();

            // Convert to LSX (if not already LSX)
            if ext_lower != "lsx" {
                let path = file_path.clone();
                let browser_state = state.clone();
                menu = menu.entry(MenuItem::new("Convert to LSX").action(move || {
                    convert_file_quick(&path, "lsx", browser_state.clone());
                }));
            }

            // Convert to LSF (if not already LSF)
            if ext_lower != "lsf" {
                let path = file_path.clone();
                let browser_state = state.clone();
                menu = menu.entry(MenuItem::new("Convert to LSF").action(move || {
                    convert_file_quick(&path, "lsf", browser_state.clone());
                }));
            }

            // Convert to LSJ (if not already LSJ)
            if ext_lower != "lsj" {
                let path = file_path.clone();
                let browser_state = state.clone();
                menu = menu.entry(MenuItem::new("Convert to LSJ").action(move || {
                    convert_file_quick(&path, "lsj", browser_state.clone());
                }));
            }
        }

        // LOCA conversion options
        if ext_lower == "loca" {
            menu = menu.separator();
            let path = file_path.clone();
            let browser_state = state.clone();
            menu = menu.entry(MenuItem::new("Convert to XML").action(move || {
                convert_file_quick(&path, "xml", browser_state.clone());
            }));
        }

        // XML to LOCA option
        if ext_lower == "xml" {
            menu = menu.separator();
            let path = file_path.clone();
            let browser_state = state.clone();
            menu = menu.entry(MenuItem::new("Convert to LOCA").action(move || {
                convert_file_quick(&path, "loca", browser_state.clone());
            }));
        }

        // GR2 conversion options
        if ext_lower == "gr2" {
            menu = menu.separator();
            let path = file_path.clone();
            let browser_state = state.clone();
            menu = menu.entry(MenuItem::new("Convert to...").action(move || {
                // Show GR2 conversion dialog
                browser_state.gr2_convert_path.set(Some(path.clone()));
                browser_state.show_gr2_dialog.set(true);
            }));
        }
    }

    menu = menu.separator();

    // Delete
    {
        let path = file_path.clone();
        let browser_state = state.clone();
        let item_type = if is_dir { "folder" } else { "file" };
        menu = menu.entry(
            MenuItem::new(format!("Delete {}", item_type)).action(move || {
                delete_file(&path, browser_state.clone());
            }),
        );
    }

    show_context_menu(menu, None);
}
