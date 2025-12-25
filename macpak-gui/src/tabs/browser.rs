//! Asset Browser Tab
//!
//! Browse directories, view file metadata, and preview contents.

use floem::prelude::*;
use floem::event::EventPropagation;
use floem::text::Weight;
use floem::views::img;
use std::path::Path;
use std::time::UNIX_EPOCH;
use std::io::Cursor;

use crate::state::{AppState, BrowserState, EditorState, FileEntry, SortColumn};
use crate::tabs::load_file;

pub fn browser_tab(
    _app_state: AppState,
    browser_state: BrowserState,
    editor_state: EditorState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    v_stack((
        browser_toolbar(browser_state.clone()),
        browser_content(browser_state.clone(), editor_state, active_tab),
        browser_status_bar(browser_state),
    ))
    .style(|s| s.width_full().height_full())
}

fn browser_toolbar(state: BrowserState) -> impl IntoView {
    let state_open = state.clone();
    let state_up = state.clone();
    let state_refresh = state.clone();
    let state_path = state.clone();
    let state_search = state.clone();
    let state_filter = state.clone();
    let state_all = state.clone();
    let state_pak = state.clone();
    let state_lsx = state.clone();
    let state_lsj = state.clone();
    let state_lsf = state.clone();

    v_stack((
        // Row 1: Navigation + file path
        h_stack((
            button("📂 Open Folder").action(move || {
                open_folder_dialog(state_open.clone());
            }),
            button("⬆️ Up").action(move || {
                go_up(state_up.clone());
            }),
            button("🔄 Refresh").action(move || {
                refresh(state_refresh.clone());
            }),
            separator(),
            // File path display box
            label(move || {
                state_path
                    .current_path
                    .get()
                    .unwrap_or_else(|| "No folder selected".to_string())
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(6.0)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
                    .background(Color::WHITE)
                    .text_ellipsis()
            }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),

        // Row 2: Search + quick filters
        h_stack((
            text_input(state_search.search_query)
                .placeholder("Search files...")
                .style(|s| {
                    s.width(200.0)
                        .padding(6.0)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                }),
            button("🔎").action(move || {
                apply_filters(state_filter.clone());
            }),
            separator(),
            filter_button("All", state_all),
            filter_button("PAK", state_pak),
            filter_button("LSX", state_lsx),
            filter_button("LSJ", state_lsj),
            filter_button("LSF", state_lsf),
            empty().style(|s| s.flex_grow(1.0)),
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),
    ))
    .style(|s| {
        s.width_full()
            .padding(10.0)
            .gap(8.0)
            .background(Color::rgb8(245, 245, 245))
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn filter_button(filter_type: &'static str, state: BrowserState) -> impl IntoView {
    let current_filter = state.type_filter;
    let state_click = state.clone();

    button(filter_type)
        .style(move |s| {
            let is_active = current_filter.get() == filter_type;
            let s = s.padding_horiz(8.0).padding_vert(4.0).border_radius(4.0);

            if is_active {
                s.background(Color::rgb8(33, 150, 243)).color(Color::WHITE)
            } else {
                s.background(Color::rgb8(230, 230, 230))
                    .color(Color::rgb8(60, 60, 60))
                    .hover(|s| s.background(Color::rgb8(200, 200, 200)))
            }
        })
        .action(move || {
            state_click.type_filter.set(filter_type.to_string());
            apply_filters(state_click.clone());
        })
}

fn separator() -> impl IntoView {
    empty().style(|s| {
        s.width(1.0)
            .height(30.0)
            .background(Color::rgb8(200, 200, 200))
            .margin_horiz(4.0)
    })
}

fn sortable_header(
    name: &'static str,
    column: SortColumn,
    sort_column: RwSignal<SortColumn>,
    sort_ascending: RwSignal<bool>,
    state: BrowserState,
) -> impl IntoView {
    h_stack((
        label(move || {
            let current = sort_column.get();
            let asc = sort_ascending.get();
            if current == column {
                if asc {
                    format!("{} ▲", name)
                } else {
                    format!("{} ▼", name)
                }
            } else {
                name.to_string()
            }
        })
        .style(|s| s.font_weight(Weight::BOLD)),
    ))
    .style(move |s| {
        s.cursor(floem::style::CursorStyle::Pointer)
            .hover(|s| s.background(Color::rgb8(230, 230, 230)))
            .padding_vert(2.0)
            .padding_horiz(4.0)
            .border_radius(4.0)
            .flex_grow(if column == SortColumn::Name { 1.0 } else { 0.0 })
    })
    .on_click_stop(move |_| {
        let current = sort_column.get();
        if current == column {
            sort_ascending.set(!sort_ascending.get());
        } else {
            sort_column.set(column);
            sort_ascending.set(true);
        }
        sort_files(state.clone());
    })
}

fn sort_files(state: BrowserState) {
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

fn browser_content(
    state: BrowserState,
    editor_state: EditorState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    h_stack((
        // File list (left side)
        file_list(state.clone(), editor_state, active_tab),
        // Preview panel (right side)
        preview_panel(state),
    ))
    .style(|s| s.width_full().flex_grow(1.0))
}

fn file_list(
    state: BrowserState,
    editor_state: EditorState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let files = state.files;
    let selected = state.selected_index;
    let state_select = state.clone();
    let sort_column = state.sort_column;
    let sort_ascending = state.sort_ascending;

    let state_name = state.clone();
    let state_type = state.clone();
    let state_size = state.clone();
    let state_modified = state.clone();

    v_stack((
        // Column headers
        h_stack((
            sortable_header("Name", SortColumn::Name, sort_column, sort_ascending, state_name),
            sortable_header("Type", SortColumn::Type, sort_column, sort_ascending, state_type)
                .style(|s| s.width(60.0)),
            sortable_header("Size", SortColumn::Size, sort_column, sort_ascending, state_size)
                .style(|s| s.width(80.0)),
            sortable_header("Modified", SortColumn::Modified, sort_column, sort_ascending, state_modified)
                .style(|s| s.width(120.0)),
        ))
        .style(|s| {
            s.width_full()
                .padding(8.0)
                .gap(8.0)
                .background(Color::rgb8(240, 240, 240))
                .border_bottom(1.0)
                .border_color(Color::rgb8(200, 200, 200))
        }),
        // File rows
        scroll(
            dyn_stack(
                move || files.get(),
                |file| file.path.clone(),
                move |file| {
                    let state_row = state_select.clone();
                    let state_dbl = state_select.clone();
                    let editor_for_open = editor_state.clone();
                    let file_path = file.path.clone();
                    let file_for_select = file.clone();
                    let file_for_open = file.clone();
                    let idx = files.get().iter().position(|f| f.path == file_path);

                    file_row(file, selected, idx)
                        .on_click_stop(move |_| {
                            if let Some(i) = idx {
                                state_row.selected_index.set(Some(i));
                                select_file(&file_for_select, state_row.clone());
                            }
                        })
                        .on_double_click(move |_| {
                            open_file_or_folder(
                                &file_for_open,
                                state_dbl.clone(),
                                editor_for_open.clone(),
                                active_tab,
                            );
                            EventPropagation::Stop
                        })
                },
            )
            .style(|s| s.width_full().flex_col()),
        )
        .style(|s| s.width_full().flex_grow(1.0)),
    ))
    .style(|s| {
        s.width_pct(60.0)
            .height_full()
            .background(Color::WHITE)
            .border_right(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn file_row(file: FileEntry, selected: RwSignal<Option<usize>>, idx: Option<usize>) -> impl IntoView {
    let is_selected = move || selected.get() == idx;
    let icon = file.icon.clone();
    let name = file.name.clone();
    let file_type = file.file_type.clone();
    let size = file.size_formatted.clone();
    let modified = file.modified.clone();

    h_stack((
        // Icon + Name
        h_stack((
            label(move || icon.clone()).style(|s| s.width(24.0)),
            label(move || name.clone()).style(|s| s.flex_grow(1.0).text_ellipsis()),
        ))
        .style(|s| s.flex_grow(1.0).gap(4.0)),
        // Type
        label(move || file_type.clone()).style(|s| {
            s.width(60.0)
                .font_size(12.0)
                .color(Color::rgb8(100, 100, 100))
        }),
        // Size
        label(move || size.clone()).style(|s| {
            s.width(80.0)
                .font_size(12.0)
                .color(Color::rgb8(100, 100, 100))
        }),
        // Modified
        label(move || modified.clone()).style(|s| {
            s.width(120.0)
                .font_size(12.0)
                .color(Color::rgb8(100, 100, 100))
        }),
    ))
    .style(move |s| {
        let s = s
            .width_full()
            .padding(8.0)
            .gap(8.0)
            .items_center()
            .border_bottom(1.0)
            .border_color(Color::rgb8(245, 245, 245));

        if is_selected() {
            s.background(Color::rgb8(227, 242, 253))
        } else {
            s.background(Color::WHITE)
                .hover(|s| s.background(Color::rgb8(250, 250, 250)))
        }
    })
}

fn preview_panel(state: BrowserState) -> impl IntoView {
    let preview_name = state.preview_name;
    let preview_info = state.preview_info;
    let preview_content = state.preview_content;
    let preview_image = state.preview_image;

    v_stack((
        // Preview header
        v_stack((
            label(move || preview_name.get())
                .style(|s| s.font_size(16.0).font_weight(Weight::BOLD)),
            label(move || preview_info.get())
                .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
        ))
        .style(|s| {
            s.width_full()
                .padding(12.0)
                .gap(4.0)
                .background(Color::rgb8(248, 248, 248))
                .border_bottom(1.0)
                .border_color(Color::rgb8(220, 220, 220))
        }),
        // Preview content (either image or text)
        scroll(
            dyn_container(
                move || preview_image.get(),
                move |img_data| {
                    if let Some(data) = img_data {
                        // Display image
                        img(move || data.clone())
                            .style(|s| {
                                s.max_width_full()
                                    .max_height_full()
                                    .padding(12.0)
                            })
                            .into_any()
                    } else {
                        // Display text
                        label(move || preview_content.get())
                            .style(|s| {
                                s.width_full()
                                    .padding(12.0)
                                    .font_family("monospace".to_string())
                                    .font_size(12.0)
                            })
                            .into_any()
                    }
                },
            ),
        )
        .style(|s| s.width_full().flex_grow(1.0).background(Color::WHITE)),
    ))
    .style(|s| s.width_pct(40.0).height_full())
}

fn browser_status_bar(state: BrowserState) -> impl IntoView {
    h_stack((
        label(move || {
            format!(
                "{} files, {} folders",
                state.file_count.get(),
                state.folder_count.get()
            )
        })
        .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0)),
        empty().style(|s| s.flex_grow(1.0)),
        label(move || state.total_size.get())
            .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0)),
        empty().style(|s| s.width(16.0)),
        label(move || state.status_message.get())
            .style(|s| s.color(Color::rgb8(76, 175, 80)).font_size(12.0)),
    ))
    .style(|s| {
        s.width_full()
            .height(32.0)
            .padding_horiz(12.0)
            .items_center()
            .background(Color::rgb8(248, 248, 248))
            .border_top(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

// ============================================================================
// File Operations
// ============================================================================

/// Format file size for display
fn format_size(bytes: u64) -> String {
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

fn open_folder_dialog(state: BrowserState) {
    let dialog = rfd::FileDialog::new().set_title("Select Folder to Browse");

    if let Some(path) = dialog.pick_folder() {
        let path_str = path.to_string_lossy().to_string();
        load_directory(&path_str, state);
    }
}

fn go_up(state: BrowserState) {
    if let Some(current) = state.current_path.get() {
        if let Some(parent) = Path::new(&current).parent() {
            let parent_str = parent.to_string_lossy().to_string();
            load_directory(&parent_str, state);
        }
    }
}

fn refresh(state: BrowserState) {
    if let Some(path) = state.current_path.get() {
        load_directory(&path, state);
    }
}

fn load_directory(dir_path: &str, state: BrowserState) {
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        return;
    }

    state.current_path.set(Some(dir_path.to_string()));

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

fn apply_filters(state: BrowserState) {
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

fn select_file(file: &FileEntry, state: BrowserState) {
    state.preview_name.set(file.name.clone());
    state.preview_image.set(None); // Clear previous image

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
            state.preview_content.set(String::new());
            match load_dds_as_png(path) {
                Ok(png_data) => {
                    state.preview_image.set(Some(png_data));
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error loading DDS: {}]", e));
                }
            }
        }
        "png" | "jpg" | "jpeg" => {
            state.preview_content.set(String::new());
            match std::fs::read(path) {
                Ok(data) => {
                    state.preview_image.set(Some(data));
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error loading image: {}]", e));
                }
            }
        }
        "gr2" => {
            state.preview_content.set("[GR2 Model file]".to_string());
        }
        "wem" | "wav" => {
            state.preview_content.set("[Audio file]".to_string());
        }
        _ => {
            state.preview_content.set(format!("File type: {}", ext.to_uppercase()));
        }
    }
}

/// Load a DDS file and convert it to PNG bytes using image_dds
fn load_dds_as_png(path: &Path) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;
    use image::codecs::png::PngEncoder;
    use image_dds::image_from_dds;
    use image_dds::ddsfile::Dds;

    // Read the DDS file
    let dds_data = std::fs::read(path).map_err(|e| e.to_string())?;
    let dds = Dds::read(&mut Cursor::new(&dds_data)).map_err(|e| e.to_string())?;

    // Convert to an RGBA image
    let img = image_from_dds(&dds, 0).map_err(|e| e.to_string())?;

    // Encode as PNG
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    ).map_err(|e| e.to_string())?;

    Ok(png_data)
}

fn open_file_or_folder(
    file: &FileEntry,
    state: BrowserState,
    editor_state: EditorState,
    active_tab: RwSignal<usize>,
) {
    if file.is_dir {
        load_directory(&file.path, state);
    } else {
        // Open file in Editor tab
        let path = Path::new(&file.path);
        load_file(path, editor_state);
        // Switch to Editor tab (index 1)
        active_tab.set(1);
    }
}
