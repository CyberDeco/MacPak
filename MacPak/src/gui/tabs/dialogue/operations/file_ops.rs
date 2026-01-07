//! File operations - opening PAKs and folders, scanning for dialogs

use std::path::{Path, PathBuf};
use std::sync::Arc;
use floem::ext_event::create_ext_action;
use floem::reactive::SignalUpdate;
use floem_reactive::Scope;
use MacLarian::formats::dialog::{LocalizationCache, FlagCache};
use MacLarian::pak::PakOperations;
use crate::gui::state::{DialogueState, DialogEntry, DialogSource};

/// Result from background dialog scanning
enum ScanResult {
    Success { entries: Vec<DialogEntry>, count: usize, loca_count: usize },
    Error(String),
}

/// Load a PAK file directly from a path (no file dialog)
/// Used by the Gustav.pak and Shared.pak buttons
pub fn load_pak_directly(state: DialogueState, pak_path: PathBuf) {
    if !pak_path.exists() {
        state.status_message.set(format!("PAK not found: {}", pak_path.display()));
        state.error_message.set(Some(format!("File not found: {}", pak_path.display())));
        return;
    }

    state.status_message.set("Scanning PAK...".to_string());
    state.is_loading.set(true);

    let pak_display = pak_path.display().to_string();
    let loca_cache = state.localization_cache.clone();
    let flag_cache = state.flag_cache.clone();

    // Create ext_action to send results back to main thread
    let send = create_ext_action(Scope::new(), move |result: ScanResult| {
        match result {
            ScanResult::Success { entries, count, loca_count } => {
                if count == 0 {
                    state.status_message.set(format!("No dialogs found in {}", pak_display));
                } else {
                    state.available_dialogs.set(entries);
                    if loca_count > 0 {
                        state.localization_loaded.set(true);
                    }
                    let status = format!("Found {} dialogs, {} loca strings", count, loca_count);
                    state.status_message.set(status);
                }
            }
            ScanResult::Error(e) => {
                state.status_message.set(format!("Error: {}", e));
                state.error_message.set(Some(e));
            }
        }
        state.is_loading.set(false);
    });

    // Spawn thread for loading
    std::thread::spawn(move || {
        // Load localization first (updates Arc<RwLock<>> directly)
        let loca_count = load_localization(&loca_cache, &pak_path);

        // Configure flag cache and build index (one-time cost)
        configure_flags(&flag_cache, &pak_path);

        // Scan for dialog files
        let mut entries = Vec::new();

        match scan_pak_for_dialogs(&pak_path, &mut entries) {
            Ok(_) => {
                let count = entries.len();
                entries.sort_by(|a, b| a.path.cmp(&b.path));
                send(ScanResult::Success { entries, count, loca_count });
            }
            Err(e) => {
                send(ScanResult::Error(e));
            }
        }
    });
}

/// Load localization from the English.pak sibling to the given PAK
fn load_localization(cache: &Arc<std::sync::RwLock<LocalizationCache>>, pak_path: &Path) -> usize {
    let Some(data_dir) = pak_path.parent() else {
        return 0;
    };

    let localization_dir = data_dir.join("Localization");
    if !localization_dir.exists() {
        return 0;
    }

    let language_pak = localization_dir.join("English.pak");
    if !language_pak.exists() {
        return 0;
    }

    let Ok(mut cache) = cache.write() else {
        return 0;
    };

    // Only load if not already loaded
    if !cache.is_empty() {
        return cache.len();
    }

    // List all .loca files in the PAK
    let entries = match PakOperations::list(&language_pak) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let loca_files: Vec<_> = entries
        .iter()
        .filter(|e| e.to_lowercase().ends_with(".loca"))
        .cloned()
        .collect();

    let mut total_count = 0;
    for loca_path in loca_files {
        if let Ok(count) = cache.load_from_pak(&language_pak, &loca_path) {
            total_count += count;
        }
    }

    total_count
}

/// Load localization from a folder that may contain PAK files or a Localization subfolder
fn load_localization_from_folder(cache: &Arc<std::sync::RwLock<LocalizationCache>>, folder: &Path) -> usize {
    // Check for Localization subfolder with English.pak
    let localization_dir = folder.join("Localization");
    if localization_dir.exists() {
        let language_pak = localization_dir.join("English.pak");
        if language_pak.exists() {
            return load_localization_from_pak(cache, &language_pak);
        }
    }

    // Check for Gustav.pak or similar in the folder and load from sibling Localization
    for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "pak").unwrap_or(false) {
            let count = load_localization(cache, &path);
            if count > 0 {
                return count;
            }
        }
    }

    0
}

/// Load localization directly from a language PAK file
fn load_localization_from_pak(cache: &Arc<std::sync::RwLock<LocalizationCache>>, language_pak: &Path) -> usize {
    let Ok(mut cache) = cache.write() else {
        return 0;
    };

    if !cache.is_empty() {
        return cache.len();
    }

    let entries = match PakOperations::list(language_pak) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let loca_files: Vec<_> = entries
        .iter()
        .filter(|e| e.to_lowercase().ends_with(".loca"))
        .cloned()
        .collect();

    let mut total_count = 0;
    for loca_path in loca_files {
        if let Ok(count) = cache.load_from_pak(language_pak, &loca_path) {
            total_count += count;
        }
    }

    total_count
}

/// Configure flag cache and build index from PAK sources
fn configure_flags(cache: &Arc<std::sync::RwLock<FlagCache>>, pak_path: &Path) {
    let Some(data_dir) = pak_path.parent() else {
        return;
    };

    let Ok(mut cache) = cache.write() else {
        return;
    };

    // Skip if already indexed
    if cache.is_indexed() {
        return;
    }

    // Configure PAK sources and build the index
    cache.configure_from_game_data(data_dir);
    if let Err(e) = cache.build_index() {
        tracing::warn!("Failed to build flag index: {}", e);
    }
}

/// Configure flag cache from a folder that may contain PAK files
fn configure_flags_from_folder(cache: &Arc<std::sync::RwLock<FlagCache>>, folder: &Path) {
    let Ok(mut cache) = cache.write() else {
        return;
    };

    // Skip if already indexed
    if cache.is_indexed() {
        return;
    }

    // Look for PAK files in the folder and add them as sources
    for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "pak").unwrap_or(false) {
            cache.add_pak_source(&path);
        }
    }

    // Build the index after adding all sources
    if let Err(e) = cache.build_index() {
        tracing::warn!("Failed to build flag index: {}", e);
    }
}

/// Open a folder containing dialog files
/// Note: File dialog must run on main thread on macOS
pub fn open_dialog_folder(state: DialogueState) {
    // File dialog runs synchronously on main thread (required for macOS)
    let dialog = rfd::FileDialog::new().set_title("Select Dialog Folder");

    if let Some(path) = dialog.pick_folder() {
        state.status_message.set("Loading dialogs...".to_string());
        state.is_loading.set(true);

        let path_display = path.display().to_string();
        let loca_cache = state.localization_cache.clone();
        let flag_cache = state.flag_cache.clone();

        // Create ext_action to send results back to main thread
        let send = create_ext_action(Scope::new(), move |result: ScanResult| {
            match result {
                ScanResult::Success { entries, count, loca_count } => {
                    if count == 0 {
                        state.status_message.set(format!("No dialogs found in {}", path_display));
                    } else {
                        state.available_dialogs.set(entries);
                        if loca_count > 0 {
                            state.localization_loaded.set(true);
                        }
                        let status = format!("Found {} dialogs, {} loca strings", count, loca_count);
                        state.status_message.set(status);
                    }
                }
                ScanResult::Error(e) => {
                    state.status_message.set(format!("Error: {}", e));
                    state.error_message.set(Some(e));
                }
            }
            state.is_loading.set(false);
        });

        // Spawn thread for the actual scanning work
        std::thread::spawn(move || {
            // Try to load localization from PAK files in the folder
            let loca_count = load_localization_from_folder(&loca_cache, &path);

            // Configure flag cache and build index
            configure_flags_from_folder(&flag_cache, &path);

            match scan_dialog_folder(&path) {
                Ok(entries) => {
                    let count = entries.len();
                    send(ScanResult::Success { entries, count, loca_count });
                }
                Err(e) => {
                    send(ScanResult::Error(e));
                }
            }
        });
    }
}

/// Scan a folder for dialog files
/// First checks for PAK files (Gustav.pak, Shared.pak, etc.) and scans inside them
/// Falls back to scanning for extracted .lsj files if no PAKs found
fn scan_dialog_folder(path: &Path) -> Result<Vec<DialogEntry>, String> {
    let mut entries = Vec::new();

    // Look for PAK files in the folder
    let pak_files = find_pak_files(path);

    if !pak_files.is_empty() {
        // Scan inside PAK files for dialogs
        for pak_path in &pak_files {
            let _ = scan_pak_for_dialogs(pak_path, &mut entries);
        }
    }

    // Also scan for extracted dialog files (both in the folder and subdirs)
    let _ = scan_extracted_dialogs(path, path, &mut entries);

    // Sort by path
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(entries)
}

/// Find PAK files in the folder (Gustav.pak, Shared.pak, Patch*.pak)
fn find_pak_files(path: &Path) -> Vec<PathBuf> {
    let mut paks = Vec::new();

    // Check the folder itself
    if let Ok(read_dir) = std::fs::read_dir(path) {
        for entry in read_dir.flatten() {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension() {
                if ext.to_string_lossy().to_lowercase() == "pak" {
                    paks.push(file_path);
                }
            }
        }
    }

    // Also check for a "Data" subfolder (common BG3 structure)
    let data_path = path.join("Data");
    if data_path.is_dir() {
        if let Ok(read_dir) = std::fs::read_dir(&data_path) {
            for entry in read_dir.flatten() {
                let file_path = entry.path();
                if let Some(ext) = file_path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "pak" {
                        paks.push(file_path);
                    }
                }
            }
        }
    }

    // Sort to ensure consistent order (Gustav first, then Shared, then Patches)
    paks.sort_by(|a, b| {
        let a_name = a.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
        let b_name = b.file_name().unwrap_or_default().to_string_lossy().to_lowercase();

        // Priority order: Gustav > Shared > Patch* > others
        let a_priority = if a_name.starts_with("gustav") { 0 }
            else if a_name.starts_with("shared") { 1 }
            else if a_name.starts_with("patch") { 2 }
            else { 3 };
        let b_priority = if b_name.starts_with("gustav") { 0 }
            else if b_name.starts_with("shared") { 1 }
            else if b_name.starts_with("patch") { 2 }
            else { 3 };

        a_priority.cmp(&b_priority).then(a_name.cmp(&b_name))
    });

    paks
}

/// Scan inside a PAK file for dialog files
/// Prefers .lsj files (has editorData with NodeContext/dev notes) over .lsf files
pub(super) fn scan_pak_for_dialogs(pak_path: &Path, entries: &mut Vec<DialogEntry>) -> Result<(), String> {
    let file_list = PakOperations::list(pak_path)
        .map_err(|e| format!("Failed to list PAK: {}", e))?;

    let pak_name = pak_path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "PAK".to_string());

    // Collect dialog files, tracking which base names we've seen
    let mut seen_dialogs = std::collections::HashSet::new();

    // First pass: collect all .lsj dialog files (preferred - has editorData with NodeContext)
    // LSJ files are in Dialogs folder
    for internal_path in &file_list {
        let lower_path = internal_path.to_lowercase();
        if lower_path.contains("story/dialogs/") && lower_path.ends_with(".lsj") {
            let name = internal_path.split('/').last()
                .unwrap_or(internal_path)
                .to_string();

            // Track the base name (without extension) to avoid duplicates
            let base_name = name.trim_end_matches(".lsj").trim_end_matches(".LSJ");
            seen_dialogs.insert(base_name.to_lowercase());

            let display_path = format!("[{}] {}", pak_name, internal_path);

            entries.push(DialogEntry {
                name,
                path: display_path,
                source: DialogSource::PakFile {
                    pak_path: pak_path.to_path_buf(),
                    internal_path: internal_path.clone(),
                },
            });
        }
    }

    // Second pass: add .lsf files only if no .lsj version exists
    // LSF files are in DialogsBinary folder (smaller but lacks editorData)
    for internal_path in &file_list {
        let lower_path = internal_path.to_lowercase();
        if lower_path.contains("story/dialogsbinary/") && lower_path.ends_with(".lsf") {
            let name = internal_path.split('/').last()
                .unwrap_or(internal_path)
                .to_string();

            let base_name = name.trim_end_matches(".lsf").trim_end_matches(".LSF");

            // Skip if we already have the .lsj version
            if seen_dialogs.contains(&base_name.to_lowercase()) {
                continue;
            }

            let display_path = format!("[{}] {}", pak_name, internal_path);

            entries.push(DialogEntry {
                name,
                path: display_path,
                source: DialogSource::PakFile {
                    pak_path: pak_path.to_path_buf(),
                    internal_path: internal_path.clone(),
                },
            });
        }
    }

    Ok(())
}

/// Scan for extracted dialog files (.lsj, .lsf) in a folder
fn scan_extracted_dialogs(dir: &Path, base: &Path, entries: &mut Vec<DialogEntry>) -> Result<(), String> {
    let read_dir = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Skip common non-dialog directories to speed up scanning
            let dir_name = path.file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            // Continue scanning if it might contain dialogs
            if !dir_name.starts_with('.') {
                let _ = scan_extracted_dialogs(&path, base, entries);
            }
        } else if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            // Only look for .lsj files in paths that look like dialog paths
            if ext_lower == "lsj" {
                let path_str = path.to_string_lossy().to_lowercase();
                // Accept if it's in a Dialogs folder OR if it just has any .lsj
                if path_str.contains("dialogs") || path_str.contains("dialog") {
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let relative = path.strip_prefix(base)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string());

                    entries.push(DialogEntry {
                        name,
                        path: relative,
                        source: DialogSource::LocalFile(path.clone()),
                    });
                }
            }
        }
    }

    Ok(())
}
