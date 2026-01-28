//! File format conversion with progress tracking

use std::path::Path;

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

use crate::gui::state::BrowserState;

use super::directory::refresh;

/// Result from background conversion
struct ConversionResult {
    success: bool,
    message: String,
}

/// Quick convert a file to another format in the same directory (async with progress)
pub fn convert_file_quick(source_path: &str, target_format: &str, state: BrowserState) {
    let source = Path::new(source_path);
    let source_ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check if conversion is supported before spawning
    if !matches!(
        (source_ext.as_str(), target_format),
        ("lsf", "lsx")
            | ("lsx", "lsf")
            | ("lsx", "lsj")
            | ("lsj", "lsx")
            | ("lsf", "lsj")
            | ("lsj", "lsf")
            | ("loca", "xml")
            | ("xml", "loca")
    ) {
        state.status_message.set(format!(
            "Unsupported conversion: {} to {}",
            source_ext, target_format
        ));
        return;
    }

    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    // Build destination path (same directory, same name, new extension)
    let dest_path = source.with_extension(target_format);
    let dest = dest_path.to_string_lossy().to_string();

    // Show loading overlay
    state.is_loading.set(true);
    state
        .loading_message
        .set(format!("Reading {}...", filename));

    // Clone values for the background thread
    let source_path = source_path.to_string();
    let target_format = target_format.to_string();
    let source_ext = source_ext.clone();
    let state_for_progress = state.clone();

    // Create callback for completion
    let send_complete = create_ext_action(Scope::new(), move |result: ConversionResult| {
        state.is_loading.set(false);
        state.loading_message.set(String::new());
        state.status_message.set(result.message);
        if result.success {
            refresh(state);
        }
    });

    // Spawn background conversion
    rayon::spawn(move || {
        let result = convert_file_with_progress(
            &source_path,
            &dest,
            &source_ext,
            &target_format,
            &filename,
            state_for_progress,
        );
        send_complete(result);
    });
}

/// Perform conversion with progress updates (runs in background thread)
fn convert_file_with_progress(
    source_path: &str,
    dest_path: &str,
    source_ext: &str,
    target_format: &str,
    filename: &str,
    state: BrowserState,
) -> ConversionResult {
    // Helper to send progress update to UI thread
    let send_progress = |msg: String| {
        let state = state.clone();
        let update = create_ext_action(Scope::new(), move |msg: String| {
            state.loading_message.set(msg);
        });
        update(msg);
    };

    // Stage 1: Reading source file
    send_progress(format!("Reading {}...", filename));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // For LSF conversion progress
    let result = match (source_ext, target_format) {
        ("lsf", "lsx") => {
            // Read LSF file
            send_progress(format!("Reading {} binary data...", filename));
            let data = match std::fs::read(source_path) {
                Ok(d) => d,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to read file: {}", e),
                    };
                }
            };

            // Parse LSF
            send_progress(format!(
                "Parsing {} ({:.1} KB)...",
                filename,
                data.len() as f64 / 1024.0
            ));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let lsf_doc = match maclarian::formats::lsf::parse_lsf_bytes(&data) {
                Ok(doc) => doc,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to parse LSF: {}", e),
                    };
                }
            };

            // Convert to XML
            let node_count = lsf_doc.nodes.len();
            send_progress(format!("Converting {} nodes to XML...", node_count));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let xml_content = match maclarian::converter::to_lsx(&lsf_doc) {
                Ok(xml) => xml,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to convert to XML: {}", e),
                    };
                }
            };

            // Write output
            send_progress(format!(
                "Writing LSX ({:.1} KB)...",
                xml_content.len() as f64 / 1024.0
            ));
            std::thread::sleep(std::time::Duration::from_millis(50));

            std::fs::write(dest_path, xml_content).map_err(|e| e.to_string())
        }
        ("lsx", "lsf") => {
            // Read LSX file
            send_progress(format!("Reading {} XML...", filename));
            let content = match std::fs::read_to_string(source_path) {
                Ok(c) => c,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to read file: {}", e),
                    };
                }
            };

            // Parse XML to LSF document
            send_progress(format!(
                "Parsing XML ({:.1} KB)...",
                content.len() as f64 / 1024.0
            ));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let lsf_doc = match maclarian::converter::from_lsx(&content) {
                Ok(doc) => doc,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to parse LSX: {}", e),
                    };
                }
            };

            // Write binary LSF
            let node_count = lsf_doc.nodes.len();
            send_progress(format!("Writing LSF binary ({} nodes)...", node_count));
            std::thread::sleep(std::time::Duration::from_millis(50));

            maclarian::formats::lsf::write_lsf(&lsf_doc, Path::new(dest_path))
                .map_err(|e| e.to_string())
        }
        ("loca", "xml") => {
            // Read LOCA file
            send_progress(format!("Reading {} binary data...", filename));
            let data = match std::fs::read(source_path) {
                Ok(d) => d,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to read file: {}", e),
                    };
                }
            };

            // Parse LOCA
            send_progress(format!(
                "Parsing {} ({:.1} KB)...",
                filename,
                data.len() as f64 / 1024.0
            ));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let resource = match maclarian::formats::loca::parse_loca_bytes(&data) {
                Ok(res) => res,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to parse LOCA: {}", e),
                    };
                }
            };

            // Convert to XML
            let entry_count = resource.entries.len();
            send_progress(format!("Converting {} entries to XML...", entry_count));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let xml_content = match maclarian::converter::loca_to_xml_string(&resource) {
                Ok(xml) => xml,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to convert to XML: {}", e),
                    };
                }
            };

            // Write output
            send_progress(format!(
                "Writing XML ({:.1} KB)...",
                xml_content.len() as f64 / 1024.0
            ));
            std::thread::sleep(std::time::Duration::from_millis(50));

            std::fs::write(dest_path, xml_content).map_err(|e| e.to_string())
        }
        ("xml", "loca") => {
            // Read XML file
            send_progress(format!("Reading {} XML...", filename));
            let content = match std::fs::read_to_string(source_path) {
                Ok(c) => c,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to read file: {}", e),
                    };
                }
            };

            // Parse XML to LOCA resource
            send_progress(format!(
                "Parsing XML ({:.1} KB)...",
                content.len() as f64 / 1024.0
            ));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let resource = match maclarian::converter::loca_from_xml(&content) {
                Ok(res) => res,
                Err(e) => {
                    return ConversionResult {
                        success: false,
                        message: format!("Failed to parse XML: {}", e),
                    };
                }
            };

            // Write binary LOCA
            let entry_count = resource.entries.len();
            send_progress(format!("Writing LOCA binary ({} entries)...", entry_count));
            std::thread::sleep(std::time::Duration::from_millis(50));

            maclarian::formats::loca::write_loca(Path::new(dest_path), &resource)
                .map_err(|e| e.to_string())
        }
        ("lsx", "lsj") => {
            // Use maclarian's progress-aware conversion
            let progress_cb = |progress: &maclarian::converter::ConvertProgress| {
                send_progress(progress.phase.as_str().to_string());
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            maclarian::converter::lsx_to_lsj_with_progress(source_path, dest_path, &progress_cb)
                .map_err(|e| e.to_string())
        }
        ("lsj", "lsx") => {
            let progress_cb = |progress: &maclarian::converter::ConvertProgress| {
                send_progress(progress.phase.as_str().to_string());
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            maclarian::converter::lsj_to_lsx_with_progress(source_path, dest_path, &progress_cb)
                .map_err(|e| e.to_string())
        }
        ("lsf", "lsj") => {
            let progress_cb = |progress: &maclarian::converter::ConvertProgress| {
                send_progress(progress.phase.as_str().to_string());
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            maclarian::converter::lsf_to_lsj_with_progress(source_path, dest_path, &progress_cb)
                .map_err(|e| e.to_string())
        }
        ("lsj", "lsf") => {
            let progress_cb = |progress: &maclarian::converter::ConvertProgress| {
                send_progress(progress.phase.as_str().to_string());
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            maclarian::converter::lsj_to_lsf_with_progress(source_path, dest_path, &progress_cb)
                .map_err(|e| e.to_string())
        }
        _ => {
            unreachable!(
                "Unsupported conversion: {} -> {}",
                source_ext, target_format
            )
        }
    };

    match result {
        Ok(_) => ConversionResult {
            success: true,
            message: format!("Converted to {}", target_format.to_uppercase()),
        },
        Err(e) => ConversionResult {
            success: false,
            message: format!("Conversion failed: {}", e),
        },
    }
}
