//! Asset Browser Tab
//!
//! Browse directories, view file metadata, and preview contents.

use floem::prelude::*;
use floem::event::EventPropagation;
use floem::keyboard::{Key, NamedKey};
use floem::text::Weight;
use floem::peniko::{self, Blob};
use floem::{ViewId, View, taffy};
use floem::action::show_context_menu;
use floem::menu::{Menu, MenuItem};
use floem_reactive::create_effect;
use floem_renderer::Renderer;
use std::sync::Arc;
use std::path::Path;
use std::time::UNIX_EPOCH;
use std::process::Command;

use crate::state::{AppState, BrowserState, EditorState, FileEntry, RawImageData, SortColumn};
use crate::tabs::load_file;

// ============================================================================
// Custom RawImg View - Displays RGBA image data without PNG encoding
// ============================================================================

/// Custom image view that works with raw RGBA pixel data
/// Uses a dynamic cache key so each image gets its own texture slot
pub struct RawImg {
    id: ViewId,
    img: Option<peniko::Image>,
    content_node: Option<taffy::tree::NodeId>,
    cache_key: u64,
}

/// Create a raw image view from RGBA data, width, height, and cache key
pub fn raw_img(width: u32, height: u32, rgba_data: Vec<u8>, cache_key: u64) -> RawImg {
    let data = Arc::new(rgba_data.into_boxed_slice());
    let blob = Blob::new(data);
    let image = peniko::Image::new(blob, peniko::Format::Rgba8, width, height);
    raw_img_dynamic(move || image.clone(), cache_key)
}

fn raw_img_dynamic(image: impl Fn() -> peniko::Image + 'static, cache_key: u64) -> RawImg {
    let id = ViewId::new();
    create_effect(move |_| {
        id.update_state(image());
    });
    RawImg {
        id,
        img: None,
        content_node: None,
        cache_key,
    }
}

impl View for RawImg {
    fn id(&self) -> ViewId {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "RawImg".into()
    }

    fn update(&mut self, _cx: &mut floem::context::UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(img) = state.downcast::<peniko::Image>() {
            self.img = Some(*img);
            self.id.request_layout();
        }
    }

    fn layout(&mut self, cx: &mut floem::context::LayoutCx) -> taffy::tree::NodeId {
        cx.layout_node(self.id(), true, |_cx| {
            if self.content_node.is_none() {
                self.content_node = Some(self.id.new_taffy_node());
            }
            let content_node = self.content_node.unwrap();

            let (width, height) = self
                .img
                .as_ref()
                .map(|img| (img.width, img.height))
                .unwrap_or((0, 0));

            let style = floem::style::Style::new()
                .width((width as f64).px())
                .height((height as f64).px())
                .to_taffy_style();
            self.id.set_taffy_style(content_node, style);

            vec![content_node]
        })
    }

    fn paint(&mut self, cx: &mut floem::context::PaintCx) {
        if let Some(ref img) = self.img {
            let rect = self.id.get_content_rect();
            let hash_bytes = self.cache_key.to_le_bytes();
            cx.draw_img(
                floem_renderer::Img {
                    img: img.clone(),
                    hash: &hash_bytes,
                },
                rect,
            );
        }
    }
}

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
    .style(|s| s.width_full().height_full().min_height(0.0))
}

fn browser_toolbar(state: BrowserState) -> impl IntoView {
    let state_open = state.clone();
    let state_up = state.clone();
    let state_refresh = state.clone();
    let state_path = state.clone();
    let state_path_nav = state.clone();
    let state_search = state.clone();
    let state_filter = state.clone();
    let state_all = state.clone();
    let state_pak = state.clone();
    let state_lsx = state.clone();
    let state_lsj = state.clone();
    let state_lsf = state.clone();

    // Create a signal for the path input text
    let path_input = RwSignal::new(
        state_path.current_path.get().unwrap_or_default()
    );

    // Keep path_input in sync with current_path
    create_effect(move |_| {
        let current = state_path.current_path.get();
        if let Some(p) = current {
            if path_input.get() != p {
                path_input.set(p);
            }
        }
    });

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
            // Editable file path input
            clip(
                text_input(path_input)
                    .placeholder("Enter path or open folder...")
                    .style(|s| {
                        s.width_full()
                            .height_full()
                            .padding(6.0)
                    }),
            )
            .style(|s| {
                s.flex_grow(1.0)
                    .height(32.0)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
                    .background(Color::WHITE)
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
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0) // Start from 0 and grow, don't expand beyond parent
    })
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
    let state_keyboard_down = state.clone();
    let state_keyboard_up = state.clone();
    let state_keyboard_enter = state.clone();
    let editor_keyboard = editor_state.clone();

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
        // File rows with scroll - use min_height(0) to allow shrinking
        scroll(
            dyn_stack(
                move || files.get(),
                |file| file.path.clone(),
                move |file| {
                    let state_row = state_select.clone();
                    let state_dbl = state_select.clone();
                    let state_ctx = state_select.clone();
                    let editor_for_open = editor_state.clone();
                    let editor_for_ctx = editor_state.clone();
                    let file_path = file.path.clone();
                    let file_for_select = file.clone();
                    let file_for_open = file.clone();
                    let file_for_ctx = file.clone();
                    let idx = files.get().iter().position(|f| f.path == file_path);

                    file_row(file, selected, idx, state_row.clone())
                        .on_click_stop(move |_| {
                            // Cancel any ongoing rename when clicking elsewhere
                            state_row.renaming_path.set(None);
                            if let Some(i) = idx {
                                state_row.selected_index.set(Some(i));
                                select_file(&file_for_select, state_row.clone());
                            }
                        })
                        .on_double_click(move |_| {
                            // Only open text files in editor on double-click
                            open_file_or_folder_filtered(
                                &file_for_open,
                                state_dbl.clone(),
                                editor_for_open.clone(),
                                active_tab,
                            );
                            EventPropagation::Stop
                        })
                        .on_secondary_click(move |_| {
                            // Select the file first
                            if let Some(i) = idx {
                                state_ctx.selected_index.set(Some(i));
                                select_file(&file_for_ctx, state_ctx.clone());
                            }
                            // Show context menu
                            show_file_context_menu(
                                &file_for_ctx,
                                state_ctx.clone(),
                                editor_for_ctx.clone(),
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
            .flex_grow(1.0)
            .flex_basis(0.0)
            .background(Color::WHITE)
            .border_right(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
    .keyboard_navigable()
    .on_key_down(
        Key::Named(NamedKey::ArrowDown),
        |_| true,
        move |_| {
            let files_list = state_keyboard_down.files.get();
            let current = state_keyboard_down.selected_index.get();
            let new_idx = match current {
                Some(i) if i + 1 < files_list.len() => Some(i + 1),
                None if !files_list.is_empty() => Some(0),
                _ => current,
            };
            if new_idx != current {
                state_keyboard_down.selected_index.set(new_idx);
                if let Some(i) = new_idx {
                    if let Some(file) = files_list.get(i) {
                        select_file(file, state_keyboard_down.clone());
                    }
                }
            }
        },
    )
    .on_key_down(
        Key::Named(NamedKey::ArrowUp),
        |_| true,
        move |_| {
            let files_list = state_keyboard_up.files.get();
            let current = state_keyboard_up.selected_index.get();
            let new_idx = match current {
                Some(i) if i > 0 => Some(i - 1),
                None if !files_list.is_empty() => Some(0),
                _ => current,
            };
            if new_idx != current {
                state_keyboard_up.selected_index.set(new_idx);
                if let Some(i) = new_idx {
                    if let Some(file) = files_list.get(i) {
                        select_file(file, state_keyboard_up.clone());
                    }
                }
            }
        },
    )
    .on_key_down(
        Key::Named(NamedKey::Enter),
        |_| true,
        move |_| {
            let files_list = state_keyboard_enter.files.get();
            if let Some(i) = state_keyboard_enter.selected_index.get() {
                if let Some(file) = files_list.get(i) {
                    open_file_or_folder_filtered(
                        file,
                        state_keyboard_enter.clone(),
                        editor_keyboard.clone(),
                        active_tab,
                    );
                }
            }
        },
    )
}

fn file_row(file: FileEntry, selected: RwSignal<Option<usize>>, idx: Option<usize>, state: BrowserState) -> impl IntoView {
    let is_selected = move || selected.get() == idx;
    let icon = file.icon.clone();
    let name = file.name.clone();
    let file_type = file.file_type.clone();
    let size = file.size_formatted.clone();
    let modified = file.modified.clone();
    let file_path = file.path.clone();
    let file_path_for_rename = file.path.clone();

    let renaming_path = state.renaming_path;
    let rename_text = state.rename_text;

    h_stack((
        // Icon + Name (with inline rename support)
        h_stack((
            label(move || icon.clone()).style(|s| s.width(24.0)),
            dyn_container(
                move || {
                    let is_renaming = renaming_path.get().as_ref() == Some(&file_path);
                    is_renaming
                },
                {
                    let name = name.clone();
                    move |is_renaming| {
                        let file_path_inner = file_path_for_rename.clone();
                        let state_inner = state.clone();
                        let name_inner = name.clone();
                        if is_renaming {
                            let state_esc = state_inner.clone();
                            // Show text input for renaming
                            text_input(rename_text)
                                .style(|s| {
                                    s.width_full()
                                        .min_width(50.0)
                                        .padding(2.0)
                                        .border(1.0)
                                        .border_color(Color::rgb8(33, 150, 243))
                                        .border_radius(2.0)
                                        .background(Color::WHITE)
                                })
                                .on_key_down(
                                    Key::Named(NamedKey::Enter),
                                    |_| true,
                                    move |_| {
                                        // Confirm rename
                                        let new_name = state_inner.rename_text.get();
                                        if !new_name.is_empty() {
                                            perform_rename(&file_path_inner, &new_name, state_inner.clone());
                                        }
                                        state_inner.renaming_path.set(None);
                                    },
                                )
                                .on_key_down(
                                    Key::Named(NamedKey::Escape),
                                    |_| true,
                                    move |_| {
                                        // Cancel rename
                                        state_esc.renaming_path.set(None);
                                    },
                                )
                                .into_any()
                        } else {
                            // Show label
                            label(move || name_inner.clone())
                                .style(|s| s.flex_grow(1.0).text_ellipsis())
                                .into_any()
                        }
                    }
                },
            )
            .style(|s| s.flex_grow(1.0).min_width(0.0)),
        ))
        .style(|s| s.flex_grow(1.0).gap(4.0).min_width(0.0)),
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

/// Perform the actual file rename
fn perform_rename(old_path: &str, new_name: &str, state: BrowserState) {
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
        // Uses dyn_stack with version as key to force complete view recreation on each image change.
        scroll(
            dyn_stack(
                move || {
                    let (version, data) = preview_image.get();
                    vec![(version, data)]
                },
                |(version, _)| *version,  // Use version as unique key to force new view creation
                move |(version, img_data)| {
                    if let Some(data) = img_data.clone() {
                        // Display image using custom RawImg view (no PNG encoding needed)
                        container(
                            raw_img(data.width, data.height, data.rgba_data, version)
                                .style(|s| s.max_width_full().max_height_full())
                        )
                        .style(|s| {
                            s.width_full()
                                .height_full()
                                .padding(12.0)
                                .items_center()
                                .justify_center()
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
            )
            .style(|s| s.width_full().flex_col()),
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
    // Clear previous image - increment version to ensure UI updates
    let (version, _) = state.preview_image.get();
    state.preview_image.set((version + 1, None));

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

/// Maximum preview dimension (width or height) for resizing
/// Note: Kept small to avoid filling vger's texture atlas (each 256x256 RGBA = 256KB)
const MAX_PREVIEW_SIZE: u32 = 256;

/// Load a DDS file, resize for preview, and return raw RGBA data
fn load_dds_image(path: &Path) -> Result<RawImageData, String> {
    use dds::{Decoder, ColorFormat, ImageViewMut};

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
fn is_text_file(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "lsf" | "lsx" | "lsj" | "txt" | "xml" | "json" | "lua" | "md" | "cfg" | "ini" | "yaml" | "yml" | "toml"
    )
}

/// Open file or folder, but only open text files in editor (not images, audio, etc.)
fn open_file_or_folder_filtered(
    file: &FileEntry,
    state: BrowserState,
    editor_state: EditorState,
    active_tab: RwSignal<usize>,
) {
    if file.is_dir {
        load_directory(&file.path, state);
    } else if is_text_file(&file.extension) {
        // Only open text files in Editor tab
        let path = Path::new(&file.path);
        load_file(path, editor_state);
        active_tab.set(1);
    }
    // Non-text files: do nothing on double-click (preview is already shown)
}

/// Show context menu for a file entry
fn show_file_context_menu(
    file: &FileEntry,
    state: BrowserState,
    editor_state: EditorState,
    active_tab: RwSignal<usize>,
) {
    let file_path = file.path.clone();
    let file_ext = file.extension.clone();
    let is_dir = file.is_dir;
    let file_name = file.name.clone();

    let mut menu = Menu::new("");

    // Open in Editor (text files only)
    if !is_dir && is_text_file(&file_ext) {
        let path = file_path.clone();
        let editor = editor_state.clone();
        menu = menu.entry(
            MenuItem::new("Open in Editor")
                .action(move || {
                    load_file(Path::new(&path), editor.clone());
                    active_tab.set(1);
                })
        );
        menu = menu.separator();
    }

    // Show in Finder
    {
        let path = file_path.clone();
        menu = menu.entry(
            MenuItem::new("Show in Finder")
                .action(move || {
                    let _ = Command::new("open")
                        .arg("-R")
                        .arg(&path)
                        .spawn();
                })
        );
    }

    // Copy Path
    {
        let path = file_path.clone();
        menu = menu.entry(
            MenuItem::new("Copy Path")
                .action(move || {
                    if let Ok(mut child) = Command::new("pbcopy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                    {
                        if let Some(stdin) = child.stdin.as_mut() {
                            use std::io::Write;
                            let _ = stdin.write_all(path.as_bytes());
                        }
                    }
                })
        );
    }

    menu = menu.separator();

    // Rename (inline)
    {
        let path = file_path.clone();
        let name = file_name.clone();
        let browser_state = state.clone();
        menu = menu.entry(
            MenuItem::new("Rename")
                .action(move || {
                    // Start inline rename
                    browser_state.rename_text.set(name.clone());
                    browser_state.renaming_path.set(Some(path.clone()));
                })
        );
    }

    // Convert options (for LSX/LSF/LSJ files only)
    if !is_dir {
        let ext_lower = file_ext.to_lowercase();
        if matches!(ext_lower.as_str(), "lsx" | "lsf" | "lsj") {
            menu = menu.separator();

            // Convert to LSX (if not already LSX)
            if ext_lower != "lsx" {
                let path = file_path.clone();
                let browser_state = state.clone();
                menu = menu.entry(
                    MenuItem::new("Convert to LSX")
                        .action(move || {
                            convert_file_quick(&path, "lsx", browser_state.clone());
                        })
                );
            }

            // Convert to LSF (if not already LSF)
            if ext_lower != "lsf" {
                let path = file_path.clone();
                let browser_state = state.clone();
                menu = menu.entry(
                    MenuItem::new("Convert to LSF")
                        .action(move || {
                            convert_file_quick(&path, "lsf", browser_state.clone());
                        })
                );
            }

            // Convert to LSJ (if not already LSJ)
            if ext_lower != "lsj" {
                let path = file_path.clone();
                let browser_state = state.clone();
                menu = menu.entry(
                    MenuItem::new("Convert to LSJ")
                        .action(move || {
                            convert_file_quick(&path, "lsj", browser_state.clone());
                        })
                );
            }
        }
    }

    menu = menu.separator();

    // Delete
    {
        let path = file_path.clone();
        let browser_state = state.clone();
        let item_type = if is_dir { "folder" } else { "file" };
        menu = menu.entry(
            MenuItem::new(format!("Delete {}", item_type))
                .action(move || {
                    delete_file(&path, browser_state.clone());
                })
        );
    }

    show_context_menu(menu, None);
}

/// Quick convert a file to another format in the same directory
fn convert_file_quick(source_path: &str, target_format: &str, state: BrowserState) {
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

/// Delete a file or folder
fn delete_file(path: &str, state: BrowserState) {
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

