//! File loading phases and conversion with progress

use std::fs;
use std::path::Path;

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

use crate::gui::state::EditorTab;
use crate::gui::utils::show_file_error;

use super::super::formatting::{format_json, format_xml};
use super::dialogs::{show_large_file_warning, show_large_lsf_warning};
use super::types::{
    FileLoadPhase1, FileLoadResult, LARGE_FILE_LINE_THRESHOLD, LARGE_LSF_NODE_THRESHOLD,
};

/// Phase 1: Quick size check (runs in background thread)
/// For binary files (LSF, LOCA), only reads bytes - conversion happens in phase 2 with progress
pub fn load_file_phase1(path: &Path) -> FileLoadPhase1 {
    let path_str = path.to_string_lossy().to_string();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_uppercase();

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    match ext.as_str() {
        "LSF" | "LSFX" => {
            // Read raw bytes and parse LSF to get node count (fast - no XML conversion yet)
            match fs::read(path) {
                Ok(data) => match maclarian::formats::lsf::parse_lsf_bytes(&data) {
                    Ok(lsf_doc) => {
                        let node_count = lsf_doc.nodes.len();
                        // Always return NeedsConversion - progress will be shown for all LSF files
                        FileLoadPhase1::LsfNeedsConversion {
                            path_str,
                            format: ext,
                            node_count,
                            lsf_data: data,
                            needs_warning: node_count > LARGE_LSF_NODE_THRESHOLD,
                        }
                    }
                    Err(e) => FileLoadPhase1::Error {
                        path_str,
                        error: format!("Error parsing {}: {}", ext, e),
                    },
                },
                Err(e) => FileLoadPhase1::Error {
                    path_str,
                    error: e.to_string(),
                },
            }
        }
        "LOCA" => {
            // Read raw bytes - conversion happens in phase 2 with progress
            match fs::read(path) {
                Ok(data) => {
                    // Quick check: estimate if result will be large (rough heuristic: LOCA expands ~10x to XML)
                    let estimated_lines = data.len() / 50; // Very rough estimate
                    FileLoadPhase1::LocaNeedsConversion {
                        path_str,
                        loca_data: data,
                        needs_warning: estimated_lines > LARGE_FILE_LINE_THRESHOLD,
                    }
                }
                Err(e) => FileLoadPhase1::Error {
                    path_str,
                    error: e.to_string(),
                },
            }
        }
        "LSX" => match fs::read_to_string(path) {
            Ok(content) => {
                let line_count = content.lines().count();
                let result = FileLoadResult {
                    content,
                    format: ext,
                    path_str: path_str.clone(),
                    converted_from_binary: false,
                    line_count,
                    needs_formatting: true,
                    error: None,
                };
                if line_count > LARGE_FILE_LINE_THRESHOLD {
                    FileLoadPhase1::TextNeedsConfirmation { result, filename }
                } else {
                    FileLoadPhase1::Ready(result)
                }
            }
            Err(e) => FileLoadPhase1::Error {
                path_str,
                error: e.to_string(),
            },
        },
        "LSJ" => match fs::read_to_string(path) {
            Ok(content) => {
                let line_count = content.lines().count();
                let result = FileLoadResult {
                    content,
                    format: ext,
                    path_str: path_str.clone(),
                    converted_from_binary: false,
                    line_count,
                    needs_formatting: true,
                    error: None,
                };
                if line_count > LARGE_FILE_LINE_THRESHOLD {
                    FileLoadPhase1::TextNeedsConfirmation { result, filename }
                } else {
                    FileLoadPhase1::Ready(result)
                }
            }
            Err(e) => FileLoadPhase1::Error {
                path_str,
                error: e.to_string(),
            },
        },
        _ => {
            // Unknown format - try to read as text
            match fs::read_to_string(path) {
                Ok(content) => {
                    let line_count = content.lines().count();
                    let result = FileLoadResult {
                        content,
                        format: ext,
                        path_str: path_str.clone(),
                        converted_from_binary: false,
                        line_count,
                        needs_formatting: false,
                        error: None,
                    };
                    if line_count > LARGE_FILE_LINE_THRESHOLD {
                        FileLoadPhase1::TextNeedsConfirmation { result, filename }
                    } else {
                        FileLoadPhase1::Ready(result)
                    }
                }
                Err(e) => {
                    let error = if e.kind() == std::io::ErrorKind::InvalidData {
                        "This appears to be a binary file and cannot be displayed as text."
                            .to_string()
                    } else {
                        e.to_string()
                    };
                    FileLoadPhase1::Error { path_str, error }
                }
            }
        }
    }
}

/// Helper to create a progress sender for background threads
fn make_progress_sender(tab: EditorTab) -> impl Fn(String) {
    move |msg: String| {
        let tab = tab.clone();
        let update = create_ext_action(Scope::new(), move |msg: String| {
            tab.loading_message.set(msg);
        });
        update(msg);
    }
}

/// Phase 2: Convert LSF data (runs in background thread)
/// Reports progress at each stage via the tab's loading_message signal
pub fn convert_lsf_data_with_progress(
    data: &[u8],
    format: String,
    path_str: String,
    filename: &str,
    tab: EditorTab,
) -> FileLoadResult {
    let send_progress = make_progress_sender(tab);

    // Stage 1: Reading file data
    let data_size = data.len();
    send_progress(format!(
        "Reading {} ({:.1} KB)...",
        filename,
        data_size as f64 / 1024.0
    ));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 2: Parse LSF binary header
    send_progress(format!("Parsing {} header...", filename));
    let lsf_doc = match maclarian::formats::lsf::parse_lsf_bytes(data) {
        Ok(doc) => doc,
        Err(e) => {
            return FileLoadResult {
                content: String::new(),
                format,
                path_str,
                converted_from_binary: false,
                line_count: 0,
                needs_formatting: false,
                error: Some(format!("Error parsing LSF: {}", e)),
            };
        }
    };

    // Stage 3: Report parsed structure
    let node_count = lsf_doc.nodes.len();
    send_progress(format!("Parsed {} ({} nodes)...", filename, node_count));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 4: Convert to XML
    send_progress(format!("Converting {} to XML...", filename));
    let content = match maclarian::converter::to_lsx(&lsf_doc) {
        Ok(xml) => xml,
        Err(e) => {
            return FileLoadResult {
                content: String::new(),
                format,
                path_str,
                converted_from_binary: false,
                line_count: 0,
                needs_formatting: false,
                error: Some(format!("Error converting to LSX: {}", e)),
            };
        }
    };

    // Stage 5: Processing output
    let content_size = content.len();
    send_progress(format!(
        "Processing output ({:.1} KB)...",
        content_size as f64 / 1024.0
    ));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 6: Count lines
    send_progress("Counting lines...".to_string());
    let line_count = content.lines().count();

    // Stage 7: Preparing editor
    send_progress(format!("Preparing editor ({} lines)...", line_count));
    std::thread::sleep(std::time::Duration::from_millis(50));

    FileLoadResult {
        content,
        format,
        path_str,
        converted_from_binary: true,
        line_count,
        needs_formatting: false,
        error: None,
    }
}

/// Phase 2: Convert LOCA data (runs in background thread)
/// Reports progress at each stage via the tab's loading_message signal
pub fn convert_loca_data_with_progress(
    data: &[u8],
    path_str: String,
    filename: &str,
    tab: EditorTab,
) -> FileLoadResult {
    let send_progress = make_progress_sender(tab);

    // Stage 1: Reading file data
    let data_size = data.len();
    send_progress(format!(
        "Reading {} ({:.1} KB)...",
        filename,
        data_size as f64 / 1024.0
    ));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 2: Parse LOCA binary
    send_progress(format!("Parsing {} localization data...", filename));
    let resource = match maclarian::formats::loca::parse_loca_bytes(data) {
        Ok(res) => res,
        Err(e) => {
            return FileLoadResult {
                content: String::new(),
                format: "LOCA".to_string(),
                path_str,
                converted_from_binary: false,
                line_count: 0,
                needs_formatting: false,
                error: Some(format!("Error parsing LOCA: {}", e)),
            };
        }
    };

    // Stage 3: Report parsed entries
    let entry_count = resource.entries.len();
    send_progress(format!("Parsed {} ({} entries)...", filename, entry_count));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 4: Convert to XML
    send_progress(format!("Converting {} to XML...", filename));
    let content = match maclarian::converter::loca_to_xml_string(&resource) {
        Ok(xml) => xml,
        Err(e) => {
            return FileLoadResult {
                content: String::new(),
                format: "LOCA".to_string(),
                path_str,
                converted_from_binary: false,
                line_count: 0,
                needs_formatting: false,
                error: Some(format!("Error converting LOCA to XML: {}", e)),
            };
        }
    };

    // Stage 5: Processing output
    let content_size = content.len();
    send_progress(format!(
        "Processing output ({:.1} KB)...",
        content_size as f64 / 1024.0
    ));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 6: Count lines
    send_progress("Counting lines...".to_string());
    let line_count = content.lines().count();

    // Stage 7: Preparing editor
    send_progress(format!("Preparing editor ({} lines)...", line_count));
    std::thread::sleep(std::time::Duration::from_millis(50));

    FileLoadResult {
        content,
        format: "LOCA".to_string(),
        path_str,
        converted_from_binary: true,
        line_count,
        needs_formatting: false,
        error: None,
    }
}

/// Phase 2: Format large text file (runs in background thread)
/// Reports progress during formatting
pub fn format_text_with_progress(
    mut result: FileLoadResult,
    filename: &str,
    tab: EditorTab,
) -> FileLoadResult {
    let send_progress = make_progress_sender(tab);

    // Stage 1: Report file info
    send_progress(format!(
        "Processing {} ({} lines)...",
        filename, result.line_count
    ));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 2: Format if needed (skip for very large files >500KB)
    if result.needs_formatting && result.content.len() <= 500_000 {
        send_progress(format!("Formatting {}...", filename));
        std::thread::sleep(std::time::Duration::from_millis(50));

        result.content = match result.format.as_str() {
            "LSX" | "LSF" | "LSFX" | "LOCA" => format_xml(&result.content),
            "LSJ" => format_json(&result.content),
            _ => result.content,
        };
        result.needs_formatting = false;
    }

    // Stage 3: Preparing editor
    send_progress(format!("Preparing editor ({} lines)...", result.line_count));
    std::thread::sleep(std::time::Duration::from_millis(50));

    result
}

/// Finalize loading: format and populate the tab
pub fn finalize_file_load(tab: EditorTab, result: FileLoadResult) {
    tab.is_loading.set(false);
    tab.loading_message.set(String::new());

    if let Some(error) = result.error {
        show_file_error(Path::new(&result.path_str), "Opening", &error);
        return;
    }

    // Format content (skip for very large files >500KB to keep editor responsive)
    let content = if result.needs_formatting && result.content.len() <= 500_000 {
        match result.format.as_str() {
            "LSX" | "LSF" | "LSFX" | "LOCA" => format_xml(&result.content),
            "LSJ" => format_json(&result.content),
            _ => result.content,
        }
    } else {
        result.content
    };

    // Populate the tab
    tab.file_format.set(result.format);
    tab.file_path.set(Some(result.path_str));
    tab.content.set(content.clone());
    tab.live_content.set(content);
    tab.modified.set(false);
    tab.converted_from_lsf.set(result.converted_from_binary);
}

/// Handle phase 1 result on the main thread
pub fn handle_phase1_result(tab: EditorTab, result: FileLoadPhase1) {
    match result {
        FileLoadPhase1::Ready(file_result) => {
            // Small file, ready to display
            finalize_file_load(tab, file_result);
        }
        FileLoadPhase1::TextNeedsConfirmation {
            result: file_result,
            filename,
        } => {
            // Large text file - show warning, then format with progress if confirmed
            if show_large_file_warning(&filename, file_result.line_count) {
                // User confirmed - show loading overlay and format in background
                tab.is_loading.set(true);
                tab.loading_message
                    .set(format!("Processing {}...", filename));

                let tab_for_progress = tab.clone();
                let send = create_ext_action(Scope::new(), move |result: FileLoadResult| {
                    finalize_file_load(tab, result);
                });

                rayon::spawn(move || {
                    let result =
                        format_text_with_progress(file_result, &filename, tab_for_progress);
                    send(result);
                });
            } else {
                tab.is_loading.set(false);
                tab.loading_message.set(String::new());
            }
        }
        FileLoadPhase1::LsfNeedsConversion {
            path_str,
            format,
            node_count,
            lsf_data,
            needs_warning,
        } => {
            let filename = Path::new(&path_str)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();

            // Show warning for large files, otherwise proceed directly
            let should_proceed = if needs_warning {
                show_large_lsf_warning(&filename, node_count)
            } else {
                true
            };

            if should_proceed {
                // Show loading overlay and do conversion with progress
                tab.is_loading.set(true);
                tab.loading_message.set(format!("Reading {}...", filename));

                let tab_for_progress = tab.clone();
                let send = create_ext_action(Scope::new(), move |result: FileLoadResult| {
                    finalize_file_load(tab, result);
                });

                rayon::spawn(move || {
                    let result = convert_lsf_data_with_progress(
                        &lsf_data,
                        format,
                        path_str,
                        &filename,
                        tab_for_progress,
                    );
                    send(result);
                });
            } else {
                tab.is_loading.set(false);
                tab.loading_message.set(String::new());
            }
        }
        FileLoadPhase1::LocaNeedsConversion {
            path_str,
            loca_data,
            needs_warning,
        } => {
            let filename = Path::new(&path_str)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();

            // Show warning for large files (estimated), otherwise proceed directly
            let should_proceed = if needs_warning {
                // Use text warning since result will be text
                let estimated_lines = loca_data.len() / 50;
                show_large_file_warning(&filename, estimated_lines)
            } else {
                true
            };

            if should_proceed {
                // Show loading overlay and do conversion with progress
                tab.is_loading.set(true);
                tab.loading_message.set(format!("Reading {}...", filename));

                let tab_for_progress = tab.clone();
                let send = create_ext_action(Scope::new(), move |result: FileLoadResult| {
                    finalize_file_load(tab, result);
                });

                rayon::spawn(move || {
                    let result = convert_loca_data_with_progress(
                        &loca_data,
                        path_str,
                        &filename,
                        tab_for_progress,
                    );
                    send(result);
                });
            } else {
                tab.is_loading.set(false);
                tab.loading_message.set(String::new());
            }
        }
        FileLoadPhase1::Error { path_str, error } => {
            tab.is_loading.set(false);
            tab.loading_message.set(String::new());
            show_file_error(Path::new(&path_str), "Opening", &error);
        }
    }
}
