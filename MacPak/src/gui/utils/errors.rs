//! Severity-based error handling utilities
//!
//! Provides consistent error handling across the GUI:
//! - Critical errors (file I/O failures) use blocking dialogs
//! - Warnings/info use status messages (handled by each component)

use std::path::Path;

/// Show a critical error dialog for file operations (blocking)
///
/// Use for: file read/write failures, rename/delete errors, data corruption
pub fn show_file_error(path: &Path, operation: &str, error: &str) {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");

    rfd::MessageDialog::new()
        .set_title(&format!("Error {} File", operation))
        .set_description(&format!(
            "Could not {} '{}':\n\n{}",
            operation.to_lowercase(),
            filename,
            error
        ))
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}
