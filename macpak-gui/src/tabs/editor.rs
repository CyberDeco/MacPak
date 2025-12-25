//! Universal Editor tab callbacks

use std::sync::{Arc, Mutex};
use slint::ComponentHandle;
use regex::RegexBuilder;

use crate::MacPakApp;

/// Pretty-print XML content with proper indentation
fn format_xml(content: &str) -> String {
    let mut result = String::new();
    let mut indent_level: i32 = 0;
    let indent_str = "    "; // 4 spaces

    // Simple XML formatter - handles basic cases
    let mut chars = content.chars().peekable();
    let mut in_tag = false;
    let mut current_tag = String::new();
    let mut text_content = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                // Output any accumulated text content
                let trimmed = text_content.trim();
                if !trimmed.is_empty() {
                    result.push_str(trimmed);
                }
                text_content.clear();

                in_tag = true;
                current_tag.clear();
                current_tag.push(ch);
            }
            '>' => {
                current_tag.push(ch);
                in_tag = false;

                let tag = current_tag.trim();

                if tag.starts_with("<?") || tag.starts_with("<!") {
                    // XML declaration or DOCTYPE - no indent change
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push_str(tag);
                    result.push('\n');
                } else if tag.starts_with("</") {
                    // Closing tag - decrease indent first
                    indent_level = (indent_level - 1).max(0);
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                } else if tag.ends_with("/>") {
                    // Self-closing tag - no indent change
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                } else {
                    // Opening tag - indent then increase
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                    indent_level += 1;
                }

                current_tag.clear();
            }
            _ => {
                if in_tag {
                    current_tag.push(ch);
                } else {
                    text_content.push(ch);
                }
            }
        }
    }

    // Add final newline if not present
    if !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

/// Pretty-print JSON content with proper indentation
fn format_json(content: &str) -> String {
    // Try to parse and re-serialize with indentation
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(value) => {
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| content.to_string())
        }
        Err(_) => content.to_string()
    }
}

/// Stores the current search match positions
struct SearchState {
    matches: Vec<(usize, usize)>, // (start, end) positions
    current_index: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            matches: Vec::new(),
            current_index: 0,
        }
    }
}

lazy_static::lazy_static! {
    static ref SEARCH_STATE: Arc<Mutex<SearchState>> = Arc::new(Mutex::new(SearchState::default()));
}

/// Register all Editor tab callbacks
pub fn register_callbacks(app: &MacPakApp) {
    register_open(app);
    register_save(app);
    register_save_as(app);
    register_convert(app);
    register_content_changed(app);
    register_search_callbacks(app);
}

fn register_content_changed(app: &MacPakApp) {
    app.on_editor_content_changed({
        let app_weak = app.as_weak();
        move |content| {
            if let Some(app) = app_weak.upgrade() {
                app.set_editor_content(content);
            }
        }
    });
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
                        "LSX" => {
                            // XML format - read and auto-indent
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    let formatted = format_xml(&content);
                                    app.set_editor_content(formatted.into());
                                    app.set_editor_modified(false);
                                    app.set_editor_status("File loaded".into());
                                }
                                Err(e) => {
                                    app.set_error_message(format!("Failed to read file: {}", e).into());
                                    app.set_show_error(true);
                                }
                            }
                        }
                        "LSJ" => {
                            // JSON format - read and auto-indent
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    let formatted = format_json(&content);
                                    app.set_editor_content(formatted.into());
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
                                                let formatted = format_xml(&content);
                                                app.set_editor_content(formatted.into());
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

fn register_search_callbacks(app: &MacPakApp) {
    register_toggle_search(app);
    register_search_text_changed(app);
    register_find_next(app);
    register_find_previous(app);
    register_replace_current(app);
    register_replace_all(app);
}

fn register_toggle_search(app: &MacPakApp) {
    app.on_editor_toggle_search({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let current = app.get_editor_search_visible();
                app.set_editor_search_visible(!current);

                // Clear search state when hiding
                if current {
                    app.set_editor_search_text("".into());
                    app.set_editor_replace_text("".into());
                    app.set_editor_search_match_count(0);
                    app.set_editor_search_current_match(0);
                    app.set_editor_search_status("".into());

                    if let Ok(mut state) = SEARCH_STATE.lock() {
                        state.matches.clear();
                        state.current_index = 0;
                    }
                }
            }
        }
    });
}

fn register_search_text_changed(app: &MacPakApp) {
    app.on_editor_search_text_changed({
        let app_weak = app.as_weak();
        move |search_text| {
            if let Some(app) = app_weak.upgrade() {
                let search_text = search_text.to_string();

                if search_text.is_empty() {
                    app.set_editor_search_match_count(0);
                    app.set_editor_search_current_match(0);
                    app.set_editor_search_status("".into());

                    if let Ok(mut state) = SEARCH_STATE.lock() {
                        state.matches.clear();
                        state.current_index = 0;
                    }
                    return;
                }

                let content = app.get_editor_content().to_string();
                let case_sensitive = app.get_editor_search_case_sensitive();
                let whole_words = app.get_editor_search_whole_words();
                let use_regex = app.get_editor_search_use_regex();

                // Build the search pattern
                let pattern = if use_regex {
                    search_text.clone()
                } else {
                    // Escape regex special characters for literal search
                    let escaped = regex::escape(&search_text);
                    if whole_words {
                        format!(r"\b{}\b", escaped)
                    } else {
                        escaped
                    }
                };

                // Build regex with options
                match RegexBuilder::new(&pattern)
                    .case_insensitive(!case_sensitive)
                    .build()
                {
                    Ok(re) => {
                        let matches: Vec<(usize, usize)> = re
                            .find_iter(&content)
                            .map(|m| (m.start(), m.end()))
                            .collect();

                        let count = matches.len();

                        if let Ok(mut state) = SEARCH_STATE.lock() {
                            state.matches = matches;
                            // Reset to first match or keep current if still valid
                            if state.current_index >= count {
                                state.current_index = 0;
                            }

                            app.set_editor_search_match_count(count as i32);
                            if count > 0 {
                                app.set_editor_search_current_match((state.current_index + 1) as i32);
                            } else {
                                app.set_editor_search_current_match(0);
                            }
                        }

                        app.set_editor_search_status("".into());
                    }
                    Err(e) => {
                        app.set_editor_search_match_count(0);
                        app.set_editor_search_current_match(0);
                        app.set_editor_search_status(format!("Invalid regex: {}", e).into());

                        if let Ok(mut state) = SEARCH_STATE.lock() {
                            state.matches.clear();
                            state.current_index = 0;
                        }
                    }
                }
            }
        }
    });
}

fn register_find_next(app: &MacPakApp) {
    app.on_editor_find_next({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                if let Ok(mut state) = SEARCH_STATE.lock() {
                    if state.matches.is_empty() {
                        return;
                    }

                    // Move to next match (wrap around)
                    state.current_index = (state.current_index + 1) % state.matches.len();
                    app.set_editor_search_current_match((state.current_index + 1) as i32);

                    // Note: Slint's TextEdit doesn't support programmatic cursor/selection control
                    // So we just update the match counter for now
                    // Future enhancement could use a custom text component with selection support
                }
            }
        }
    });
}

fn register_find_previous(app: &MacPakApp) {
    app.on_editor_find_previous({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                if let Ok(mut state) = SEARCH_STATE.lock() {
                    if state.matches.is_empty() {
                        return;
                    }

                    // Move to previous match (wrap around)
                    if state.current_index == 0 {
                        state.current_index = state.matches.len() - 1;
                    } else {
                        state.current_index -= 1;
                    }
                    app.set_editor_search_current_match((state.current_index + 1) as i32);
                }
            }
        }
    });
}

fn register_replace_current(app: &MacPakApp) {
    app.on_editor_replace_current({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let content = app.get_editor_content().to_string();
                let replace_text = app.get_editor_replace_text().to_string();

                if let Ok(state) = SEARCH_STATE.lock() {
                    if state.matches.is_empty() {
                        return;
                    }

                    let (start, end) = state.matches[state.current_index];

                    // Build new content
                    let mut new_content = String::with_capacity(content.len());
                    new_content.push_str(&content[..start]);
                    new_content.push_str(&replace_text);
                    new_content.push_str(&content[end..]);

                    app.set_editor_content(new_content.into());
                    app.set_editor_modified(true);

                    // Trigger re-search to update matches
                    drop(state); // Release lock before triggering callback
                }

                // Re-trigger search to update match positions
                let search_text = app.get_editor_search_text();
                app.invoke_editor_search_text_changed(search_text);
            }
        }
    });
}

fn register_replace_all(app: &MacPakApp) {
    app.on_editor_replace_all({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let content = app.get_editor_content().to_string();
                let search_text = app.get_editor_search_text().to_string();
                let replace_text = app.get_editor_replace_text().to_string();
                let case_sensitive = app.get_editor_search_case_sensitive();
                let whole_words = app.get_editor_search_whole_words();
                let use_regex = app.get_editor_search_use_regex();

                if search_text.is_empty() {
                    return;
                }

                // Build the search pattern
                let pattern = if use_regex {
                    search_text.clone()
                } else {
                    let escaped = regex::escape(&search_text);
                    if whole_words {
                        format!(r"\b{}\b", escaped)
                    } else {
                        escaped
                    }
                };

                match RegexBuilder::new(&pattern)
                    .case_insensitive(!case_sensitive)
                    .build()
                {
                    Ok(re) => {
                        let new_content = re.replace_all(&content, replace_text.as_str()).to_string();
                        let count = app.get_editor_search_match_count();

                        app.set_editor_content(new_content.into());
                        app.set_editor_modified(true);
                        app.set_editor_search_status(format!("Replaced {} occurrences", count).into());

                        // Clear status after a delay
                        let app_weak2 = app.as_weak();
                        slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
                            if let Some(app) = app_weak2.upgrade() {
                                app.set_editor_search_status("".into());
                            }
                        });

                        // Update match count (should be 0 after replace all)
                        if let Ok(mut state) = SEARCH_STATE.lock() {
                            state.matches.clear();
                            state.current_index = 0;
                        }
                        app.set_editor_search_match_count(0);
                        app.set_editor_search_current_match(0);
                    }
                    Err(e) => {
                        app.set_editor_search_status(format!("Replace failed: {}", e).into());
                    }
                }
            }
        }
    });
}
