//! Dialogue tab for viewing BG3 dialog files
//!
//! Features:
//! - Browse dialogs from PAK files or extracted folders
//! - View dialog tree structure with localized text
//! - Export to HTML and DE2 formats
//! - Audio playback via right-click context menu

mod toolbar;
mod browser;
mod tree_view;
mod context_menu;
pub mod operations;

use floem::prelude::*;
use floem::reactive::create_effect;
use floem::style::{FlexDirection, Position};
use floem::text::Weight;
use crate::gui::state::{AppState, ConfigState, DialogEntry, DialogSource, DialogueState};

pub use operations::{open_dialog_folder, load_dialog_from_pak};

/// Main dialogue tab view
pub fn dialogue_tab(_app_state: AppState, state: DialogueState, config: ConfigState) -> impl IntoView {
    let state_for_content = state.clone();
    let state_for_overlay = state.clone();
    let state_for_pending = state.clone();

    // Watch for pending load - when caches are ready, load the dialog
    create_effect(move |_| {
        let pending = state_for_pending.pending_load.get();
        let ready = state_for_pending.pending_caches_ready.get();

        if let Some(source) = pending {
            if ready {
                // Clear pending state first
                state_for_pending.pending_load.set(None);
                state_for_pending.pending_caches_ready.set(false);

                // Now load the dialog
                match source.clone() {
                    DialogSource::PakFile { pak_path, internal_path } => {
                        // Add to browser list if not already there
                        let display_path = format!("[Search] {}", internal_path);
                        let name = internal_path.split('/').last()
                            .unwrap_or(&internal_path)
                            .to_string();

                        let mut dialogs = state_for_pending.available_dialogs.get();
                        if !dialogs.iter().any(|d| d.path == display_path) {
                            dialogs.push(DialogEntry {
                                name,
                                path: display_path.clone(),
                                source: source.clone(),
                            });
                            state_for_pending.available_dialogs.set(dialogs);
                        }

                        // Select it in the browser
                        state_for_pending.selected_dialog_path.set(Some(display_path));

                        // Load the dialog
                        operations::load_dialog_from_pak(
                            state_for_pending.clone(),
                            pak_path,
                            internal_path,
                        );
                    }
                    DialogSource::LocalFile(_path) => {
                        state_for_pending.status_message.set("Loading not supported for local files from Search".to_string());
                    }
                }
            }
        }
    });

    // Main content
    let main_content = v_stack((
        // Toolbar
        toolbar::toolbar(state.clone(), config),

        // Main content area - horizontal split with adjustable divider
        dialogue_content(state.clone(), state_for_content),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .flex_direction(FlexDirection::Column)
            .background(Color::rgb8(250, 250, 250))
    });

    // Stack main content with loading overlay
    (main_content, loading_overlay(state_for_overlay))
        .style(|s| s.width_full().height_full().position(Position::Relative))
}

/// Main content area with adjustable split between browser and tree view
fn dialogue_content(state: DialogueState, state_for_content: DialogueState) -> impl IntoView {
    let browser_panel_width = state.browser_panel_width;

    // Drag state - tracked at parent level gets events even when pointer leaves divider
    let is_dragging = RwSignal::new(false);
    // Offset from mouse position to divider edge (where within the divider you clicked)
    let drag_offset = RwSignal::new(0.0);

    let content = h_stack((
        // Left panel: Dialog browser - width controlled by signal
        browser::browser_panel(state.clone())
            .style(move |s| s.width(browser_panel_width.get()).height_full()),

        // Draggable divider
        divider_handle(is_dragging, drag_offset),

        // Right panel: Dialog tree view and details
        dialog_content(state_for_content)
            .style(|s| s.flex_grow(1.0).height_full().min_width(0.0)),
    ))
    .style(move |s| {
        s.flex_grow(1.0)
            .width_full()
            .min_height(0.0)
    });

    // Transparent overlay that appears during drag to capture all pointer events
    let drag_overlay = empty()
        .style(move |s| {
            let dragging = is_dragging.get();
            if dragging {
                s.position(floem::style::Position::Absolute)
                    .inset_left(0.0)
                    .inset_top(0.0)
                    .inset_right(0.0)
                    .inset_bottom(0.0)
                    .z_index(100)
                    .cursor(floem::style::CursorStyle::ColResize)
            } else {
                // When not dragging, make overlay non-interactive and invisible
                s.display(floem::style::Display::None)
            }
        })
        .on_event(floem::event::EventListener::PointerMove, move |e| {
            if is_dragging.get() {
                if let floem::event::Event::PointerMove(pe) = e {
                    let new_width = (pe.pos.x - drag_offset.get()).clamp(200.0, 600.0);
                    browser_panel_width.set(new_width);
                }
            }
            floem::event::EventPropagation::Stop
        })
        .on_event_stop(floem::event::EventListener::PointerUp, move |_| {
            is_dragging.set(false);
        });

    // Stack content with overlay on top
    (content, drag_overlay)
        .style(|s| {
            s.position(floem::style::Position::Relative)
                .flex_grow(1.0)
                .width_full()
                .min_height(0.0)
        })
}

/// Divider handle that initiates drag
fn divider_handle(
    is_dragging: RwSignal<bool>,
    drag_offset: RwSignal<f64>,
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
                drag_offset.set(pe.pos.x);
            }
        })
}

/// Dialog content area (tree view + details)
/// Uses a stack with CSS visibility instead of dyn_container to prevent
/// tree view recreation which was causing browser scroll reset
fn dialog_content(state: DialogueState) -> impl IntoView {
    let state_for_tree = state.clone();
    let current_dialog = state.current_dialog;

    // Stack both views - only one is visible at a time via CSS
    // This prevents recreating tree_view_panel when switching dialogs
    (
        // Empty state - shown when no dialog loaded
        empty_state()
            .style(move |s| {
                let has_dialog = current_dialog.get().is_some();
                if has_dialog {
                    s.display(floem::style::Display::None)
                } else {
                    s.width_full().height_full()
                }
            }),
        // Tree view - shown when dialog is loaded
        tree_view::tree_view_panel(state_for_tree)
            .style(move |s| {
                let has_dialog = current_dialog.get().is_some();
                if has_dialog {
                    s.width_full().height_full()
                } else {
                    s.display(floem::style::Display::None)
                }
            }),
    )
        .style(|s| s.width_full().height_full().min_width(0.0))
}

/// Empty state when no dialog is loaded
fn empty_state() -> impl IntoView {
    v_stack((
        label(|| "No Dialog Loaded")
            .style(|s| {
                s.font_size(18.0)
                    .font_weight(Weight::MEDIUM)
                    .color(Color::rgb8(100, 100, 100))
            }),
        label(|| "Select a dialog file from the browser\nor open a folder containing dialogs")
            .style(|s| {
                s.font_size(13.0)
                    .color(Color::rgb8(150, 150, 150))
                    .margin_top(8.0)
            }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .items_center()
            .justify_center()
            .padding(40.0)
    })
}

/// Loading overlay shown during cache initialization
fn loading_overlay(state: DialogueState) -> impl IntoView {
    let show = state.is_building_flag_index;
    let message = state.flag_index_message;

    dyn_container(
        move || show.get(),
        move |is_loading| {
            if is_loading {
                container(
                    label(move || message.get())
                        .style(|s| s.font_size(14.0)),
                )
                .style(|s| {
                    s.padding(24.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .min_width(300.0)
                })
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
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
