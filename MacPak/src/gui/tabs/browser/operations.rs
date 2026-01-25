//! File operations: loading directories, filtering, selection, image loading

use std::path::Path;
use std::time::UNIX_EPOCH;

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

use crate::gui::state::{BrowserState, EditorTabsState, FileEntry, RawImageData, SortColumn};
use crate::gui::tabs::load_file_in_tab;
use crate::gui::utils::show_file_error;

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
            match maclarian::converter::convert_loca_to_xml(path, temp_path) {
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
            match maclarian::pak::PakOperations::list(path) {
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
            show_file_error(old_path_obj, "Renaming", &e.to_string());
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
            show_file_error(path_obj, "Deleting", &e.to_string());
        }
    }
}

/// Result from background conversion
struct ConversionResult {
    success: bool,
    message: String,
}

/// Quick convert a file to another format in the same directory (async with progress)
pub fn convert_file_quick(source_path: &str, target_format: &str, state: BrowserState) {
    let source = Path::new(source_path);
    let source_ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check if conversion is supported before spawning
    if !matches!(
        (source_ext.as_str(), target_format),
        ("lsf", "lsx") | ("lsx", "lsf") | ("lsx", "lsj") | ("lsj", "lsx") |
        ("lsf", "lsj") | ("lsj", "lsf") | ("loca", "xml") | ("xml", "loca")
    ) {
        state.status_message.set(format!(
            "Unsupported conversion: {} to {}",
            source_ext, target_format
        ));
        return;
    }

    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    // Build destination path (same directory, same name, new extension)
    let dest_path = source.with_extension(target_format);
    let dest = dest_path.to_string_lossy().to_string();

    // Show loading overlay
    state.is_loading.set(true);
    state.loading_message.set(format!("Reading {}...", filename));

    // Clone values for the background thread
    let source_path = source_path.to_string();
    let target_format = target_format.to_string();
    let source_ext = source_ext.clone();
    let state_for_progress = state.clone();

    // Create callback for completion
    let send_complete = create_ext_action(Scope::new(), move |result: ConversionResult| {
        state.is_loading.set(false);
        state.loading_message.set(String::new());
        state.status_message.set(result.message);
        if result.success {
            refresh(state);
        }
    });

    // Spawn background conversion
    rayon::spawn(move || {
        let result = convert_file_with_progress(
            &source_path,
            &dest,
            &source_ext,
            &target_format,
            &filename,
            state_for_progress,
        );
        send_complete(result);
    });
}

/// Perform conversion with progress updates (runs in background thread)
fn convert_file_with_progress(
    source_path: &str,
    dest_path: &str,
    source_ext: &str,
    target_format: &str,
    filename: &str,
    state: BrowserState,
) -> ConversionResult {
    // Helper to send progress update to UI thread
    let send_progress = |msg: String| {
        let state = state.clone();
        let update = create_ext_action(Scope::new(), move |msg: String| {
            state.loading_message.set(msg);
        });
        update(msg);
    };

    // Stage 1: Reading source file
    send_progress(format!("Reading {}...", filename));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // For LSF conversions, we can provide more granular progress
    let result = match (source_ext, target_format) {
        ("lsf", "lsx") => {
            // Read LSF file
            send_progress(format!("Reading {} binary data...", filename));
            let data = match std::fs::read(source_path) {
                Ok(d) => d,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                },
            };

            // Parse LSF
            send_progress(format!("Parsing {} ({:.1} KB)...", filename, data.len() as f64 / 1024.0));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let lsf_doc = match maclarian::formats::lsf::parse_lsf_bytes(&data) {
                Ok(doc) => doc,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to parse LSF: {}", e),
                },
            };

            // Convert to XML
            let node_count = lsf_doc.nodes.len();
            send_progress(format!("Converting {} nodes to XML...", node_count));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let xml_content = match maclarian::converter::to_lsx(&lsf_doc) {
                Ok(xml) => xml,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to convert to XML: {}", e),
                },
            };

            // Write output
            send_progress(format!("Writing LSX ({:.1} KB)...", xml_content.len() as f64 / 1024.0));
            std::thread::sleep(std::time::Duration::from_millis(50));

            std::fs::write(dest_path, xml_content).map_err(|e| e.to_string())
        }
        ("lsx", "lsf") => {
            // Read LSX file
            send_progress(format!("Reading {} XML...", filename));
            let content = match std::fs::read_to_string(source_path) {
                Ok(c) => c,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                },
            };

            // Parse XML to LSF document
            send_progress(format!("Parsing XML ({:.1} KB)...", content.len() as f64 / 1024.0));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let lsf_doc = match maclarian::converter::from_lsx(&content) {
                Ok(doc) => doc,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to parse LSX: {}", e),
                },
            };

            // Write binary LSF
            let node_count = lsf_doc.nodes.len();
            send_progress(format!("Writing LSF binary ({} nodes)...", node_count));
            std::thread::sleep(std::time::Duration::from_millis(50));

            maclarian::formats::lsf::write_lsf(&lsf_doc, Path::new(dest_path))
                .map_err(|e| e.to_string())
        }
        ("loca", "xml") => {
            // Read LOCA file
            send_progress(format!("Reading {} binary data...", filename));
            let data = match std::fs::read(source_path) {
                Ok(d) => d,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                },
            };

            // Parse LOCA
            send_progress(format!("Parsing {} ({:.1} KB)...", filename, data.len() as f64 / 1024.0));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let resource = match maclarian::formats::loca::parse_loca_bytes(&data) {
                Ok(res) => res,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to parse LOCA: {}", e),
                },
            };

            // Convert to XML
            let entry_count = resource.entries.len();
            send_progress(format!("Converting {} entries to XML...", entry_count));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let xml_content = match maclarian::converter::loca_to_xml_string(&resource) {
                Ok(xml) => xml,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to convert to XML: {}", e),
                },
            };

            // Write output
            send_progress(format!("Writing XML ({:.1} KB)...", xml_content.len() as f64 / 1024.0));
            std::thread::sleep(std::time::Duration::from_millis(50));

            std::fs::write(dest_path, xml_content).map_err(|e| e.to_string())
        }
        ("xml", "loca") => {
            // Read XML file
            send_progress(format!("Reading {} XML...", filename));
            let content = match std::fs::read_to_string(source_path) {
                Ok(c) => c,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to read file: {}", e),
                },
            };

            // Parse XML to LOCA resource
            send_progress(format!("Parsing XML ({:.1} KB)...", content.len() as f64 / 1024.0));
            std::thread::sleep(std::time::Duration::from_millis(50));

            let resource = match maclarian::converter::loca_from_xml(&content) {
                Ok(res) => res,
                Err(e) => return ConversionResult {
                    success: false,
                    message: format!("Failed to parse XML: {}", e),
                },
            };

            // Write binary LOCA
            let entry_count = resource.entries.len();
            send_progress(format!("Writing LOCA binary ({} entries)...", entry_count));
            std::thread::sleep(std::time::Duration::from_millis(50));

            maclarian::formats::loca::write_loca(Path::new(dest_path), &resource)
                .map_err(|e| e.to_string())
        }
        ("lsx", "lsj") => {
            // Use maclarian's progress-aware conversion
            let progress_cb = |msg: &str| {
                send_progress(msg.to_string());
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            maclarian::converter::lsx_to_lsj_with_progress(source_path, dest_path, &progress_cb)
                .map_err(|e| e.to_string())
        }
        ("lsj", "lsx") => {
            let progress_cb = |msg: &str| {
                send_progress(msg.to_string());
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            maclarian::converter::lsj_to_lsx_with_progress(source_path, dest_path, &progress_cb)
                .map_err(|e| e.to_string())
        }
        ("lsf", "lsj") => {
            let progress_cb = |msg: &str| {
                send_progress(msg.to_string());
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            maclarian::converter::lsf_to_lsj_with_progress(source_path, dest_path, &progress_cb)
                .map_err(|e| e.to_string())
        }
        ("lsj", "lsf") => {
            let progress_cb = |msg: &str| {
                send_progress(msg.to_string());
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            maclarian::converter::lsj_to_lsf_with_progress(source_path, dest_path, &progress_cb)
                .map_err(|e| e.to_string())
        }
        _ => {
            unreachable!("Unsupported conversion: {} -> {}", source_ext, target_format)
        }
    };

    match result {
        Ok(_) => ConversionResult {
            success: true,
            message: format!("Converted to {}", target_format.to_uppercase()),
        },
        Err(e) => ConversionResult {
            success: false,
            message: format!("Conversion failed: {}", e),
        },
    }
}

/// Clean up temporary files created by the browser
pub fn cleanup_temp_files() {
    // Clean up loca preview temp file
    let _ = std::fs::remove_file("/tmp/temp_loca.xml");
}

/// Convert a GR2 file with bundle options
pub fn convert_gr2_file(state: BrowserState, config_state: crate::gui::state::ConfigState) {
    let Some(gr2_path) = state.gr2_convert_path.get() else {
        return;
    };

    // Capture options before closing dialog
    let keep_gr2 = state.gr2_extract_gr2.get();
    let convert_to_glb = state.gr2_convert_to_glb.get();
    let convert_to_gltf = state.gr2_convert_to_gltf.get();
    let extract_textures = state.gr2_extract_textures.get();
    let convert_to_png = state.gr2_convert_to_png.get();
    let game_data = config_state.bg3_data_path.get();

    // Close the dialog
    state.show_gr2_dialog.set(false);
    state.gr2_convert_path.set(None);

    // Check if any conversion is requested
    if !convert_to_glb && !convert_to_gltf && !extract_textures {
        state.status_message.set("No conversion options selected".to_string());
        return;
    }

    // Show loading overlay
    state.is_loading.set(true);
    state.loading_message.set("Converting GR2...".to_string());

    let state_for_callback = state.clone();
    let send = create_ext_action(Scope::new(), move |result: GR2ConversionResult| {
        state_for_callback.is_loading.set(false);
        if result.success {
            state_for_callback.status_message.set(result.message);
            // Refresh the file list to show new files
            if let Some(current) = state_for_callback.current_path.get() {
                load_directory(&current, state_for_callback.clone());
            }
        } else {
            state_for_callback.status_message.set(format!("Error: {}", result.message));
        }
    });

    // Get output directory (same as GR2 file location)
    let output_dir = std::path::Path::new(&gr2_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    rayon::spawn(move || {
        let result = perform_gr2_conversion(
            &gr2_path,
            &output_dir,
            keep_gr2,
            convert_to_glb,
            convert_to_gltf,
            extract_textures,
            convert_to_png,
            &game_data,
        );
        send(result);
    });
}

/// Result of GR2 conversion
struct GR2ConversionResult {
    success: bool,
    message: String,
}

/// Perform GR2 conversion (runs in background thread)
fn perform_gr2_conversion(
    gr2_path: &str,
    output_dir: &std::path::Path,
    keep_gr2: bool,
    convert_to_glb: bool,
    convert_to_gltf: bool,
    extract_textures: bool,
    convert_to_png: bool,
    game_data_path: &str,
) -> GR2ConversionResult {
    use std::path::Path;

    let gr2_file = Path::new(gr2_path);
    let file_stem = gr2_file.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());

    // Use subfolder if doing more than just GR2->GLB (glTF or textures)
    let use_subfolder = convert_to_gltf || extract_textures;
    let actual_output_dir = if use_subfolder {
        let subfolder = output_dir.join(&file_stem);
        if let Err(e) = std::fs::create_dir_all(&subfolder) {
            return GR2ConversionResult {
                success: false,
                message: format!("Failed to create output folder: {}", e),
            };
        }
        subfolder
    } else {
        output_dir.to_path_buf()
    };

    let mut results = Vec::new();

    // Convert to GLB or glTF
    if convert_to_glb || convert_to_gltf {
        let ext = if convert_to_glb { "glb" } else { "gltf" };
        let output_path = actual_output_dir.join(format!("{}.{}", file_stem, ext));

        let result = if convert_to_glb {
            maclarian::converter::convert_gr2_to_glb(gr2_file, &output_path)
        } else {
            maclarian::converter::convert_gr2_to_gltf(gr2_file, &output_path)
        };

        match result {
            Ok(()) => {
                results.push(format!("{}", ext.to_uppercase()));
            }
            Err(e) => return GR2ConversionResult {
                success: false,
                message: format!("Failed to convert to {}: {}", ext.to_uppercase(), e),
            },
        }
    }

    // Extract textures if requested
    if extract_textures {
        let options = maclarian::gr2_extraction::Gr2ExtractionOptions {
            convert_to_glb: false, // Already converted above if needed
            extract_textures: true,
            game_data_path: if game_data_path.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(game_data_path))
            },
            virtual_textures_path: None,
            convert_to_png,
            // Keep DDS if "Extract textures DDS" is checked (even if also converting to PNG)
            keep_original_dds: true,
        };

        match maclarian::gr2_extraction::process_extracted_gr2_to_dir(gr2_file, &actual_output_dir, &options) {
            Ok(tex_result) => {
                if !tex_result.texture_paths.is_empty() {
                    // Always keep DDS when extract_textures is checked
                    let format = if convert_to_png { "DDS + PNG" } else { "DDS" };
                    results.push(format!("{} {} textures", tex_result.texture_paths.len(), format));
                }
                for warning in tex_result.warnings {
                    eprintln!("Texture extraction warning: {}", warning);
                }
            }
            Err(e) => {
                // Don't fail the whole operation, just report the warning
                eprintln!("Texture extraction failed: {}", e);
            }
        }
    }

    // Copy original GR2 to output directory if requested
    if keep_gr2 && use_subfolder {
        let gr2_dest = actual_output_dir.join(gr2_file.file_name().unwrap_or_default());
        let _ = std::fs::copy(gr2_file, &gr2_dest);
    }

    let folder_info = if use_subfolder {
        format!(" in {}/", file_stem)
    } else {
        String::new()
    };

    if results.is_empty() {
        GR2ConversionResult {
            success: true,
            message: "No output files created".to_string(),
        }
    } else {
        GR2ConversionResult {
            success: true,
            message: format!("Created{}: {}", folder_info, results.join(", ")),
        }
    }
}
