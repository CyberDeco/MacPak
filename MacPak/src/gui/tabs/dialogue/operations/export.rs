//! Export operations - HTML and DE2 export

use floem::reactive::{SignalGet, SignalUpdate};
use crate::gui::state::DialogueState;

/// Export dialog to HTML format
pub fn export_html(state: DialogueState) {
    let Some(dialog) = state.current_dialog.get() else {
        return;
    };

    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save HTML Export")
            .add_filter("HTML", &["html"])
            .set_file_name("dialog.html")
            .save_file()
        {
            state.status_message.set("Exporting to HTML...".to_string());

            // Use MacLarian's HTML export
            match MacLarian::formats::dialog::export::generate_html(&dialog) {
                Ok(html) => {
                    match std::fs::write(&path, html) {
                        Ok(_) => {
                            state.status_message.set(format!("Exported to {}", path.display()));
                        }
                        Err(e) => {
                            state.status_message.set(format!("Write error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    state.status_message.set(format!("Export error: {}", e));
                }
            }
        }
    });
}

/// Export dialog to DE2 format
pub fn export_de2(state: DialogueState) {
    let Some(_dialog) = state.current_dialog.get() else {
        return;
    };

    std::thread::spawn(move || {
        if let Some(_path) = rfd::FileDialog::new()
            .set_title("Save DE2 Export")
            .add_filter("LSJ", &["lsj"])
            .set_file_name("dialog_de2.lsj")
            .save_file()
        {
            state.status_message.set("Exporting to DE2...".to_string());

            // TODO: Implement DE2 export
            state.status_message.set("DE2 export not yet implemented".to_string());
        }
    });
}

// HTML export logic is now in MacLarian::formats::dialog::export
