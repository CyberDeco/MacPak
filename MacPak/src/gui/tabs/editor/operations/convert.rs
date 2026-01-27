//! Content validation and file format conversion

use floem::prelude::*;

use crate::gui::state::EditorTab;

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
            ("lsf", "lsx") => maclarian::converter::lsf_to_lsx(&source_path, &dest),
            ("lsx", "lsf") => maclarian::converter::lsx_to_lsf(&source_path, &dest),
            ("lsx", "lsj") => maclarian::converter::lsx_to_lsj(&source_path, &dest),
            ("lsj", "lsx") => maclarian::converter::lsj_to_lsx(&source_path, &dest),
            ("lsf", "lsj") => maclarian::converter::lsf_to_lsj(&source_path, &dest),
            ("lsj", "lsf") => maclarian::converter::lsj_to_lsf(&source_path, &dest),
            ("loca", "xml") => maclarian::converter::convert_loca_to_xml(&source_path, &dest),
            ("xml", "loca") => maclarian::converter::convert_xml_to_loca(&source_path, &dest),
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
