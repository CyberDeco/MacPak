//! Dialog file browser panel

use floem::prelude::*;
use floem::text::Weight;
use floem::views::{virtual_list, VirtualDirection, VirtualItemSize};
use im::Vector as ImVector;
use crate::gui::state::{DialogueState, DialogEntry};
use super::operations;

const ROW_HEIGHT: f64 = 44.0;
const ROW_PADDING: f64 = 24.0;  // 12px on each side
const CHAR_WIDTH: f64 = 7.0;    // estimated average character width at 13px font

/// Left panel showing available dialog files
pub fn browser_panel(state: DialogueState) -> impl IntoView {
    let state_for_list = state.clone();
    let state_for_search = state.clone();
    let state_for_count = state.clone();

    v_stack((
        // Search box
        search_box(state_for_search),

        // Count label
        label(move || {
            let count = state_for_count.available_dialogs.get().len();
            format!("{} dialogs", count)
        })
        .style(|s| {
            s.padding(8.0)
                .font_size(12.0)
                .color(Color::rgb8(100, 100, 100))
                .border_bottom(1.0)
                .border_color(Color::rgb8(230, 230, 230))
        }),

        // Dialog list using virtual_list like browser tab
        dialog_list(state_for_list, state.browser_panel_width),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .min_height(0.0)  // Critical for scroll to work
            .background(Color::WHITE)
            .border_right(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

/// Search box for filtering dialogs
fn search_box(state: DialogueState) -> impl IntoView {
    let search_text = state.dialog_search;

    // Wrap in container with padding to prevent overflow
    container(
        text_input(search_text)
            .placeholder("Search dialogs...")
            .style(|s| {
                s.width_full()
                    .padding(8.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .font_size(13.0)
            })
    )
    .style(|s| s.width_full().padding(8.0))
}

/// List of dialog files
fn dialog_list(state: DialogueState, panel_width: RwSignal<f64>) -> impl IntoView {
    let available = state.available_dialogs;
    let search_filter = state.dialog_search;
    let selected = state.selected_dialog_path;
    let state_for_items = state.clone();

    // Cache the filtered results to avoid returning a new collection on every call
    // which could cause floem to think data changed and reset scroll
    let cached_result: std::rc::Rc<std::cell::RefCell<(Vec<DialogEntry>, String, ImVector<DialogEntry>)>> =
        std::rc::Rc::new(std::cell::RefCell::new((Vec::new(), String::new(), ImVector::new())));
    let cache = cached_result.clone();

    // virtual_list handles empty data gracefully
    scroll(
        virtual_list(
            VirtualDirection::Vertical,
            VirtualItemSize::Fixed(Box::new(|| ROW_HEIGHT)),
            move || {
                let dialogs = available.get();
                let search = search_filter.get();

                // Check if we need to recompute
                let mut cache_ref = cache.borrow_mut();
                let (cached_dialogs, cached_search, cached_im) = &mut *cache_ref;

                // Only recompute if source data or search changed
                let dialogs_changed = cached_dialogs.len() != dialogs.len() ||
                    cached_dialogs.first().map(|d| &d.path) != dialogs.first().map(|d| &d.path);
                let search_changed = cached_search != &search;

                if dialogs_changed || search_changed {
                    let filtered: Vec<DialogEntry> = if search.is_empty() {
                        dialogs.clone()
                    } else {
                        let search_lower = search.to_lowercase();
                        dialogs.iter()
                            .filter(|d| d.name.to_lowercase().contains(&search_lower) ||
                                        d.path.to_lowercase().contains(&search_lower))
                            .cloned()
                            .collect()
                    };

                    *cached_dialogs = dialogs;
                    *cached_search = search;
                    *cached_im = filtered.into_iter().collect();
                }

                cached_im.clone()
            },
            |entry: &DialogEntry| entry.path.clone(),
            {
                move |entry: DialogEntry| {
                    let state_click = state_for_items.clone();
                    let entry_for_load = entry.clone();

                    dialog_row(entry, selected, panel_width)
                        // Stop PointerDown propagation to prevent scroll container from
                        // resetting scroll position when clicking on rows
                        .on_event_stop(floem::event::EventListener::PointerDown, |_| {})
                        .on_click_stop(move |_| {
                            operations::load_dialog_entry(state_click.clone(), entry_for_load.clone());
                        })
                }
            },
        )
        .style(|s| s.width_full().flex_col())
    )
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)
    })
}

/// Single dialog row in the list
fn dialog_row(
    entry: DialogEntry,
    selected_path: RwSignal<Option<String>>,
    panel_width: RwSignal<f64>,
) -> impl IntoView {
    let path = entry.path.clone();
    let name = entry.name.clone();
    // Show only the directory path, not the filename (already shown above)
    let display_path = entry.path
        .rfind(|c| c == '/' || c == '\\')
        .map(|i| entry.path[..i].to_string())
        .unwrap_or_default();

    // Capture panel width once - don't create reactive subscriptions in each row
    // Using get_untracked to avoid subscriptions that could cause scroll resets
    let initial_width = panel_width.get_untracked();

    h_stack((
        // Name and path - use fixed width to avoid reactive updates
        v_stack((
            {
                let name_for_label = name.clone();
                let available = initial_width - ROW_PADDING - 8.0;
                let max_chars = (available / CHAR_WIDTH).max(10.0) as usize;
                let truncated = truncate_middle(&name_for_label, max_chars);
                label(move || truncated.clone())
                    .style(move |s| {
                        s.font_size(13.0)
                            .font_weight(Weight::MEDIUM)
                            .color(Color::rgb8(40, 40, 40))
                            .width(available as f32)
                    })
            },
            {
                let path_for_label = display_path.clone();
                let available = initial_width - ROW_PADDING - 8.0;
                let max_chars = (available / 6.0).max(10.0) as usize;
                let truncated = truncate_middle(&path_for_label, max_chars);
                label(move || truncated.clone())
                    .style(move |s| {
                        s.font_size(11.0)
                            .color(Color::rgb8(120, 120, 120))
                            .width(available as f32)
                    })
            },
        ))
        .style(|s| s.flex_grow(1.0).min_width(0.0)),
    ))
    .style(move |s| {
        let selected = selected_path.get();
        let is_sel = selected.as_ref() == Some(&path);

        let base = s
            .width_full()
            .height(ROW_HEIGHT)
            .padding_horiz(12.0)
            .gap(8.0)
            .items_center()
            .border_bottom(1.0)
            .border_color(Color::rgb8(245, 245, 245))
            .cursor(floem::style::CursorStyle::Pointer);

        if is_sel {
            base.background(Color::rgb8(227, 242, 253))
        } else {
            base.background(Color::WHITE)
                .hover(|s| s.background(Color::rgb8(250, 250, 250)))
        }
    })
}

/// Truncate filename with middle ellipsis like macOS Finder
/// e.g., "VeryLongFileName.extension" -> "VeryLong...ension"
fn truncate_middle(name: &str, max_chars: usize) -> String {
    if max_chars < 8 || name.chars().count() <= max_chars {
        return name.to_string();
    }

    let chars: Vec<char> = name.chars().collect();
    let total = chars.len();

    // Try to preserve file extension
    let extension_start = name.rfind('.').unwrap_or(total);
    let extension_len = total - extension_start;

    // If extension is reasonable length, preserve it
    if extension_len > 0 && extension_len <= 10 && extension_start > 0 {
        let available = max_chars.saturating_sub(3).saturating_sub(extension_len); // 3 for "..."
        if available >= 4 {
            let prefix: String = chars[..available].iter().collect();
            let suffix: String = chars[extension_start..].iter().collect();
            return format!("{}...{}", prefix, suffix);
        }
    }

    // Otherwise, just split in the middle
    let half = (max_chars.saturating_sub(3)) / 2;
    let prefix: String = chars[..half].iter().collect();
    let suffix: String = chars[total - half..].iter().collect();
    format!("{}...{}", prefix, suffix)
}
