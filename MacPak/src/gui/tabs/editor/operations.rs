//! File operations: open, save, load, convert, format, validate

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;
use std::fs;
use std::path::Path;

use crate::gui::state::{EditorTab, EditorTabsState};
use crate::gui::utils::show_file_error;
use super::formatting::{format_json, format_xml};

/// Threshold for large file warning (50,000 lines for text, 5,000 nodes for LSF)
const LARGE_FILE_LINE_THRESHOLD: usize = 50_000;
const LARGE_LSF_NODE_THRESHOLD: usize = 5_000;

/// Result from background file loading - first phase (size check)
enum FileLoadPhase1 {
    /// Text file ready to display (small enough, no confirmation needed)
    Ready(FileLoadResult),
    /// LSF file needs conversion (always show progress)
    LsfNeedsConversion {
        path_str: String,
        format: String,
        node_count: usize,
        lsf_data: Vec<u8>,
        needs_warning: bool,  // True if large file warning should be shown
    },
    /// LOCA file needs conversion (always show progress)
    LocaNeedsConversion {
        path_str: String,
        loca_data: Vec<u8>,
        needs_warning: bool,  // True if result will be large
    },
    /// Large text file needs confirmation then formatting with progress
    TextNeedsConfirmation {
        result: FileLoadResult,
        filename: String,
    },
    /// Error occurred
    Error { path_str: String, error: String },
}

/// Result from background file loading
struct FileLoadResult {
    content: String,
    format: String,
    path_str: String,
    converted_from_binary: bool,
    line_count: usize,
    needs_formatting: bool,
    error: Option<String>,
}

/// Show warning dialog for large text files (line count based).
/// Returns true if user wants to proceed, false if cancelled.
fn show_large_file_warning(filename: &str, line_count: usize) -> bool {
    let result = rfd::MessageDialog::new()
        .set_title("Large File Warning")
        .set_description(&format!(
            "{} has {} lines.\n\n\
            Large files may cause slow scrolling and editing.",
            filename, line_count
        ))
        .set_buttons(rfd::MessageButtons::OkCancelCustom("Open Anyway".to_string(), "Cancel".to_string()))
        .show();

    // OkCancelCustom returns Custom(button_text) for both buttons
    matches!(result, rfd::MessageDialogResult::Custom(ref s) if s == "Open Anyway")
}

/// Show warning dialog for large LSF files (node count based).
/// Returns true if user wants to proceed, false if cancelled.
fn show_large_lsf_warning(filename: &str, node_count: usize) -> bool {
    let result = rfd::MessageDialog::new()
        .set_title("Large File Warning")
        .set_description(&format!(
            "{} has {} nodes.\n\n\
            Converting and displaying large binary files may take a moment.",
            filename, node_count
        ))
        .set_buttons(rfd::MessageButtons::OkCancelCustom("Open Anyway".to_string(), "Cancel".to_string()))
        .show();

    // OkCancelCustom returns Custom(button_text) for both buttons
    matches!(result, rfd::MessageDialogResult::Custom(ref s) if s == "Open Anyway")
}

/// Phase 1: Quick size check (runs in background thread)
/// For binary files (LSF, LOCA), only reads bytes - conversion happens in phase 2 with progress
fn load_file_phase1(path: &Path) -> FileLoadPhase1 {
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
                Ok(data) => {
                    match MacLarian::formats::lsf::parse_lsf_bytes(&data) {
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
                    }
                }
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
        "LSX" => {
            match fs::read_to_string(path) {
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
            }
        }
        "LSJ" => {
            match fs::read_to_string(path) {
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
            }
        }
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
                        "This appears to be a binary file and cannot be displayed as text.".to_string()
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
fn convert_lsf_data_with_progress(
    data: &[u8],
    format: String,
    path_str: String,
    filename: &str,
    tab: EditorTab,
) -> FileLoadResult {
    let send_progress = make_progress_sender(tab);

    // Stage 1: Reading file data
    let data_size = data.len();
    send_progress(format!("Reading {} ({:.1} KB)...", filename, data_size as f64 / 1024.0));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 2: Parse LSF binary header
    send_progress(format!("Parsing {} header...", filename));
    let lsf_doc = match MacLarian::formats::lsf::parse_lsf_bytes(data) {
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
    let content = match MacLarian::converter::to_lsx(&lsf_doc) {
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
    send_progress(format!("Processing output ({:.1} KB)...", content_size as f64 / 1024.0));
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
fn convert_loca_data_with_progress(
    data: &[u8],
    path_str: String,
    filename: &str,
    tab: EditorTab,
) -> FileLoadResult {
    let send_progress = make_progress_sender(tab);

    // Stage 1: Reading file data
    let data_size = data.len();
    send_progress(format!("Reading {} ({:.1} KB)...", filename, data_size as f64 / 1024.0));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stage 2: Parse LOCA binary
    send_progress(format!("Parsing {} localization data...", filename));
    let resource = match MacLarian::formats::loca::parse_loca_bytes(data) {
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
    let content = match MacLarian::converter::loca_to_xml_string(&resource) {
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
    send_progress(format!("Processing output ({:.1} KB)...", content_size as f64 / 1024.0));
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
fn format_text_with_progress(
    mut result: FileLoadResult,
    filename: &str,
    tab: EditorTab,
) -> FileLoadResult {
    let send_progress = make_progress_sender(tab);

    // Stage 1: Report file info
    send_progress(format!("Processing {} ({} lines)...", filename, result.line_count));
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

/// Finalize loading: format and populate the tab
fn finalize_file_load(tab: EditorTab, result: FileLoadResult) {
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
    tab.content.set(content);
    tab.modified.set(false);
    tab.converted_from_lsf.set(result.converted_from_binary);
}

/// Handle phase 1 result on the main thread
fn handle_phase1_result(tab: EditorTab, result: FileLoadPhase1) {
    match result {
        FileLoadPhase1::Ready(file_result) => {
            // Small file, ready to display
            finalize_file_load(tab, file_result);
        }
        FileLoadPhase1::TextNeedsConfirmation { result: file_result, filename } => {
            // Large text file - show warning, then format with progress if confirmed
            if show_large_file_warning(&filename, file_result.line_count) {
                // User confirmed - show loading overlay and format in background
                tab.is_loading.set(true);
                tab.loading_message.set(format!("Processing {}...", filename));

                let tab_for_progress = tab.clone();
                let send = create_ext_action(Scope::new(), move |result: FileLoadResult| {
                    finalize_file_load(tab, result);
                });

                rayon::spawn(move || {
                    let result = format_text_with_progress(file_result, &filename, tab_for_progress);
                    send(result);
                });
            } else {
                tab.is_loading.set(false);
                tab.loading_message.set(String::new());
            }
        }
        FileLoadPhase1::LsfNeedsConversion { path_str, format, node_count, lsf_data, needs_warning } => {
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
        FileLoadPhase1::LocaNeedsConversion { path_str, loca_data, needs_warning } => {
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
    use floem::action::exec_after;
    use std::time::Duration;

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
        let result = match (current_format.as_str(), target.as_str()) {
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

        // Show status badge on success
        if result.is_ok() {
            let save_status = tab.save_status;
            save_status.set(format!("Saved as {}", target.to_uppercase()));

            // Clear status after 3 seconds
            exec_after(Duration::from_secs(3), move |_| {
                save_status.set(String::new());
            });
        }
    }
}
