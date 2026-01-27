//! Warning dialogs for large files

/// Show warning dialog for large text files (line count based).
/// Returns true if user wants to proceed, false if cancelled.
pub fn show_large_file_warning(filename: &str, line_count: usize) -> bool {
    let result = rfd::MessageDialog::new()
        .set_title("Large File Warning")
        .set_description(&format!(
            "{} has {} lines.\n\n\
            Large files may cause slow scrolling and editing.",
            filename, line_count
        ))
        .set_buttons(rfd::MessageButtons::OkCancelCustom(
            "Open Anyway".to_string(),
            "Cancel".to_string(),
        ))
        .show();

    // OkCancelCustom returns Custom(button_text) for both buttons
    matches!(result, rfd::MessageDialogResult::Custom(ref s) if s == "Open Anyway")
}

/// Show warning dialog for large LSF files (node count based).
/// Returns true if user wants to proceed, false if cancelled.
pub fn show_large_lsf_warning(filename: &str, node_count: usize) -> bool {
    let result = rfd::MessageDialog::new()
        .set_title("Large File Warning")
        .set_description(&format!(
            "{} has {} nodes.\n\n\
            Converting and displaying large binary files may take a moment.",
            filename, node_count
        ))
        .set_buttons(rfd::MessageButtons::OkCancelCustom(
            "Open Anyway".to_string(),
            "Cancel".to_string(),
        ))
        .show();

    // OkCancelCustom returns Custom(button_text) for both buttons
    matches!(result, rfd::MessageDialogResult::Custom(ref s) if s == "Open Anyway")
}
