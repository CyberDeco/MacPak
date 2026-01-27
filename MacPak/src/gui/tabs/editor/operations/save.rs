//! File save operations

use std::fs;
use std::path::Path;

use floem::prelude::*;

use crate::gui::state::EditorTab;
use crate::gui::utils::show_file_error;

pub fn save_file(tab: EditorTab) {
    if let Some(path_str) = tab.file_path.get() {
        let path = Path::new(&path_str);
        // Use live_content which is synced from the editor
        let content = tab.live_content.get();
        let format = tab.file_format.get().to_uppercase();
        let converted_from_binary = tab.converted_from_lsf.get();

        let result = if (format == "LSF" || format == "LSFX") && converted_from_binary {
            // Convert XML back to LSF binary
            match maclarian::converter::from_lsx(&content) {
                Ok(lsf_doc) => {
                    maclarian::formats::lsf::write_lsf(&lsf_doc, path).map_err(|e| e.to_string())
                }
                Err(e) => Err(format!("Failed to parse LSX: {}", e)),
            }
        } else if format == "LOCA" && converted_from_binary {
            // Convert XML back to LOCA binary
            match maclarian::converter::loca_from_xml(&content) {
                Ok(resource) => {
                    maclarian::formats::loca::write_loca(path, &resource).map_err(|e| e.to_string())
                }
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
        if let Some(filename) = Path::new(&current_path)
            .file_name()
            .and_then(|n| n.to_str())
        {
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
                match maclarian::converter::from_lsx(&content) {
                    Ok(lsf_doc) => maclarian::formats::lsf::write_lsf(&lsf_doc, &path)
                        .map_err(|e| e.to_string()),
                    Err(e) => Err(format!("Failed to parse LSX: {}", e)),
                }
            }
            "LOCA" => {
                // Convert XML to LOCA binary
                match maclarian::converter::loca_from_xml(&content) {
                    Ok(resource) => maclarian::formats::loca::write_loca(&path, &resource)
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
                // Mark as converted from binary even if saved to a binary format
                tab.converted_from_lsf
                    .set(matches!(target_ext.as_str(), "LSF" | "LOCA"));
                tab.file_format.set(target_ext);
            }
            Err(e) => {
                show_file_error(&path, "Saving", &e);
            }
        }
    }
}
