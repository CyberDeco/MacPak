//! Asset Browser tab callbacks

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{FileEntry, MacPakApp};

/// Register all Asset Browser tab callbacks
pub fn register_callbacks(
    app: &MacPakApp,
    all_files: Rc<RefCell<Vec<FileEntry>>>,
) {
    register_open_folder(app, all_files.clone());
    register_navigation(app, all_files.clone());
    register_file_selection(app);
    register_filtering(app, all_files.clone());
    register_file_actions(app);
    register_quick_convert(app, all_files.clone());
    register_context_menu(app, all_files);
}

/// Helper function to format file sizes
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return format!("{:.1} {}", size, unit);
        }
        size /= 1024.0;
    }
    format!("{:.1} PB", size)
}

/// Load a directory into the file list
pub fn load_directory(app: &MacPakApp, dir_path: &str, all_files_storage: &Rc<RefCell<Vec<FileEntry>>>) {
    use std::time::UNIX_EPOCH;

    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        return;
    }

    // Update path
    app.set_browser_path(dir_path.into());


    // Load files
    let mut entries: Vec<FileEntry> = Vec::new();
    let mut file_count = 0;
    let mut folder_count = 0;
    let mut total_size: u64 = 0;

    if let Ok(dir_entries) = std::fs::read_dir(path) {
        for entry in dir_entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let name = entry.file_name().to_string_lossy().to_string();
                let full_path = entry.path().to_string_lossy().to_string();
                let is_dir = metadata.is_dir();

                let (file_type, icon) = if is_dir {
                    folder_count += 1;
                    ("Folder".to_string(), "📁".to_string())
                } else {
                    file_count += 1;
                    total_size += metadata.len();
                    let ext = Path::new(&name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_uppercase();

                    let icon = match ext.as_str() {
                        "PAK" => "📦",
                        "LSF" | "LSX" | "LSJ" => "📄",
                        "DDS" | "PNG" | "JPG" | "JPEG" => "🖼️",
                        "GR2" => "🎨",
                        "WEM" | "WAV" => "🔊",
                        "LUA" => "📜",
                        "XML" => "📝",
                        "LOCA" => "🌐",
                        _ => "📄",
                    };
                    (ext, icon.to_string())
                };

                let size = if is_dir {
                    "--".to_string()
                } else {
                    format_size(metadata.len())
                };

                let modified = metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| {
                        let secs = d.as_secs();
                        let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
                            .unwrap_or_default();
                        dt.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_else(|| "--".to_string());

                entries.push(FileEntry {
                    name: name.into(),
                    file_type: file_type.into(),
                    size: size.into(),
                    modified: modified.into(),
                    is_dir,
                    full_path: full_path.into(),
                    icon: icon.into(),
                });
            }
        }
    }

    // Sort: folders first, then files, alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    // Store in shared state for filtering
    *all_files_storage.borrow_mut() = entries.clone();

    // Reset filters when loading new directory
    app.set_browser_search_text("".into());
    app.set_browser_type_filter(0);
    app.set_browser_filter_pak(false);
    app.set_browser_filter_larian(false);
    app.set_browser_filter_images(false);
    app.set_browser_filter_models(false);
    app.set_browser_filter_scripts(false);
    app.set_browser_filter_audio(false);
    app.set_browser_filter_effects(false);

    app.set_browser_files(ModelRc::new(VecModel::from(entries)));
    app.set_browser_file_count(file_count);
    app.set_browser_folder_count(folder_count);
    app.set_browser_visible_count(file_count + folder_count);
    app.set_browser_total_size(format_size(total_size).into());
    app.set_browser_selected(-1);
    app.set_browser_preview_content("".into());
    app.set_browser_preview_name("".into());
    app.set_browser_preview_info("".into());
}

/// Apply filters to the file list
fn apply_filters(app: &MacPakApp, all_files: &[FileEntry]) {
    let search_text = app.get_browser_search_text().to_string().to_lowercase();
    let type_filter = app.get_browser_type_filter();
    let filter_pak = app.get_browser_filter_pak();
    let filter_larian = app.get_browser_filter_larian();
    let filter_images = app.get_browser_filter_images();
    let filter_models = app.get_browser_filter_models();
    let filter_scripts = app.get_browser_filter_scripts();
    let filter_audio = app.get_browser_filter_audio();
    let filter_effects = app.get_browser_filter_effects();

    let has_quick_filter = filter_pak || filter_larian || filter_images || filter_models || filter_scripts || filter_audio || filter_effects;

    let filtered: Vec<FileEntry> = all_files.iter().filter(|file| {
        // Always show directories
        if file.is_dir {
            // But still filter by search text
            if !search_text.is_empty() && !file.name.to_lowercase().contains(&search_text) {
                return false;
            }
            return true;
        }

        // Search text filter
        if !search_text.is_empty() && !file.name.to_lowercase().contains(&search_text) {
            return false;
        }

        let file_type = file.file_type.to_string().to_uppercase();

        // Type dropdown filter
        let passes_type_filter = match type_filter {
            0 => true, // All files
            1 => file_type == "PAK",
            2 => file_type == "LSF",
            3 => file_type == "LSX",
            4 => file_type == "LSJ",
            5 => file_type == "DDS",
            6 => file_type == "GR2",
            7 => matches!(file_type.as_str(), "WEM" | "WAV"),
            8 => file_type == "LOCA",
            _ => true,
        };

        if !passes_type_filter {
            return false;
        }

        // Quick filters (additive - show if matches ANY active filter)
        if has_quick_filter {
            let passes_quick =
                (filter_pak && file_type == "PAK") ||
                (filter_larian && matches!(file_type.as_str(), "LSF" | "LSX" | "LSJ" | "LSB" | "LSBC" | "LSBS")) ||
                (filter_images && matches!(file_type.as_str(), "DDS" | "PNG" | "JPG" | "JPEG")) ||
                (filter_models && file_type == "GR2") ||
                (filter_scripts && matches!(file_type.as_str(), "LUA" | "SCRIPT")) ||
                (filter_audio && matches!(file_type.as_str(), "WEM" | "WAV")) ||
                (filter_effects && file_type == "LSFX");

            if !passes_quick {
                return false;
            }
        }

        true
    }).cloned().collect();

    let visible_count = filtered.len() as i32;
    app.set_browser_files(ModelRc::new(VecModel::from(filtered)));
    app.set_browser_visible_count(visible_count);
    app.set_browser_selected(-1);
}

fn register_open_folder(
    app: &MacPakApp,
    all_files: Rc<RefCell<Vec<FileEntry>>>,
) {
    app.on_browser_open_folder({
        let app_weak = app.as_weak();
        let files = all_files;
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Select Folder to Browse");

            if let Some(path) = dialog.pick_folder() {
                if let Some(app) = app_weak.upgrade() {
                    let path_str = path.to_string_lossy().to_string();
                    tracing::info!("Opening folder: {}", path_str);
                    load_directory(&app, &path_str, &files);
                }
            }
        }
    });
}

fn register_navigation(app: &MacPakApp, all_files: Rc<RefCell<Vec<FileEntry>>>) {
    let files = all_files.clone();
    app.on_browser_go_up({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let current_path = app.get_browser_path().to_string();
                if let Some(parent) = Path::new(&current_path).parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    tracing::info!("Going up to: {}", parent_str);
                    load_directory(&app, &parent_str, &files);
                }
            }
        }
    });

    let files = all_files.clone();
    app.on_browser_go_to_path({
        let app_weak = app.as_weak();
        move |path| {
            if let Some(app) = app_weak.upgrade() {
                let path_str = path.to_string();
                tracing::info!("Navigating to: {}", path_str);
                load_directory(&app, &path_str, &files);
            }
        }
    });

    let files = all_files.clone();
    app.on_browser_refresh({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let path = app.get_browser_path().to_string();
                if !path.is_empty() {
                    tracing::info!("Refreshing browser: {}", path);
                    load_directory(&app, &path, &files);
                }
            }
        }
    });

    app.on_browser_clear_cache({
        move || {
            tracing::info!("Cache cleared");
        }
    });

    let files = all_files;
    app.on_browser_file_double_clicked({
        let app_weak = app.as_weak();
        move |index| {
            if let Some(app) = app_weak.upgrade() {
                let file_list = app.get_browser_files();
                if let Some(file) = file_list.row_data(index as usize) {
                    let full_path = file.full_path.to_string();
                    tracing::info!("Double-clicked: {}", full_path);

                    if file.is_dir {
                        load_directory(&app, &full_path, &files);
                    } else {
                        // Check if it's an editable file type
                        let ext = Path::new(&full_path)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        match ext.as_str() {
                            // Open Larian files in the Universal Editor tab
                            "lsf" | "lsx" | "lsj" | "xml" | "lua" | "txt" | "json" => {
                                tracing::info!("Opening in editor: {}", full_path);
                                // Switch to editor tab and load file
                                app.set_current_tab(1); // Universal Editor tab
                                app.set_editor_file_path(full_path.clone().into());

                                // Try to read the file content
                                if ext == "lsf" {
                                    // For binary LSF, we need to convert it first
                                    match MacLarian::formats::lsf::read_lsf(&full_path) {
                                        Ok(lsf_doc) => {
                                            match MacLarian::converter::to_lsx(&lsf_doc) {
                                                Ok(content) => {
                                                    app.set_editor_content(content.into());
                                                    app.set_editor_format("LSF".into());
                                                    app.set_editor_modified(false);
                                                }
                                                Err(e) => {
                                                    tracing::error!("Failed to convert LSF: {}", e);
                                                    app.set_editor_content(format!("[Error converting LSF: {}]", e).into());
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to read LSF: {}", e);
                                            app.set_editor_content(format!("[Error reading LSF: {}]", e).into());
                                        }
                                    }
                                } else {
                                    // For text files, read directly
                                    match std::fs::read_to_string(&full_path) {
                                        Ok(content) => {
                                            app.set_editor_content(content.into());
                                            app.set_editor_format(ext.to_uppercase().into());
                                            app.set_editor_modified(false);
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to read file: {}", e);
                                            app.set_editor_content(format!("[Error reading file: {}]", e).into());
                                        }
                                    }
                                }
                            }
                            // For other files, try to open with system
                            _ => {
                                #[cfg(target_os = "macos")]
                                {
                                    let _ = std::process::Command::new("open")
                                        .arg(&full_path)
                                        .spawn();
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

fn register_file_selection(app: &MacPakApp) {
    app.on_browser_file_selected({
        let app_weak = app.as_weak();
        move |index| {
            if let Some(app) = app_weak.upgrade() {
                let files = app.get_browser_files();
                if let Some(file) = files.row_data(index as usize) {
                    let full_path = file.full_path.to_string();
                    tracing::info!("Selected: {}", full_path);

                    app.set_browser_preview_name(file.name.clone());

                    let path = Path::new(&full_path);
                    if path.is_file() {
                        let metadata = std::fs::metadata(&path);
                        let size_str = metadata.as_ref().map(|m| format_size(m.len())).unwrap_or_default();
                        app.set_browser_preview_info(format!("{} | {}", file.file_type, size_str).into());

                        let ext = path.extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        match ext.as_str() {
                            "lsx" | "lsj" | "xml" | "txt" | "json" | "lua" => {
                                match std::fs::read_to_string(&path) {
                                    Ok(content) => {
                                        let preview = if content.len() > 5000 {
                                            format!("{}...\n\n[Truncated - {} bytes total]",
                                                &content[..5000], content.len())
                                        } else {
                                            content
                                        };
                                        app.set_browser_preview_content(preview.into());
                                    }
                                    Err(_) => {
                                        app.set_browser_preview_content("[Unable to read file]".into());
                                    }
                                }
                            }
                            "lsf" => {
                                app.set_browser_preview_content("[Binary LSF file - double-click to open in editor]".into());
                            }
                            "pak" => {
                                match MacLarian::pak::PakOperations::list(&path) {
                                    Ok(pak_files) => {
                                        let preview = format!(
                                            "PAK Archive: {} files\n\n{}",
                                            pak_files.len(),
                                            pak_files.iter().take(50).cloned().collect::<Vec<_>>().join("\n")
                                        );
                                        app.set_browser_preview_content(preview.into());
                                    }
                                    Err(e) => {
                                        app.set_browser_preview_content(format!("[Error reading PAK: {}]", e).into());
                                    }
                                }
                            }
                            "dds" | "png" | "jpg" | "jpeg" => {
                                app.set_browser_preview_content("[Image file - preview not available]".into());
                            }
                            "gr2" => {
                                app.set_browser_preview_content("[GR2 Model file]".into());
                            }
                            "wem" | "wav" => {
                                app.set_browser_preview_content("[Audio file]".into());
                            }
                            _ => {
                                app.set_browser_preview_content(format!("File type: {}", ext.to_uppercase()).into());
                            }
                        }
                    } else if path.is_dir() {
                        app.set_browser_preview_info("Directory".into());
                        app.set_browser_preview_content("[Double-click to open]".into());
                    }
                }
            }
        }
    });
}

fn register_filtering(app: &MacPakApp, all_files: Rc<RefCell<Vec<FileEntry>>>) {
    let files = all_files.clone();
    app.on_browser_search_changed({
        let app_weak = app.as_weak();
        move |search_text| {
            tracing::info!("Search changed: {}", search_text);
            if let Some(app) = app_weak.upgrade() {
                let files = files.borrow();
                apply_filters(&app, &files);
            }
        }
    });

    let files = all_files.clone();
    app.on_browser_filter_changed({
        let app_weak = app.as_weak();
        move |filter_index| {
            tracing::info!("Filter changed: {}", filter_index);
            if let Some(app) = app_weak.upgrade() {
                let files = files.borrow();
                apply_filters(&app, &files);
            }
        }
    });

    let files = all_files;
    app.on_browser_toggle_filter({
        let app_weak = app.as_weak();
        move |filter_type| {
            tracing::info!("Toggle filter: {}", filter_type);
            if let Some(app) = app_weak.upgrade() {
                let files = files.borrow();
                apply_filters(&app, &files);
            }
        }
    });
}

fn register_file_actions(app: &MacPakApp) {
    app.on_browser_copy_path({
        let app_weak = app.as_weak();
        move |path| {
            let path_str = path.to_string();
            tracing::info!("Copy path: {}", path_str);

            #[cfg(target_os = "macos")]
            {
                use std::process::Command;
                let _ = Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        use std::io::Write;
                        if let Some(stdin) = child.stdin.as_mut() {
                            stdin.write_all(path_str.as_bytes())?;
                        }
                        child.wait()
                    });
            }

            if let Some(app) = app_weak.upgrade() {
                app.set_copy_status("Copied!".into());
                let app_weak2 = app.as_weak();
                slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
                    if let Some(app) = app_weak2.upgrade() {
                        app.set_copy_status("".into());
                    }
                });
            }
        }
    });

    app.on_browser_show_in_finder({
        move |path| {
            let path_str = path.to_string();
            tracing::info!("Show in Finder: {}", path_str);

            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg("-R")
                    .arg(&path_str)
                    .spawn();
            }
        }
    });

    app.on_browser_open_in_editor({
        move |path| {
            let path_str = path.to_string();
            tracing::info!("Open in editor: {}", path_str);

            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg(&path_str)
                    .spawn();
            }
        }
    });
}

fn register_quick_convert(app: &MacPakApp, _all_files: Rc<RefCell<Vec<FileEntry>>>) {
    // Note: We use invoke_browser_refresh() to refresh the file list after conversion,
    // which uses the handler already registered in register_navigation()
    app.on_browser_quick_convert({
        let app_weak = app.as_weak();
        move |source_path, conversion_type| {
            let source = source_path.to_string();
            let conversion = conversion_type.to_string();
            tracing::info!("Quick convert: {} ({})", source, conversion);

            // Determine output path by changing extension
            let source_path = Path::new(&source);
            let target_ext = match conversion.as_str() {
                "lsf-to-lsx" => "lsx",
                "lsx-to-lsf" => "lsf",
                "lsx-to-lsj" => "lsj",
                "lsj-to-lsx" => "lsx",
                "lsf-to-lsj" => "lsj",
                "lsj-to-lsf" => "lsf",
                _ => return,
            };

            let dest = source_path.with_extension(target_ext);
            let dest_str = dest.to_string_lossy().to_string();

            let app_weak2 = app_weak.clone();
            std::thread::spawn(move || {
                let result = match conversion.as_str() {
                    "lsf-to-lsx" => MacLarian::converter::lsf_to_lsx(&source, &dest_str),
                    "lsx-to-lsf" => MacLarian::converter::lsx_to_lsf(&source, &dest_str),
                    "lsx-to-lsj" => MacLarian::converter::lsx_to_lsj(&source, &dest_str),
                    "lsj-to-lsx" => MacLarian::converter::lsj_to_lsx(&source, &dest_str),
                    "lsf-to-lsj" => MacLarian::converter::lsf_to_lsj(&source, &dest_str),
                    "lsj-to-lsf" => MacLarian::converter::lsj_to_lsf(&source, &dest_str),
                    _ => return,
                };

                slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_weak2.upgrade() {
                        match result {
                            Ok(_) => {
                                tracing::info!("Quick convert complete: {}", dest_str);
                                app.set_copy_status(format!("Converted to {}", target_ext.to_uppercase()).into());

                                // Trigger refresh via the callback to show the new file
                                app.invoke_browser_refresh();

                                let app_weak3 = app.as_weak();
                                slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
                                    if let Some(app) = app_weak3.upgrade() {
                                        app.set_copy_status("".into());
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Quick convert failed: {}", e);
                                app.set_error_message(format!("Conversion failed: {}", e).into());
                                app.set_show_error(true);
                            }
                        }
                    }
                }).unwrap();
            });
        }
    });
}

fn register_context_menu(app: &MacPakApp, all_files: Rc<RefCell<Vec<FileEntry>>>) {
    let files = all_files;
    app.on_browser_context_menu_action({
        let app_weak = app.as_weak();
        move |action, file_path| {
            let action_str = action.to_string();
            let path_str = file_path.to_string();
            tracing::info!("Context menu action: {} on {}", action_str, path_str);

            match action_str.as_str() {
                "delete" => {
                    let path = Path::new(&path_str);
                    let result = if path.is_dir() {
                        std::fs::remove_dir_all(path)
                    } else {
                        std::fs::remove_file(path)
                    };

                    if let Some(app) = app_weak.upgrade() {
                        match result {
                            Ok(_) => {
                                tracing::info!("Deleted: {}", path_str);
                                app.set_copy_status("Deleted!".into());

                                // Refresh the file list
                                let current_path = app.get_browser_path().to_string();
                                if !current_path.is_empty() {
                                    load_directory(&app, &current_path, &files);
                                }

                                let app_weak2 = app.as_weak();
                                slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
                                    if let Some(app) = app_weak2.upgrade() {
                                        app.set_copy_status("".into());
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Failed to delete: {}", e);
                                app.set_error_message(format!("Failed to delete: {}", e).into());
                                app.set_show_error(true);
                            }
                        }
                    }
                }
                _ => {
                    tracing::warn!("Unknown context menu action: {}", action_str);
                }
            }
        }
    });
}
