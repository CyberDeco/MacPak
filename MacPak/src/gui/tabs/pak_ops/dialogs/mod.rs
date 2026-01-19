//! Dialog overlays for PAK operations
//!
//! This module provides a unified dialog overlay system that uses a single
//! `dyn_container` checking a single `ActiveDialog` signal. This replaces
//! the previous approach of 5 separate dialog overlays, which caused
//! event delivery delays due to excessive signal checking.

mod create_options;
mod drop_action;
mod file_select;
mod folder_drop_action;
mod progress;

use floem::action::exec_after;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem_reactive::create_effect;
use std::time::Duration;

use crate::gui::state::{ActiveDialog, PakOpsState};
use super::types::get_shared_progress;

use create_options::create_options_content;
use drop_action::drop_action_content;
use file_select::file_select_content;
use folder_drop_action::folder_drop_action_content;
use progress::progress_content;

/// Unified dialog overlay - only ONE dyn_container checking ONE signal
pub fn dialog_overlay(state: PakOpsState) -> impl IntoView {
    let active = state.active_dialog;

    // Progress polling state - only used when Progress dialog is active
    let polled_pct = RwSignal::new(0u32);
    let polled_current = RwSignal::new(0u32);
    let polled_total = RwSignal::new(0u32);
    let polled_msg = RwSignal::new(String::new());
    let timer_active = RwSignal::new(false);

    // Polling function for progress
    fn poll_and_schedule(
        polled_pct: RwSignal<u32>,
        polled_current: RwSignal<u32>,
        polled_total: RwSignal<u32>,
        polled_msg: RwSignal<String>,
        active: RwSignal<ActiveDialog>,
        timer_active: RwSignal<bool>,
    ) {
        let shared = get_shared_progress();
        let pct = shared.get_pct();
        let (current, total) = shared.get_counts();
        let msg = shared.get_message();

        polled_pct.set(pct);
        polled_current.set(current);
        polled_total.set(total);
        if !msg.is_empty() {
            polled_msg.set(msg);
        }

        if matches!(active.get_untracked(), ActiveDialog::Progress) && timer_active.get_untracked() {
            exec_after(Duration::from_millis(50), move |_| {
                if matches!(active.get_untracked(), ActiveDialog::Progress) && timer_active.get_untracked() {
                    poll_and_schedule(polled_pct, polled_current, polled_total, polled_msg, active, timer_active);
                }
            });
        }
    }

    // Start/stop progress polling based on active dialog
    create_effect(move |_| {
        let dialog = active.get();
        if matches!(dialog, ActiveDialog::Progress) {
            get_shared_progress().reset();
            polled_pct.set(0);
            polled_current.set(0);
            polled_total.set(0);
            polled_msg.set("Starting...".to_string());
            timer_active.set(true);

            exec_after(Duration::from_millis(50), move |_| {
                if matches!(active.get_untracked(), ActiveDialog::Progress) {
                    poll_and_schedule(polled_pct, polled_current, polled_total, polled_msg, active, timer_active);
                }
            });
        } else {
            timer_active.set(false);
        }
    });

    let state_for_content = state.clone();
    let state_for_escape = state.clone();

    dyn_container(
        move || active.get(),
        move |dialog| {
            let state = state_for_content.clone();
            match dialog {
                ActiveDialog::None => empty().into_any(),

                ActiveDialog::Progress => {
                    progress_content(polled_pct, polled_current, polled_total, polled_msg).into_any()
                }

                ActiveDialog::CreateOptions => {
                    create_options_content(state).into_any()
                }

                ActiveDialog::DropAction => {
                    drop_action_content(state).into_any()
                }

                ActiveDialog::FileSelect => {
                    file_select_content(state).into_any()
                }

                ActiveDialog::FolderDropAction => {
                    folder_drop_action_content(state).into_any()
                }
            }
        },
    )
    .style(move |s| {
        if active.get() != ActiveDialog::None {
            s.position(floem::style::Position::Absolute)
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
    .on_event_stop(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(key_event) = e {
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                let state = state_for_escape.clone();
                match state.active_dialog.get() {
                    ActiveDialog::None => {}
                    ActiveDialog::Progress => {} // Can't cancel progress
                    ActiveDialog::CreateOptions => {
                        state.pending_create.set(None);
                        state.active_dialog.set(ActiveDialog::None);
                    }
                    ActiveDialog::DropAction => {
                        state.dropped_file.set(None);
                        state.active_dialog.set(ActiveDialog::None);
                    }
                    ActiveDialog::FileSelect => {
                        state.file_select_pak.set(None);
                        state.file_select_list.set(Vec::new());
                        state.file_select_selected.set(std::collections::HashSet::new());
                        state.clear_results();
                        state.active_dialog.set(ActiveDialog::None);
                    }
                    ActiveDialog::FolderDropAction => {
                        state.dropped_folder.set(None);
                        state.active_dialog.set(ActiveDialog::None);
                    }
                }
            }
        }
    })
    .keyboard_navigable()
}
