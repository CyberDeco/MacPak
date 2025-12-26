//! File operations: open, save, load, convert, format, validate

use floem::prelude::*;
use std::fs;
use std::path::Path;

use crate::state::EditorState;
use super::formatting::{format_json, format_xml};

pub fn open_file_dialog(state: EditorState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Open File")
        .add_filter("Larian Files", &["lsx", "lsf", "lsj"])
        .add_filter("LSX (XML)", &["lsx"])
        .add_filter("LSF (Binary)", &["lsf"])
        .add_filter("LSJ (JSON)", &["lsj"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.pick_file() {
        load_file(&path, state);
    }
}

pub fn load_file(path: &Path, state: EditorState) {
    let path_str = path.to_string_lossy().to_string();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_uppercase();

    state.file_format.set(ext.clone());
    state.file_path.set(Some(path_str.clone()));
    state.status_message.set("Loading...".to_string());

    match ext.as_str() {
        "LSX" => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Skip formatting for large files (>500KB)
                    let (formatted, was_large) = if content.len() > 500_000 {
                        (content, true)
                    } else {
                        (format_xml(&content), false)
                    };

                    state.content.set(formatted);
                    state.modified.set(false);
                    state.converted_from_lsf.set(false);

                    if was_large {
                        state
                            .status_message
                            .set("Large file - formatting skipped".to_string());
                    } else {
                        state.status_message.set("File loaded".to_string());
                    }
                }
                Err(e) => {
                    state.status_message.set(format!("Error: {}", e));
                }
            }
        }
        "LSJ" => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    let (formatted, was_large) = if content.len() > 500_000 {
                        (content, true)
                    } else {
                        (format_json(&content), false)
                    };

                    state.content.set(formatted);
                    state.modified.set(false);
                    state.converted_from_lsf.set(false);

                    if was_large {
                        state
                            .status_message
                            .set("Large file - formatting skipped".to_string());
                    } else {
                        state.status_message.set("File loaded".to_string());
                    }
                }
                Err(e) => {
                    state.status_message.set(format!("Error: {}", e));
                }
            }
        }
        "LSF" => {
            // Binary format - convert to LSX for display using MacLarian
            match MacLarian::formats::lsf::read_lsf(path) {
                Ok(lsf_doc) => match MacLarian::converter::to_lsx(&lsf_doc) {
                    Ok(content) => {
                        let (formatted, was_large) = if content.len() > 500_000 {
                            (content, true)
                        } else {
                            (format_xml(&content), false)
                        };

                        state.content.set(formatted);
                        state.modified.set(false);
                        state.converted_from_lsf.set(true);

                        if was_large {
                            state
                                .status_message
                                .set("Converted from LSF (large file)".to_string());
                        } else {
                            state
                                .status_message
                                .set("Converted from LSF - use Save As".to_string());
                        }
                    }
                    Err(e) => {
                        state
                            .status_message
                            .set(format!("Conversion error: {}", e));
                    }
                },
                Err(e) => {
                    state
                        .status_message
                        .set(format!("Failed to read LSF: {}", e));
                }
            }
        }
        _ => {
            // Unknown format - try to read as text
            match fs::read_to_string(path) {
                Ok(content) => {
                    state.content.set(content);
                    state.modified.set(false);
                    state.converted_from_lsf.set(false);
                    state.status_message.set("File loaded".to_string());
                }
                Err(_) => {
                    state
                        .content
                        .set("[Binary file - cannot display]".to_string());
                    state.status_message.set("Binary file".to_string());
                }
            }
        }
    }
}

pub fn save_file(state: EditorState) {
    if let Some(path) = state.file_path.get() {
        let content = state.content.get();
        match fs::write(&path, &content) {
            Ok(_) => {
                state.modified.set(false);
                state.status_message.set("Saved".to_string());
            }
            Err(e) => {
                state.status_message.set(format!("Save failed: {}", e));
            }
        }
    } else {
        state.status_message.set("No file loaded".to_string());
    }
}

pub fn save_file_as_dialog(state: EditorState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Save As")
        .add_filter("LSX (XML)", &["lsx"])
        .add_filter("LSJ (JSON)", &["lsj"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.save_file() {
        let content = state.content.get();
        match fs::write(&path, &content) {
            Ok(_) => {
                let path_str = path.to_string_lossy().to_string();
                state.file_path.set(Some(path_str));
                state.modified.set(false);
                state.converted_from_lsf.set(false);

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    state.file_format.set(ext.to_uppercase());
                }

                state.status_message.set("Saved".to_string());
            }
            Err(e) => {
                state.status_message.set(format!("Save failed: {}", e));
            }
        }
    }
}

pub fn format_content(state: EditorState) {
    let content = state.content.get();
    let format = state.file_format.get().to_uppercase();

    if content.is_empty() {
        state
            .status_message
            .set("No content to format".to_string());
        return;
    }

    let formatted = match format.as_str() {
        "LSX" | "LSF" => format_xml(&content),
        "LSJ" => format_json(&content),
        _ => content,
    };

    state.content.set(formatted);
    state.modified.set(true);
    state.status_message.set("Content formatted".to_string());
}

pub fn validate_content(state: EditorState) {
    let content = state.content.get();
    let format = state.file_format.get().to_uppercase();

    if content.is_empty() {
        state
            .status_message
            .set("No content to validate".to_string());
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
        Ok(msg) => state.status_message.set(msg.to_string()),
        Err(msg) => state.status_message.set(msg),
    }
}

pub fn convert_file(state: EditorState, target_format: &str) {
    let source_path = match state.file_path.get() {
        Some(p) => p,
        None => {
            state.status_message.set("No file loaded".to_string());
            return;
        }
    };

    let current_format = state.file_format.get().to_lowercase();
    let target = target_format.to_lowercase();

    // Show save dialog for converted file
    let dialog = rfd::FileDialog::new()
        .set_title(&format!("Save as {} File", target.to_uppercase()))
        .add_filter(&target.to_uppercase(), &[&target]);

    if let Some(dest_path) = dialog.save_file() {
        let dest = dest_path.to_string_lossy().to_string();

        state.status_message.set("Converting...".to_string());

        // Perform conversion
        let result = match (current_format.as_str(), target.as_str()) {
            ("lsf", "lsx") => MacLarian::converter::lsf_to_lsx(&source_path, &dest),
            ("lsx", "lsf") => MacLarian::converter::lsx_to_lsf(&source_path, &dest),
            ("lsx", "lsj") => MacLarian::converter::lsx_to_lsj(&source_path, &dest),
            ("lsj", "lsx") => MacLarian::converter::lsj_to_lsx(&source_path, &dest),
            ("lsf", "lsj") => MacLarian::converter::lsf_to_lsj(&source_path, &dest),
            ("lsj", "lsf") => MacLarian::converter::lsj_to_lsf(&source_path, &dest),
            _ => {
                state.status_message.set(format!(
                    "Unsupported conversion: {} to {}",
                    current_format, target
                ));
                return;
            }
        };

        match result {
            Ok(_) => {
                state
                    .status_message
                    .set(format!("Converted to {}", target.to_uppercase()));
            }
            Err(e) => {
                state
                    .status_message
                    .set(format!("Conversion failed: {}", e));
            }
        }
    }
}
