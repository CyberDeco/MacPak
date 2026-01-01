//! PAK Operations Tab
//!
//! Extract, create, list, and validate PAK files with progress tracking.
//! Matches the layout of the original PyQt6 mac-pak implementation.

mod dialogs;
mod operations;
mod results;
mod sections;
mod types;
mod widgets;

use floem::prelude::*;
use floem::style::Position;

use crate::gui::state::{AppState, PakOpsState};
use dialogs::{create_options_dialog, drop_action_dialog, file_select_dialog, progress_overlay};
use results::results_area;
use sections::{header_section, operations_row};

pub fn pak_ops_tab(_app_state: AppState, pak_state: PakOpsState) -> impl IntoView {
    let state = pak_state.clone();
    let state2 = pak_state.clone();
    let state3 = pak_state.clone();
    let state4 = pak_state.clone();

    v_stack((
        // Header with title and status message
        header_section(pak_state.clone()),
        // Main content area
        v_stack((
            // Operations row - 3 columns
            operations_row(pak_state.clone()),
            // Results area
            results_area(pak_state.clone()),
        ))
        .style(|s| {
            s.width_full()
                .height_full()
                .min_height(0.0)
                .flex_grow(1.0)
                .flex_basis(0.0)
                .padding(24.0)
                .gap(16.0)
        }),
        // Dialog overlays - absolutely positioned so they don't affect layout
        progress_overlay(state),
        create_options_dialog(state2),
        drop_action_dialog(state3),
        file_select_dialog(state4),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(250, 250, 250))
            .position(Position::Relative)
    })
}
