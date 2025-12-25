//! Universal Editor tab callbacks

use std::path::Path;
use slint::ComponentHandle;

use crate::MacPakApp;

/// Register all Editor tab callbacks
pub fn register_callbacks(app: &MacPakApp) {
    register_open(app);
    register_save(app);
    register_save_as(app);
    register_convert(app);
}

fn register_open(app: &MacPakApp) {
    app.on_editor_open({
        let app_weak = app.as_weak();
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Open File")
                .add_filter("Larian Files", &["lsx", "lsf", "lsj"])
                .add_filter("LSX (XML)", &["lsx"])
                .add_filter("LSF (Binary)", &["lsf"])
                .add_filter("LSJ (JSON)", &["lsj"])
                .add_filter("All Files", &["*"]);

            if let Some(path) = dialog.pick_file() {
                if let Some(app) = app_weak.upgrade() {
                    let path_str = path.to_string_lossy().to_string();
                    tracing::info!("Opening file: {}", path_str);

                    // Determine format from extension
                    let ext = path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_uppercase();

                    app.set_editor_format(ext.clone().into());
                    app.set_editor_file_path(path_str.clone().into());

                    // Read file content
                    match ext.as_str() {
                        "LSX" | "LSJ" => {
                            // Text formats - read directly
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    app.set_editor_content(content.into());
                                    app.set_editor_modified(false);
                                    app.set_editor_status("File loaded".into());
                                }
                                Err(e) => {
                                    app.set_error_message(format!("Failed to read file: {}", e).into());
                                    app.set_show_error(true);
                                }
                            }
                        }
                        "LSF" => {
                            // Binary format - convert to LSX for display
                            let app_weak2 = app.as_weak();
                            let path_clone = path.clone();
                            std::thread::spawn(move || {
                                // Read LSF and convert to LSX string for display
                                let result = MacLarian::formats::lsf::read_lsf(&path_clone)
                                    .and_then(|lsf_doc| {
                                        // Convert LSF to LSX string
                                        MacLarian::converter::to_lsx(&lsf_doc)
                                    });

                                slint::invoke_from_event_loop(move || {
                                    if let Some(app) = app_weak2.upgrade() {
                                        match result {
                                            Ok(content) => {
                                                app.set_editor_content(content.into());
                                                app.set_editor_modified(false);
                                                app.set_editor_status("LSF loaded (showing as LSX)".into());
                                            }
                                            Err(e) => {
                                                app.set_error_message(format!("Failed to read LSF: {}", e).into());
                                                app.set_show_error(true);
                                            }
                                        }
                                    }
                                }).unwrap();
                            });
                        }
                        _ => {
                            // Unknown format - try to read as text
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    app.set_editor_content(content.into());
                                    app.set_editor_modified(false);
                                }
                                Err(_) => {
                                    app.set_editor_content("[Binary file - cannot display]".into());
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

fn register_save(app: &MacPakApp) {
    app.on_editor_save({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let path = app.get_editor_file_path().to_string();
                let content = app.get_editor_content().to_string();

                if path.is_empty() {
                    app.set_error_message("No file loaded".into());
                    app.set_show_error(true);
                    return;
                }

                tracing::info!("Saving file: {}", path);

                match std::fs::write(&path, &content) {
                    Ok(_) => {
                        app.set_editor_modified(false);
                        app.set_editor_status("Saved".into());
                        tracing::info!("File saved");

                        let app_weak2 = app.as_weak();
                        slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
                            if let Some(app) = app_weak2.upgrade() {
                                app.set_editor_status("".into());
                            }
                        });
                    }
                    Err(e) => {
                        app.set_error_message(format!("Failed to save: {}", e).into());
                        app.set_show_error(true);
                    }
                }
            }
        }
    });
}

fn register_save_as(app: &MacPakApp) {
    app.on_editor_save_as({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let content = app.get_editor_content().to_string();

                let dialog = rfd::FileDialog::new()
                    .set_title("Save As")
                    .add_filter("LSX (XML)", &["lsx"])
                    .add_filter("LSJ (JSON)", &["lsj"])
                    .add_filter("All Files", &["*"]);

                if let Some(path) = dialog.save_file() {
                    let path_str = path.to_string_lossy().to_string();
                    tracing::info!("Saving as: {}", path_str);

                    match std::fs::write(&path, &content) {
                        Ok(_) => {
                            app.set_editor_file_path(path_str.into());
                            app.set_editor_modified(false);
                            app.set_editor_status("Saved".into());

                            // Update format from new extension
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                app.set_editor_format(ext.to_uppercase().into());
                            }
                        }
                        Err(e) => {
                            app.set_error_message(format!("Failed to save: {}", e).into());
                            app.set_show_error(true);
                        }
                    }
                }
            }
        }
    });
}

fn register_convert(app: &MacPakApp) {
    app.on_editor_convert({
        let app_weak = app.as_weak();
        move |conversion_type| {
            if let Some(app) = app_weak.upgrade() {
                let source_path = app.get_editor_file_path().to_string();

                if source_path.is_empty() {
                    app.set_error_message("No file loaded. Open a file first.".into());
                    app.set_show_error(true);
                    return;
                }

                let conversion = conversion_type.to_string();
                tracing::info!("Convert: {} ({})", source_path, conversion);

                app.set_editor_converting(true);

                // Determine output extension based on conversion type
                let (_source_ext, target_ext) = match conversion.as_str() {
                    "lsf-to-lsx" => ("lsf", "lsx"),
                    "lsx-to-lsf" => ("lsx", "lsf"),
                    "lsx-to-lsj" => ("lsx", "lsj"),
                    "lsj-to-lsx" => ("lsj", "lsx"),
                    "lsf-to-lsj" => ("lsf", "lsj"),
                    "lsj-to-lsf" => ("lsj", "lsf"),
                    _ => {
                        app.set_error_message(format!("Unknown conversion: {}", conversion).into());
                        app.set_show_error(true);
                        app.set_editor_converting(false);
                        return;
                    }
                };

                // Show save dialog for converted file
                let dialog = rfd::FileDialog::new()
                    .set_title(&format!("Save Converted File ({})", target_ext.to_uppercase()))
                    .add_filter(&target_ext.to_uppercase(), &[target_ext]);

                if let Some(dest_path) = dialog.save_file() {
                    let source = source_path.clone();
                    let dest = dest_path.to_string_lossy().to_string();
                    let app_weak2 = app_weak.clone();

                    std::thread::spawn(move || {
                        let result = match conversion.as_str() {
                            "lsf-to-lsx" => MacLarian::converter::lsf_to_lsx(&source, &dest),
                            "lsx-to-lsf" => MacLarian::converter::lsx_to_lsf(&source, &dest),
                            "lsx-to-lsj" => MacLarian::converter::lsx_to_lsj(&source, &dest),
                            "lsj-to-lsx" => MacLarian::converter::lsj_to_lsx(&source, &dest),
                            "lsf-to-lsj" => MacLarian::converter::lsf_to_lsj(&source, &dest),
                            "lsj-to-lsf" => MacLarian::converter::lsj_to_lsf(&source, &dest),
                            _ => Err(MacLarian::Error::ConversionError("Unknown conversion".into())),
                        };

                        slint::invoke_from_event_loop(move || {
                            if let Some(app) = app_weak2.upgrade() {
                                match result {
                                    Ok(_) => {
                                        app.set_editor_status(format!("Converted to {}", target_ext.to_uppercase()).into());
                                        tracing::info!("Conversion complete: {}", dest);
                                    }
                                    Err(e) => {
                                        app.set_error_message(format!("Conversion failed: {}", e).into());
                                        app.set_show_error(true);
                                        tracing::error!("Conversion failed: {}", e);
                                    }
                                }
                                app.set_editor_converting(false);
                            }
                        }).unwrap();
                    });
                } else {
                    app.set_editor_converting(false);
                }
            }
        }
    });
}
