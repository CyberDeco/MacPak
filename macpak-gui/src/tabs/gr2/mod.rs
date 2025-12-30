//! GR2 Conversion Tab
//!
//! Convert between GR2 (Granny 3D) and glTF/GLB formats:
//! - Single file conversion with operation buttons
//! - Batch conversion of directories
//! - Drag & drop support

mod conversion;
mod dialogs;
mod results;
mod sections;
pub mod types;

use floem::prelude::*;
use floem::style::Position;

use crate::state::{AppState, Gr2State};
use dialogs::progress_overlay;
use results::results_section;
use sections::{header_section, operations_row};

pub fn gr2_tab(_app_state: AppState, gr2_state: Gr2State) -> impl IntoView {
    let state = gr2_state.clone();

    v_stack((
        // Header with title and status message
        header_section(gr2_state.clone()),
        // Main content area
        v_stack((
            // Operations row - 3 columns
            operations_row(gr2_state.clone()),
            // Results area
            results_section(gr2_state.clone()),
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
        // Progress overlay (shown when converting) - absolutely positioned
        progress_overlay(state),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(250, 250, 250))
            .position(Position::Relative)
    })
}
