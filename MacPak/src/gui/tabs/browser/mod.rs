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
use floem::style::Position;

use crate::gui::state::{AppState, BrowserState, EditorTabsState};
use file_list::file_list;
use preview::preview_panel;
use status_bar::browser_status_bar;
use toolbar::browser_toolbar;

pub use preview_3d::kill_preview_process;
pub use operations::cleanup_temp_files;
pub use operations::open_folder_dialog;

pub fn browser_tab(
    _app_state: AppState,
    browser_state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let browser_state_overlay = browser_state.clone();

    let main_content = v_stack((
        browser_toolbar(browser_state.clone()),
        browser_content(browser_state.clone(), editor_tabs_state, active_tab),
        browser_status_bar(browser_state),
    ))
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)  // Allow shrinking for scroll to work
    });

    // Stack main content with loading overlay
    (main_content, loading_overlay(browser_state_overlay))
        .style(|s| {
            s.width_full()
                .height_full()
                .position(Position::Relative)
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

/// Loading overlay shown while a file conversion is in progress
fn loading_overlay(state: BrowserState) -> impl IntoView {
    let state_for_style = state.clone();

    dyn_container(
        move || {
            let is_loading = state.is_loading.get();
            let message = state.loading_message.get();
            if is_loading { Some(message) } else { None }
        },
        move |maybe_message| {
            if let Some(message) = maybe_message {
                container(
                    v_stack((
                        // Loading message
                        label(move || message.clone())
                            .style(|s| s.font_size(14.0).margin_bottom(16.0)),
                        // Indeterminate progress indicator (animated bar)
                        container(
                            container(empty())
                                .style(|s| {
                                    s.height_full()
                                        .width_pct(30.0)
                                        .background(Color::rgb8(76, 175, 80))
                                        .border_radius(4.0)
                                })
                        )
                        .style(|s| {
                            s.width_full()
                                .height(8.0)
                                .background(Color::rgb8(220, 220, 220))
                                .border_radius(4.0)
                        }),
                    ))
                    .style(|s| {
                        s.padding(24.0)
                            .background(Color::WHITE)
                            .border(1.0)
                            .border_color(Color::rgb8(200, 200, 200))
                            .border_radius(8.0)
                            .width(400.0)
                    }),
                )
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        let is_loading = state_for_style.is_loading.get();

        if is_loading {
            s.position(Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
}
