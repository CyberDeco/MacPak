//! Directory loading, navigation, filtering, and sorting

use std::path::Path;
use std::time::UNIX_EPOCH;

use floem::prelude::*;

use crate::gui::state::{BrowserState, FileEntry, SortColumn};

use super::utils::format_size;

pub fn open_folder_dialog(state: BrowserState) {
    let dialog = rfd::FileDialog::new().set_title("Select Folder to Browse");

    if let Some(path) = dialog.pick_folder() {
        let path_str = path.to_string_lossy().to_string();
        load_directory(&path_str, state);
    }
}

pub fn go_up(state: BrowserState) {
    if let Some(current) = state.current_path.get() {
        if let Some(parent) = Path::new(&current).parent() {
            let parent_str = parent.to_string_lossy().to_string();
            load_directory(&parent_str, state);
        }
    }
}

pub fn refresh(state: BrowserState) {
    if let Some(path) = state.current_path.get() {
        load_directory(&path, state);
    }
}

pub fn load_directory(dir_path: &str, state: BrowserState) {
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        return;
    }

    state.current_path.set(Some(dir_path.to_string()));
    state.browser_path.set(dir_path.to_string());

    let mut entries: Vec<FileEntry> = Vec::new();
    let mut file_count = 0;
    let mut folder_count = 0;
    let mut total_size: u64 = 0;

    if let Ok(dir_entries) = std::fs::read_dir(path) {
        for entry in dir_entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files
                if name.starts_with('.') {
                    continue;
                }

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
                        "LSF" | "LSX" | "LSJ" | "LSFX" | "LSBC" | "LSBS" => "📖",
                        "DDS" | "PNG" | "JPG" | "JPEG" => "🖼️",
                        "GR2" | "DAE" | "glTF" | "GLB" => "🎨",
                        "WEM" | "WAV" => "🔊",
                        "LUA" | "OSI" | "gameScript" | "itemScript" => "📜",
                        "XML" | "TXT" | "KHN" | "TMPL" => "📝",
                        "LOCA" => "🌐",
                        "SHD" | "BSHD" | "METAL" => "✏️",
                        "DAT" | "DATA" | "PATCH" | "CLC" | "CLM" | "CLN" => "🖥️",
                        "ANC" | "ANM" | "ANN" => "🪄",
                        _ => "📄",
                    };
                    (ext, icon.to_string())
                };

                let size = if is_dir { 0 } else { metadata.len() };
                let size_formatted = if is_dir {
                    "--".to_string()
                } else {
                    format_size(size)
                };

                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| {
                        let secs = d.as_secs();
                        let dt =
                            chrono::DateTime::from_timestamp(secs as i64, 0).unwrap_or_default();
                        dt.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_else(|| "--".to_string());

                let extension = Path::new(&name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string();

                entries.push(FileEntry {
                    name,
                    path: full_path,
                    is_dir,
                    size,
                    size_formatted,
                    extension,
                    file_type,
                    modified,
                    icon,
                });
            }
        }
    }

    // Store all files for filtering
    state.all_files.set(entries.clone());
    state.files.set(entries);

    // Apply current sort settings
    sort_files(state.clone());

    // Reset filters
    state.search_query.set(String::new());
    state.type_filter.set("All".to_string());

    // Update counts
    state.file_count.set(file_count);
    state.folder_count.set(folder_count);
    state.total_size.set(format_size(total_size));

    // Clear selection and preview
    state.selected_index.set(None);
    state.preview_name.set(String::new());
    state.preview_info.set(String::new());
    state.preview_content.set(String::new());
}

pub fn apply_filters(state: BrowserState) {
    let all_files = state.all_files.get();
    let search = state.search_query.get().to_lowercase();
    let type_filter = state.type_filter.get();

    let filtered: Vec<FileEntry> = all_files
        .iter()
        .filter(|file| {
            // Always show directories (but filter by name)
            if file.is_dir {
                if !search.is_empty() && !file.name.to_lowercase().contains(&search) {
                    return false;
                }
                return true;
            }

            // Search text filter
            if !search.is_empty() && !file.name.to_lowercase().contains(&search) {
                return false;
            }

            // Type filter
            if type_filter != "All" && file.file_type != type_filter {
                return false;
            }

            true
        })
        .cloned()
        .collect();

    state.files.set(filtered);
    sort_files(state.clone());
    state.selected_index.set(None);
}

pub fn sort_files(state: BrowserState) {
    let sort_col = state.sort_column.get();
    let ascending = state.sort_ascending.get();
    let mut files = state.files.get();

    files.sort_by(|a, b| {
        // Always put directories first
        match (a.is_dir, b.is_dir) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        let cmp = match sort_col {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Type => a.file_type.cmp(&b.file_type),
            SortColumn::Size => a.size.cmp(&b.size),
            SortColumn::Modified => a.modified.cmp(&b.modified),
        };

        if ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });

    state.files.set(files);
}
