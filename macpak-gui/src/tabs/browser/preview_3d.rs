//! 3D Model preview launcher
//!
//! Spawns the macpak-bevy binary as a subprocess to display .glb files

use std::process::{Child, Command};
use std::sync::{Arc, Mutex, OnceLock};

use floem::prelude::*;

use crate::state::BrowserState;

/// Global handle to the preview process (only one at a time)
static PREVIEW_PROCESS: OnceLock<Arc<Mutex<Option<Child>>>> = OnceLock::new();

fn get_preview_handle() -> &'static Arc<Mutex<Option<Child>>> {
    PREVIEW_PROCESS.get_or_init(|| Arc::new(Mutex::new(None)))
}

/// Launch the 3D preview window for a .glb file
pub fn launch_3d_preview(file_path: &str, state: BrowserState) {
    // Close any existing preview first
    close_3d_preview(state.clone());

    // Find the preview binary
    let preview_binary = find_preview_binary();

    match Command::new(&preview_binary).arg(file_path).spawn() {
        Ok(child) => {
            if let Ok(mut handle) = get_preview_handle().lock() {
                *handle = Some(child);
            }
            state.status_message.set("3D preview opened".to_string());
        }
        Err(e) => {
            state
                .status_message
                .set(format!("Failed to open preview: {}", e));
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
