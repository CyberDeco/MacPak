//! Asset Browser Tab
//!
//! Browse directories, view file metadata, and preview contents.

mod context_menu;
mod file_list;
mod operations;
mod preview;
mod preview_3d;
mod raw_img;
mod status_bar;
mod toolbar;

use floem::prelude::*;

use crate::state::{AppState, BrowserState, EditorTabsState};
use file_list::file_list;
use preview::preview_panel;
use status_bar::browser_status_bar;
use toolbar::browser_toolbar;

pub use preview_3d::kill_preview_process;

pub fn browser_tab(
    _app_state: AppState,
    browser_state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    v_stack((
        browser_toolbar(browser_state.clone()),
        browser_content(browser_state.clone(), editor_tabs_state, active_tab),
        browser_status_bar(browser_state),
    ))
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)  // Allow shrinking for scroll to work
    })
}

fn browser_content(
    state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let file_list_width = state.file_list_width;

    // Drag state - tracked at parent level so we get events even when pointer leaves divider
    let is_dragging = RwSignal::new(false);
    // Offset from mouse position to divider edge (where within the divider you clicked)
    let drag_offset = RwSignal::new(0.0);

    h_stack((
        // File list (left side) - fixed width from signal
        file_list(state.clone(), editor_tabs_state, active_tab),
        // Draggable divider
        divider_handle(is_dragging, drag_offset, file_list_width),
        // Preview panel (right side) - takes remaining space
        preview_panel(state),
    ))
    .style(move |s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)
            .cursor(if is_dragging.get() {
                floem::style::CursorStyle::ColResize
            } else {
                floem::style::CursorStyle::Default
            })
    })
    .on_event(floem::event::EventListener::PointerMove, move |e| {
        if is_dragging.get() {
            if let floem::event::Event::PointerMove(pe) = e {
                // Mouse X in parent coords minus the click offset = new width
                let new_width = (pe.pos.x - drag_offset.get()).clamp(400.0, 800.0);
                file_list_width.set(new_width);
            }
        }
        floem::event::EventPropagation::Continue
    })
    .on_event_stop(floem::event::EventListener::PointerUp, move |_| {
        is_dragging.set(false);
    })
}

/// Divider handle that initiates drag
fn divider_handle(
    is_dragging: RwSignal<bool>,
    drag_offset: RwSignal<f64>,
    _width_signal: RwSignal<f64>,
) -> impl IntoView {
    empty()
        .style(move |s| {
            let dragging = is_dragging.get();
            s.width(6.0)
                .height_full()
                .cursor(floem::style::CursorStyle::ColResize)
                .background(if dragging {
                    Color::rgb8(33, 150, 243)
                } else {
                    Color::rgb8(200, 200, 200)
                })
                .hover(|s| s.background(Color::rgb8(150, 150, 150)))
        })
        .on_event_stop(floem::event::EventListener::PointerDown, move |e| {
            if let floem::event::Event::PointerDown(pe) = e {
                is_dragging.set(true);
                // pe.pos.x is position within divider (0-6), need to offset from the left edge
                // offset = parent_mouse_x - current_width = (width + pe.pos.x) - width = pe.pos.x
                drag_offset.set(pe.pos.x);
            }
        })
}
