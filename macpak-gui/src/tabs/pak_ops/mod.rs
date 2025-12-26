//! PAK Operations Tab
//!
//! Extract, create, list, and validate PAK files with progress tracking.
//! Matches the layout of the original PyQt6 mac-pak implementation.

mod dialogs;
mod operations;
mod operations_ui;
mod results;
mod types;
mod widgets;

use floem::prelude::*;

use crate::state::{AppState, PakOpsState};
use dialogs::{create_options_dialog, drop_action_dialog, progress_overlay};
use operations_ui::operations_row;
use results::results_area;

pub fn pak_ops_tab(_app_state: AppState, pak_state: PakOpsState) -> impl IntoView {
    let state = pak_state.clone();
    let state2 = pak_state.clone();
    let state3 = pak_state.clone();

    // Main content with dialog overlays using absolute positioning
    // The v_stack has position: Relative so absolutely positioned children
    // are positioned relative to it
    v_stack((
        // Operations row - 3 columns
        operations_row(pak_state.clone()),
        // Results area
        results_area(pak_state.clone()),
        // Dialog overlays - absolutely positioned so they don't affect layout
        progress_overlay(state),
        create_options_dialog(state2),
        drop_action_dialog(state3),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .padding(20.0)
            .position(floem::style::Position::Relative)
    })
}
