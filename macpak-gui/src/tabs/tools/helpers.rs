//! Helper Functions

use floem::prelude::*;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::state::ToolsState;

pub fn copy_to_clipboard(value: &str) {
    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(value.as_bytes());
            }
            let _ = child.wait();
        }
    }
}

pub fn export_history(state: ToolsState) {
    let uuids = state.uuid_history.get();
    let handles = state.handle_history.get();
    let colors = state.color_history.get();

    let json = serde_json::json!({
        "uuids": uuids,
        "handles": handles.iter().map(|h| format!("h{}", h)).collect::<Vec<_>>(),
        "colors": colors
    });

    let dialog = rfd::FileDialog::new()
        .set_title("Export History")
        .add_filter("JSON", &["json"])
        .set_file_name("macpak_tools_history.json");

    if let Some(path) = dialog.save_file() {
        match fs::write(&path, serde_json::to_string_pretty(&json).unwrap()) {
            Ok(_) => {
                state.status_message.set("Exported successfully!".to_string());
            }
            Err(e) => {
                state.status_message.set(format!("Export failed: {}", e));
            }
        }
    }
}

pub fn clear_all(state: ToolsState) {
    state.uuid_history.set(Vec::new());
    state.handle_history.set(Vec::new());
    state.color_history.set(Vec::new());
    state.generated_uuid.set(String::new());
    state.generated_handle.set(String::new());
    state.status_message.set("All history cleared".to_string());
}
