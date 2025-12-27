//! File operations: open, save, load, convert, format, validate

use floem::prelude::*;
use std::fs;
use std::path::Path;

use crate::state::{EditorTab, EditorTabsState};
use super::formatting::{format_json, format_xml};

/// Open file dialog - creates a new tab or uses empty existing tab
pub fn open_file_dialog(tabs_state: EditorTabsState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Open File")
        .add_filter("Larian Files", &["lsx", "lsf", "lsj"])
        .add_filter("LSX (XML)", &["lsx"])
        .add_filter("LSF (Binary)", &["lsf"])
        .add_filter("LSJ (JSON)", &["lsj"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.pick_file() {
        let path_str = path.to_string_lossy().to_string();

        // Check if file is already open
        if tabs_state.switch_to_file(&path_str) {
            return;
        }

        // Check if current tab is empty (new, unmodified, no content)
        let use_current = tabs_state.active_tab().map_or(false, |tab| {
            tab.file_path.get().is_none()
                && tab.content.get().is_empty()
                && !tab.modified.get()
        });

        let tab = if use_current {
            tabs_state.active_tab().unwrap()
        } else {
            tabs_state.new_tab()
        };

        load_file(&path, tab);
    }
}

/// Load a file into a specific tab (used by browser and other components)
pub fn load_file_in_tab(path: &Path, tabs_state: EditorTabsState) {
    let path_str = path.to_string_lossy().to_string();

    // Check if file is already open
    if tabs_state.switch_to_file(&path_str) {
        return;
    }

    // Check if current tab is empty
    let use_current = tabs_state.active_tab().map_or(false, |tab| {
        tab.file_path.get().is_none()
            && tab.content.get().is_empty()
            && !tab.modified.get()
    });

    let tab = if use_current {
        tabs_state.active_tab().unwrap()
    } else {
        tabs_state.new_tab()
    };

    load_file(path, tab);
}

/// Load file contents into the given tab
pub fn load_file(path: &Path, tab: EditorTab) {
    let path_str = path.to_string_lossy().to_string();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_uppercase();

    tab.file_format.set(ext.clone());
    tab.file_path.set(Some(path_str.clone()));

    match ext.as_str() {
        "LSX" => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Skip formatting for large files (>500KB)
                    let formatted = if content.len() > 500_000 {
                        content
                    } else {
                        format_xml(&content)
                    };

                    tab.content.set(formatted);
                    tab.modified.set(false);
                    tab.converted_from_lsf.set(false);
                }
                Err(_e) => {
                    tab.content.set(String::new());
                }
            }
        }
        "LSJ" => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    let formatted = if content.len() > 500_000 {
                        content
                    } else {
                        format_json(&content)
                    };

                    tab.content.set(formatted);
                    tab.modified.set(false);
                    tab.converted_from_lsf.set(false);
                }
                Err(_e) => {
                    tab.content.set(String::new());
                }
            }
        }
        "LSF" => {
            // Binary format - convert to LSX for display using MacLarian
            match MacLarian::formats::lsf::read_lsf(path) {
                Ok(lsf_doc) => match MacLarian::converter::to_lsx(&lsf_doc) {
                    Ok(content) => {
                        let formatted = if content.len() > 500_000 {
                            content
                        } else {
                            format_xml(&content)
                        };

                        tab.content.set(formatted);
                        tab.modified.set(false);
                        tab.converted_from_lsf.set(true);
                    }
                    Err(_e) => {
                        tab.content.set(String::new());
                    }
                },
                Err(_e) => {
                    tab.content.set(String::new());
                }
            }
        }
        _ => {
            // Unknown format - try to read as text
            match fs::read_to_string(path) {
                Ok(content) => {
                    tab.content.set(content);
                    tab.modified.set(false);
                    tab.converted_from_lsf.set(false);
                }
                Err(_) => {
                    tab.content.set("[Binary file - cannot display]".to_string());
                }
            }
        }
    }
}

pub fn save_file(tab: EditorTab) {
    if let Some(path) = tab.file_path.get() {
        let content = tab.content.get();
        match fs::write(&path, &content) {
            Ok(_) => {
                tab.modified.set(false);
            }
            Err(_e) => {
                // Error handling without status_message on tab
            }
        }
    }
}

pub fn save_file_as_dialog(tab: EditorTab) {
    let dialog = rfd::FileDialog::new()
        .set_title("Save As")
        .add_filter("LSX (XML)", &["lsx"])
        .add_filter("LSJ (JSON)", &["lsj"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.save_file() {
        let content = tab.content.get();
        match fs::write(&path, &content) {
            Ok(_) => {
                let path_str = path.to_string_lossy().to_string();
                tab.file_path.set(Some(path_str));
                tab.modified.set(false);
                tab.converted_from_lsf.set(false);

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    tab.file_format.set(ext.to_uppercase());
                }
            }
            Err(_e) => {
                // Error handling
            }
        }
    }
}

pub fn format_content(tab: EditorTab) {
    let content = tab.content.get();
    let format = tab.file_format.get().to_uppercase();

    if content.is_empty() {
        return;
    }

    let formatted = match format.as_str() {
        "LSX" | "LSF" => format_xml(&content),
        "LSJ" => format_json(&content),
        _ => content,
    };

    tab.content.set(formatted);
    tab.modified.set(true);
}

pub fn validate_content(tab: EditorTab, status_message: RwSignal<String>) {
    let content = tab.content.get();
    let format = tab.file_format.get().to_uppercase();

    if content.is_empty() {
        status_message.set("No content to validate".to_string());
        return;
    }

    let result = match format.as_str() {
        "LSX" | "LSF" => match roxmltree::Document::parse(&content) {
            Ok(_) => Ok("Valid XML structure"),
            Err(e) => Err(format!("Invalid XML: {}", e)),
        },
        "LSJ" => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => Ok("Valid JSON structure"),
            Err(e) => Err(format!("Invalid JSON: {}", e)),
        },
        _ => Ok("Unknown format - skipped validation"),
    };

    match result {
        Ok(msg) => status_message.set(msg.to_string()),
        Err(msg) => status_message.set(msg),
    }
}

pub fn convert_file(tab: EditorTab, target_format: &str) {
    let source_path = match tab.file_path.get() {
        Some(p) => p,
        None => {
            return;
        }
    };

    let current_format = tab.file_format.get().to_lowercase();
    let target = target_format.to_lowercase();

    // Show save dialog for converted file
    let dialog = rfd::FileDialog::new()
        .set_title(&format!("Save as {} File", target.to_uppercase()))
        .add_filter(&target.to_uppercase(), &[&target]);

    if let Some(dest_path) = dialog.save_file() {
        let dest = dest_path.to_string_lossy().to_string();

        // Perform conversion
        let _result = match (current_format.as_str(), target.as_str()) {
            ("lsf", "lsx") => MacLarian::converter::lsf_to_lsx(&source_path, &dest),
            ("lsx", "lsf") => MacLarian::converter::lsx_to_lsf(&source_path, &dest),
            ("lsx", "lsj") => MacLarian::converter::lsx_to_lsj(&source_path, &dest),
            ("lsj", "lsx") => MacLarian::converter::lsj_to_lsx(&source_path, &dest),
            ("lsf", "lsj") => MacLarian::converter::lsf_to_lsj(&source_path, &dest),
            ("lsj", "lsf") => MacLarian::converter::lsj_to_lsf(&source_path, &dest),
            _ => {
                return;
            }
        };
    }
}
