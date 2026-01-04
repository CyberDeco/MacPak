//! File operations: open, save, load, convert, format, validate

use floem::prelude::*;
use std::fs;
use std::path::Path;

use crate::gui::state::{EditorTab, EditorTabsState};
use crate::gui::utils::show_file_error;
use super::formatting::{format_json, format_xml};

/// Threshold for large file warning (50,000 lines)
const LARGE_FILE_LINE_THRESHOLD: usize = 50_000;

/// Check line count and show warning dialog for large files
/// Returns: Some(true) = proceed, Some(false) = convert, None = cancel
fn check_large_file(content: &str, path: &Path) -> Option<bool> {
    let line_count = content.lines().count();
    if line_count <= LARGE_FILE_LINE_THRESHOLD {
        return Some(true); // Proceed normally
    }

    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let result = rfd::MessageDialog::new()
        .set_title("Large File Warning")
        .set_description(&format!(
            "{} has {} lines.\n\n\
            Large files may cause slow scrolling and editing.",
            filename, line_count
        ))
        .set_buttons(rfd::MessageButtons::OkCancelCustom("Open Anyway".to_string(), "Cancel".to_string()))
        .show();

    match result {
        rfd::MessageDialogResult::Ok => Some(true),
        rfd::MessageDialogResult::Cancel => None,
        _ => None,
    }
}

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

        // Check if file is already open
        if tabs_state.switch_to_file(&path_str) {
            return;
        }

        // Check if current tab is empty (new, unmodified, no content)
        // If so, reuse it; otherwise create a new tab
        let tab = match tabs_state.active_tab() {
            Some(active) if active.file_path.get().is_none()
                && active.content.get().is_empty()
                && !active.modified.get() => active,
            _ => tabs_state.new_tab(),
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

    // Check if current tab is empty (new, unmodified, no content)
    // If so, reuse it; otherwise create a new tab
    let tab = match tabs_state.active_tab() {
        Some(active) if active.file_path.get().is_none()
            && active.content.get().is_empty()
            && !active.modified.get() => active,
        _ => tabs_state.new_tab(),
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

    // Helper to finalize tab state after successful load
    let finalize_tab = |content: String, converted_from_lsf: bool| {
        tab.file_format.set(ext.clone());
        tab.file_path.set(Some(path_str.clone()));
        tab.content.set(content);
        tab.modified.set(false);
        tab.converted_from_lsf.set(converted_from_lsf);
    };

    match ext.as_str() {
        "LSX" => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Check for large file and prompt user
                    if check_large_file(&content, path).is_none() {
                        return; // User cancelled
                    }

                    // Skip formatting for large files (>500KB)
                    let formatted = if content.len() > 500_000 {
                        content
                    } else {
                        format_xml(&content)
                    };

                    finalize_tab(formatted, false);
                }
                Err(e) => {
                    show_file_error(path, "Opening", &e.to_string());
                }
            }
        }
        "LSJ" => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Check for large file and prompt user
                    if check_large_file(&content, path).is_none() {
                        return; // User cancelled
                    }

                    let formatted = if content.len() > 500_000 {
                        content
                    } else {
                        format_json(&content)
                    };

                    finalize_tab(formatted, false);
                }
                Err(e) => {
                    show_file_error(path, "Opening", &e.to_string());
                }
            }
        }
        "LSF" | "LSFX" => {
            // Binary format - convert to LSX for display using MacLarian
            match MacLarian::formats::lsf::read_lsf(path) {
                Ok(lsf_doc) => match MacLarian::converter::to_lsx(&lsf_doc) {
                    Ok(content) => {
                        // Check for large file and prompt user
                        if check_large_file(&content, path).is_none() {
                            return; // User cancelled
                        }

                        let formatted = if content.len() > 500_000 {
                            content
                        } else {
                            format_xml(&content)
                        };

                        finalize_tab(formatted, true);
                    }
                    Err(e) => {
                        finalize_tab(format!("<!-- Error converting {} to LSX: {} -->", ext, e), false);
                    }
                },
                Err(e) => {
                    finalize_tab(format!("<!-- Error reading {}: {} -->", ext, e), false);
                }
            }
        }
        "LOCA" => {
            // Binary LOCA format - convert to XML for display
            match MacLarian::formats::loca::read_loca(path) {
                Ok(resource) => match MacLarian::converter::loca_to_xml_string(&resource) {
                    Ok(content) => {
                        // Check for large file and prompt user
                        if check_large_file(&content, path).is_none() {
                            return; // User cancelled
                        }

                        let formatted = if content.len() > 500_000 {
                            content
                        } else {
                            format_xml(&content)
                        };

                        // Set format to LOCA (not XML) so we know it came from a .loca file
                        tab.file_format.set("LOCA".to_string());
                        tab.file_path.set(Some(path_str.clone()));
                        tab.content.set(formatted);
                        tab.modified.set(false);
                        tab.converted_from_lsf.set(true); // Reuse flag for "converted from binary"
                    }
                    Err(e) => {
                        finalize_tab(format!("<!-- Error converting {} to XML: {} -->", ext, e), false);
                    }
                },
                Err(e) => {
                    finalize_tab(format!("<!-- Error reading {}: {} -->", ext, e), false);
                }
            }
        }
        _ => {
            // Unknown format - try to read as text
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Check for large file and prompt user
                    if check_large_file(&content, path).is_none() {
                        return; // User cancelled
                    }

                    finalize_tab(content, false);
                }
                Err(e) => {
                    // Check if it's a binary file (contains invalid UTF-8)
                    if e.kind() == std::io::ErrorKind::InvalidData {
                        show_file_error(path, "Opening", "This appears to be a binary file and cannot be displayed as text.");
                    } else {
                        show_file_error(path, "Opening", &e.to_string());
                    }
                }
            }
        }
    }
}

pub fn save_file(tab: EditorTab) {
    if let Some(path_str) = tab.file_path.get() {
        let path = Path::new(&path_str);
        let content = tab.content.get();
        let format = tab.file_format.get().to_uppercase();
        let converted_from_binary = tab.converted_from_lsf.get();

        let result = if (format == "LSF" || format == "LSFX") && converted_from_binary {
            // Convert XML back to LSF binary
            match MacLarian::converter::from_lsx(&content) {
                Ok(lsf_doc) => MacLarian::formats::lsf::write_lsf(&lsf_doc, path)
                    .map_err(|e| e.to_string()),
                Err(e) => Err(format!("Failed to parse LSX: {}", e)),
            }
        } else if format == "LOCA" && converted_from_binary {
            // Convert XML back to LOCA binary
            match MacLarian::converter::loca_from_xml(&content) {
                Ok(resource) => MacLarian::formats::loca::write_loca(path, &resource)
                    .map_err(|e| e.to_string()),
                Err(e) => Err(format!("Failed to parse LOCA XML: {}", e)),
            }
        } else {
            // Write as plain text
            fs::write(path, &content).map_err(|e| e.to_string())
        };

        match result {
            Ok(_) => {
                tab.modified.set(false);
            }
            Err(e) => {
                show_file_error(path, "Saving", &e);
            }
        }
    }
}

pub fn save_file_as_dialog(tab: EditorTab) {
    let current_format = tab.file_format.get().to_uppercase();

    // Build dialog with filters based on current format
    let mut dialog = rfd::FileDialog::new().set_title("Save As");

    // Add appropriate filters based on current format type
    match current_format.as_str() {
        "LOCA" | "XML" => {
            // LOCA-related formats
            dialog = dialog
                .add_filter("XML", &["xml"])
                .add_filter("LOCA (Binary)", &["loca"]);
        }
        _ => {
            // LSF-related formats (LSX, LSJ, LSF, or unknown)
            dialog = dialog
                .add_filter("LSX (XML)", &["lsx"])
                .add_filter("LSJ (JSON)", &["lsj"])
                .add_filter("LSF (Binary)", &["lsf"]);
        }
    }

    dialog = dialog.add_filter("All Files", &["*"]);

    // Set default filename from current path if available
    if let Some(current_path) = tab.file_path.get() {
        if let Some(filename) = Path::new(&current_path).file_name().and_then(|n| n.to_str()) {
            dialog = dialog.set_file_name(filename);
        }
        if let Some(parent) = Path::new(&current_path).parent() {
            dialog = dialog.set_directory(parent);
        }
    }

    if let Some(path) = dialog.save_file() {
        let content = tab.content.get();
        let target_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_uppercase();

        // Perform conversion if saving to binary format
        let result: Result<(), String> = match target_ext.as_str() {
            "LSF" => {
                // Convert XML/LSX to LSF binary
                match MacLarian::converter::from_lsx(&content) {
                    Ok(lsf_doc) => MacLarian::formats::lsf::write_lsf(&lsf_doc, &path)
                        .map_err(|e| e.to_string()),
                    Err(e) => Err(format!("Failed to parse LSX: {}", e)),
                }
            }
            "LOCA" => {
                // Convert XML to LOCA binary
                match MacLarian::converter::loca_from_xml(&content) {
                    Ok(resource) => MacLarian::formats::loca::write_loca(&path, &resource)
                        .map_err(|e| e.to_string()),
                    Err(e) => Err(format!("Failed to parse LOCA XML: {}", e)),
                }
            }
            _ => {
                // Write as plain text
                fs::write(&path, &content).map_err(|e| e.to_string())
            }
        };

        match result {
            Ok(_) => {
                let path_str = path.to_string_lossy().to_string();
                tab.file_path.set(Some(path_str));
                tab.modified.set(false);
                // Mark as converted from binary if we saved to a binary format
                tab.converted_from_lsf.set(matches!(target_ext.as_str(), "LSF" | "LOCA"));
                tab.file_format.set(target_ext);
            }
            Err(e) => {
                show_file_error(&path, "Saving", &e);
            }
        }
    }
}


pub fn validate_content(tab: EditorTab, status_message: RwSignal<String>) {
    let content = tab.content.get();
    let format = tab.file_format.get().to_uppercase();

    if content.is_empty() {
        status_message.set("No content to validate".to_string());
        return;
    }

    let result = match format.as_str() {
        "LSX" | "LSF" | "LSFX" | "LOCA" => match roxmltree::Document::parse(&content) {
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
            ("loca", "xml") => MacLarian::converter::convert_loca_to_xml(&source_path, &dest),
            ("xml", "loca") => MacLarian::converter::convert_xml_to_loca(&source_path, &dest),
            _ => {
                return;
            }
        };
    }
}
