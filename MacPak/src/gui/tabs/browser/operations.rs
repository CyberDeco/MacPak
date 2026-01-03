//! File operations: loading directories, filtering, selection, image loading

use std::path::Path;
use std::time::UNIX_EPOCH;

use floem::prelude::*;

use crate::gui::state::{BrowserState, EditorTabsState, FileEntry, RawImageData, SortColumn};
use crate::gui::tabs::load_file_in_tab;

/// Format file size for display
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

pub fn select_file(file: &FileEntry, state: BrowserState) {
    state.preview_name.set(file.name.clone());
    // Clear previous image - increment version to ensure UI updates
    let (version, _) = state.preview_image.get();
    state.preview_image.set((version + 1, None));
    // Clear 3D preview path (will be set if .glb/.gltf selected)
    state.preview_3d_path.set(None);

    if file.is_dir {
        state.preview_info.set("Directory".to_string());
        state.preview_content.set("[Double-click to open]".to_string());
        return;
    }

    state.preview_info.set(format!("{} | {}", file.file_type, file.size_formatted));

    let path = Path::new(&file.path);
    let ext = file.extension.to_lowercase();

    match ext.as_str() {
        "lsx" | "lsj" | "xml" | "txt" | "json" | "lua" => {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let preview = if content.len() > 5000 {
                        format!(
                            "{}...\n\n[Truncated - {} bytes total]",
                            &content[..5000],
                            content.len()
                        )
                    } else {
                        content
                    };
                    state.preview_content.set(preview);
                }
                Err(_) => {
                    state.preview_content.set("[Unable to read file]".to_string());
                }
            }
        }
        "lsf" => {
            state.preview_content.set("[Binary LSF file - double-click to open in editor]".to_string());
        }
        "loca" => {
            // Convert to XML for preview
            let temp_path = std::path::Path::new("/tmp/temp_loca.xml");
            match MacLarian::converter::convert_loca_to_xml(path, temp_path) {
                Ok(_) => {
                    match std::fs::read_to_string(temp_path) {
                        Ok(content) => {
                            let preview = if content.len() > 5000 {
                                format!(
                                    "{}...\n\n[Truncated - {} bytes total]",
                                    &content[..5000],
                                    content.len()
                                )
                            } else {
                                content
                            };
                            state.preview_content.set(preview);
                        }
                        Err(_) => {
                            state.preview_content.set("[Unable to read converted file]".to_string());
                        }
                    }
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error converting LOCA: {}]", e));
                }
            }
        }
        "pak" => {
            match MacLarian::pak::PakOperations::list(path) {
                Ok(pak_files) => {
                    let preview = format!(
                        "PAK Archive: {} files\n\n{}",
                        pak_files.len(),
                        pak_files.iter().take(50).cloned().collect::<Vec<_>>().join("\n")
                    );
                    state.preview_content.set(preview);
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error reading PAK: {}]", e));
                }
            }
        }
        "dds" => {
            // Load DDS synchronously (decode to raw RGBA, no PNG encoding)
            let current_version = state.preview_image.get().0;
            match load_dds_image(path) {
                Ok(img_data) => {
                    state.preview_content.set(String::new());
                    state.preview_image.set((current_version + 1, Some(img_data)));
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error loading DDS: {}]", e));
                }
            }
        }
        "png" | "jpg" | "jpeg" => {
            // Load image synchronously (decode to raw RGBA, no PNG encoding)
            let current_version = state.preview_image.get().0;
            match load_standard_image(path) {
                Ok(img_data) => {
                    state.preview_content.set(String::new());
                    state.preview_image.set((current_version + 1, Some(img_data)));
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error loading image: {}]", e));
                }
            }
        }
        "glb" | "gltf" => {
            state.preview_content.set("[3D Model - Click button to preview]".to_string());
            state.preview_3d_path.set(Some(file.path.clone()));
        }
        "gr2" => {
            state.preview_content.set("[GR2 Model - Click button to preview]".to_string());
            state.preview_3d_path.set(Some(file.path.clone()));
        }
        "wem" | "wav" => {
            state.preview_content.set("[Audio file]".to_string());
        }
        _ => {
            state.preview_content.set(format!("File type: {}", ext.to_uppercase()));
        }
    }
}

/// Maximum preview dimension (width or height) for resizing
/// Note: Kept small to avoid filling vger's texture atlas (each 256x256 RGBA = 256KB)
const MAX_PREVIEW_SIZE: u32 = 256;

/// Load a DDS file, resize for preview, and return raw RGBA data
fn load_dds_image(path: &Path) -> Result<RawImageData, String> {
    use dds::{ColorFormat, Decoder, ImageViewMut};

    // Open and decode the DDS file
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut decoder = Decoder::new(file).map_err(|e| e.to_string())?;

    // Get dimensions
    let size = decoder.main_size();

    // Create buffer for RGBA8 output (4 bytes per pixel)
    let mut rgba_buffer = vec![0u8; size.pixels() as usize * 4];

    // Create image view and decode
    let view = ImageViewMut::new(&mut rgba_buffer, size, ColorFormat::RGBA_U8)
        .ok_or_else(|| "Failed to create image view".to_string())?;

    decoder.read_surface(view).map_err(|e| format!("Failed to decode DDS: {:?}", e))?;

    // Create image from raw bytes for resizing
    let img: image::RgbaImage = image::ImageBuffer::from_raw(size.width, size.height, rgba_buffer)
        .ok_or_else(|| "Failed to create image buffer".to_string())?;

    // Resize if larger than preview size
    let img = resize_for_preview(img);

    Ok(RawImageData {
        width: img.width(),
        height: img.height(),
        rgba_data: img.into_raw(),
    })
}

/// Load a regular image file (PNG, JPG), resize for preview, and return raw RGBA data
fn load_standard_image(path: &Path) -> Result<RawImageData, String> {
    // Load the image
    let img = image::open(path).map_err(|e| e.to_string())?;
    let img = img.into_rgba8();

    // Resize if larger than preview size
    let img = resize_for_preview(img);

    Ok(RawImageData {
        width: img.width(),
        height: img.height(),
        rgba_data: img.into_raw(),
    })
}

/// Resize an image to fit within the preview pane while maintaining aspect ratio
fn resize_for_preview(img: image::RgbaImage) -> image::RgbaImage {
    use image::imageops::FilterType;

    let (width, height) = (img.width(), img.height());

    // Only resize if larger than max preview size
    if width <= MAX_PREVIEW_SIZE && height <= MAX_PREVIEW_SIZE {
        return img;
    }

    // Calculate new dimensions maintaining aspect ratio
    let scale = if width > height {
        MAX_PREVIEW_SIZE as f32 / width as f32
    } else {
        MAX_PREVIEW_SIZE as f32 / height as f32
    };

    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    image::imageops::resize(&img, new_width, new_height, FilterType::Triangle)
}

/// Check if a file extension is a text/editable file type
pub fn is_text_file(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "lsf" | "lsx" | "lsj" | "loca" | "khn" | "txt" | "xml" | "json" | "lua" | "md" | "cfg" | "ini" | "yaml" | "yml" | "toml"
    )
}

/// Open file or folder, but only open text files in editor (not images, audio, etc.)
pub fn open_file_or_folder_filtered(
    file: &FileEntry,
    state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: floem::prelude::RwSignal<usize>,
) {
    if file.is_dir {
        load_directory(&file.path, state);
    } else if is_text_file(&file.extension) {
        // Only open text files in Editor tab (opens in new tab or switches to existing)
        let path = Path::new(&file.path);
        load_file_in_tab(path, editor_tabs_state);
        active_tab.set(1);
    }
    // Non-text files: do nothing on double-click (preview is already shown)
}

/// Perform the actual file rename
pub fn perform_rename(old_path: &str, new_name: &str, state: BrowserState) {
    let old_path_obj = Path::new(old_path);
    let parent = old_path_obj.parent().unwrap_or(Path::new("/"));
    let new_path = parent.join(new_name);

    match std::fs::rename(old_path, &new_path) {
        Ok(_) => {
            state.status_message.set("Renamed".to_string());
            refresh(state);
        }
        Err(e) => {
            state.status_message.set(format!("Rename failed: {}", e));
        }
    }
}

/// Delete a file or folder
pub fn delete_file(path: &str, state: BrowserState) {
    let path_obj = Path::new(path);

    let result = if path_obj.is_dir() {
        std::fs::remove_dir_all(path_obj)
    } else {
        std::fs::remove_file(path_obj)
    };

    match result {
        Ok(_) => {
            state.status_message.set("Deleted".to_string());
            // Refresh to update file list
            refresh(state);
        }
        Err(e) => {
            state.status_message.set(format!("Delete failed: {}", e));
        }
    }
}

/// Quick convert a file to another format in the same directory
pub fn convert_file_quick(source_path: &str, target_format: &str, state: BrowserState) {
    let source = Path::new(source_path);
    let source_ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Build destination path (same directory, same name, new extension)
    let dest_path = source.with_extension(target_format);
    let dest = dest_path.to_string_lossy().to_string();

    state.status_message.set(format!("Converting to {}...", target_format.to_uppercase()));

    let result = match (source_ext.as_str(), target_format) {
        ("lsf", "lsx") => MacLarian::converter::lsf_to_lsx(source_path, &dest),
        ("lsx", "lsf") => MacLarian::converter::lsx_to_lsf(source_path, &dest),
        ("lsx", "lsj") => MacLarian::converter::lsx_to_lsj(source_path, &dest),
        ("lsj", "lsx") => MacLarian::converter::lsj_to_lsx(source_path, &dest),
        ("lsf", "lsj") => MacLarian::converter::lsf_to_lsj(source_path, &dest),
        ("lsj", "lsf") => MacLarian::converter::lsj_to_lsf(source_path, &dest),
        ("loca", "xml") => MacLarian::converter::convert_loca_to_xml(source_path, &dest),
        ("xml", "loca") => MacLarian::converter::convert_xml_to_loca(source_path, &dest),
        _ => {
            state.status_message.set(format!(
                "Unsupported conversion: {} to {}",
                source_ext, target_format
            ));
            return;
        }
    };

    match result {
        Ok(_) => {
            state.status_message.set(format!("Converted to {}", target_format.to_uppercase()));
            // Refresh the directory to show new file
            refresh(state);
        }
        Err(e) => {
            state.status_message.set(format!("Conversion failed: {}", e));
        }
    }
}

/// Clean up temporary files created by the browser
pub fn cleanup_temp_files() {
    // Clean up loca preview temp file
    let _ = std::fs::remove_file("/tmp/temp_loca.xml");
}
