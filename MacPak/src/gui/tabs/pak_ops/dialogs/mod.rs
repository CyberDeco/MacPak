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
mod validate_choice;

use floem::action::exec_after;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem_reactive::create_effect;
use std::time::Duration;

use crate::gui::state::{ActiveDialog, ConfigState, PakOpsState};
use super::types::get_shared_progress;

use create_options::create_options_content;
use drop_action::drop_action_content;
use file_select::file_select_content;
use folder_drop_action::folder_drop_action_content;
use progress::progress_content;
use validate_choice::validate_choice_content;

/// Unified dialog overlay - only ONE dyn_container checking ONE signal
pub fn dialog_overlay(state: PakOpsState, config_state: ConfigState) -> impl IntoView {
    let active = state.active_dialog;

    // Use persistent state signals to avoid accumulation on tab switch
    let polled_pct = state.polled_pct;
    let polled_current = state.polled_current;
    let polled_total = state.polled_total;
    let polled_msg = state.polled_msg;
    let timer_active = state.timer_active;

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

    // Only register the effect ONCE - guard against re-registration on tab switch
    if !state.polling_effect_registered.get_untracked() {
        state.polling_effect_registered.set(true);

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
    }

    let state_for_content = state.clone();
    let state_for_escape = state.clone();
    let config_for_content = config_state;

    dyn_container(
        move || active.get(),
        move |dialog| {
            let state = state_for_content.clone();
            let config = config_for_content.clone();

            // When None, return empty - the dyn_container styling handles visibility
            if dialog == ActiveDialog::None {
                return empty().into_any();
            }

            // For active dialogs, just return the content - backdrop is on dyn_container
            match dialog {
                ActiveDialog::None => unreachable!(),
                ActiveDialog::Progress => {
                    progress_content(polled_pct, polled_current, polled_total, polled_msg).into_any()
                }
                ActiveDialog::CreateOptions => create_options_content(state).into_any(),
                ActiveDialog::DropAction => drop_action_content(state).into_any(),
                ActiveDialog::FileSelect => file_select_content(state, config).into_any(),
                ActiveDialog::FolderDropAction => folder_drop_action_content(state).into_any(),
                ActiveDialog::ValidateChoice => validate_choice_content(state).into_any(),
            }
        },
    )
    .style(move |s| {
        let is_active = active.get() != ActiveDialog::None;
        let s = s
            .position(floem::style::Position::Absolute)
            .inset_top(0.0)
            .inset_left(0.0)
            .inset_bottom(0.0)
            .inset_right(0.0)
            .items_center()
            .justify_center()
            .z_index(100);

        if is_active {
            s.background(Color::rgba8(0, 0, 0, 100))
        } else {
            // When no dialog, make it non-interactive
            s.display(floem::style::Display::None)
        }
    })
    .on_event_stop(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(key_event) = e {
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                let state = state_for_escape.clone();
                match state.active_dialog.get() {
                    ActiveDialog::None | ActiveDialog::Progress => {}
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
                        state.file_select_filter.set(String::new());
                        state.clear_results();
                        state.active_dialog.set(ActiveDialog::None);
                    }
                    ActiveDialog::FolderDropAction => {
                        state.dropped_folder.set(None);
                        state.active_dialog.set(ActiveDialog::None);
                    }
                    ActiveDialog::ValidateChoice => {
                        state.active_dialog.set(ActiveDialog::None);
                    }
                }
            }
        }
    })
    .keyboard_navigable()
}
