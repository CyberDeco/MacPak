//! 3D Model preview launcher
//!
//! Spawns the macpak-bevy binary as a subprocess to display .glb/.gr2 files

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex, OnceLock};

use floem::prelude::*;

use crate::gui::state::BrowserState;

/// Global handle to the preview process (only one at a time)
static PREVIEW_PROCESS: OnceLock<Arc<Mutex<Option<Child>>>> = OnceLock::new();

/// Global handle to the temp file path (to clean up when preview closes)
static TEMP_GLB_PATH: OnceLock<Arc<Mutex<Option<std::path::PathBuf>>>> = OnceLock::new();

fn get_preview_handle() -> &'static Arc<Mutex<Option<Child>>> {
    PREVIEW_PROCESS.get_or_init(|| Arc::new(Mutex::new(None)))
}

fn get_temp_path_handle() -> &'static Arc<Mutex<Option<std::path::PathBuf>>> {
    TEMP_GLB_PATH.get_or_init(|| Arc::new(Mutex::new(None)))
}

/// Launch the 3D preview window for a .glb or .gr2 file
pub fn launch_3d_preview(file_path: &str, state: BrowserState) {
    // Close any existing preview first
    close_3d_preview(state.clone());

    let path = Path::new(file_path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // If it's a GR2 file, convert to temp GLB first
    let preview_path = if ext == "gr2" {
        state.status_message.set("Converting GR2 to GLB...".to_string());

        match convert_gr2_to_temp_glb(path) {
            Ok(result) => {
                // Store temp path for cleanup
                if let Ok(mut handle) = get_temp_path_handle().lock() {
                    *handle = Some(result.glb_path.clone());
                }

                // Show any warnings about textures
                if !result.warnings.is_empty() {
                    let warning_msg = result.warnings.join("; ");
                    // Log warnings and show briefly in status
                    tracing::warn!("GR2 preview warnings: {}", warning_msg);
                    // Show first warning in status bar
                    state.status_message.set(format!("Warning: {}", result.warnings[0]));
                }

                result.glb_path.to_string_lossy().to_string()
            }
            Err(e) => {
                state.status_message.set(format!("GR2 conversion failed: {}", e));
                return;
            }
        }
    } else {
        file_path.to_string()
    };

    // Find and launch the preview binary
    let preview_binary = find_preview_binary();
    state.status_message.set("Loading 3D preview...".to_string());

    match Command::new(&preview_binary).arg(&preview_path).spawn() {
        Ok(child) => {
            if let Ok(mut handle) = get_preview_handle().lock() {
                *handle = Some(child);
            }

            // Spawn a background thread to monitor when the preview window closes
            std::thread::spawn(move || {
                // Wait for the process to exit
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    let should_cleanup = if let Ok(mut handle) = get_preview_handle().lock() {
                        if let Some(ref mut child) = *handle {
                            // Check if process has exited
                            match child.try_wait() {
                                Ok(Some(_)) => {
                                    // Process exited, remove from handle
                                    *handle = None;
                                    true
                                }
                                Ok(None) => false, // Still running
                                Err(_) => {
                                    *handle = None;
                                    true
                                }
                            }
                        } else {
                            // No process, stop monitoring
                            break;
                        }
                    } else {
                        break;
                    };

                    if should_cleanup {
                        // Clean up temp file
                        if let Ok(mut temp_handle) = get_temp_path_handle().lock() {
                            if let Some(temp_path) = temp_handle.take() {
                                let _ = std::fs::remove_file(temp_path);
                            }
                        }
                        // Clear loading message
                        state.status_message.set(String::new());
                        break;
                    }
                }
            });
        }
        Err(e) => {
            state.status_message.set(format!("Failed to open preview: {}", e));
        }
    }
}

/// Result of GR2 to GLB conversion
struct Gr2ConversionResult {
    /// Path to the generated temp GLB file
    glb_path: PathBuf,
    /// Warnings to display (e.g., "textures not found")
    warnings: Vec<String>,
}

/// Convert a GR2 file to a temporary GLB file (geometry only for now)
fn convert_gr2_to_temp_glb(gr2_path: &Path) -> Result<Gr2ConversionResult, String> {
    // Create temp file path based on original filename
    let file_stem = gr2_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("preview");

    let temp_dir = std::env::temp_dir();
    let temp_glb = temp_dir.join(format!("{}_preview.glb", file_stem));

    // Use geometry-only conversion (texture matching temporarily disabled)
    MacLarian::converter::convert_gr2_to_glb(gr2_path, &temp_glb)
        .map_err(|e| e.to_string())?;

    Ok(Gr2ConversionResult {
        glb_path: temp_glb,
        warnings: vec![],
    })
}

/// Kill any running preview process (called on app exit)
pub fn kill_preview_process() {
    if let Ok(mut handle) = get_preview_handle().lock() {
        if let Some(mut child) = handle.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    // Clean up temp file
    if let Ok(mut temp_handle) = get_temp_path_handle().lock() {
        if let Some(temp_path) = temp_handle.take() {
            let _ = std::fs::remove_file(temp_path);
        }
    }
}

/// Close the 3D preview window if open
pub fn close_3d_preview(state: BrowserState) {
    if let Ok(mut handle) = get_preview_handle().lock() {
        if let Some(mut child) = handle.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    // Clean up temp file if any
    if let Ok(mut temp_handle) = get_temp_path_handle().lock() {
        if let Some(temp_path) = temp_handle.take() {
            let _ = std::fs::remove_file(temp_path);
        }
    }

    state.status_message.set(String::new());
}

/// Find the preview binary path
fn find_preview_binary() -> String {
    // Check if we're running from cargo (development)
    let exe_path = std::env::current_exe().ok();

    if let Some(exe) = exe_path {
        // Check sibling directory (release/debug build)
        if let Some(parent) = exe.parent() {
            let sibling = parent.join("macpak-bevy");
            if sibling.exists() {
                return sibling.to_string_lossy().to_string();
            }
        }
    }

    // Fallback: assume it's in PATH or current directory
    "macpak-bevy".to_string()
}

/// Create a button to launch 3D preview
pub fn preview_3d_button(state: BrowserState) -> impl IntoView {
    let preview_path = state.preview_3d_path;

    dyn_container(
        move || preview_path.get().is_some(),
        move |has_path| {
            if has_path {
                let state_inner = state.clone();
                let path = state.preview_3d_path.get();

                button("Open 3D Preview")
                    .style(|s| {
                        s.padding_horiz(16.0)
                            .padding_vert(8.0)
                            .background(Color::rgb8(33, 150, 243))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .margin_top(12.0)
                            .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                    })
                    .action(move || {
                        if let Some(ref p) = path {
                            launch_3d_preview(p, state_inner.clone());
                        }
                    })
                    .into_any()
            } else {
                empty().into_any()
            }
        },
    )
}
