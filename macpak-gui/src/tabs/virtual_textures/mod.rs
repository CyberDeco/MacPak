//! Virtual Textures Extraction Tab
//!
//! Extract individual DDS textures from GTS/GTP virtual texture files:
//! - Single file extraction with operation buttons
//! - Batch extraction of directories
//! - Drag & drop support

mod dialogs;
mod extraction;
mod results;
mod sections;
pub mod types;

use floem::prelude::*;
use floem::style::Position;

use crate::state::{AppState, VirtualTexturesState};
use dialogs::progress_overlay;
use results::results_section;
use sections::{header_section, operations_row};

pub fn virtual_textures_tab(_app_state: AppState, vt_state: VirtualTexturesState) -> impl IntoView {
    let state = vt_state.clone();

    v_stack((
        // Header with title and status message
        header_section(vt_state.clone()),
        // Main content area
        v_stack((
            // Operations row
            operations_row(vt_state.clone()),
            // Results area
            results_section(vt_state.clone()),
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
        // Progress overlay (shown when extracting) - absolutely positioned
        progress_overlay(state),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(250, 250, 250))
            .position(Position::Relative)
    })
}
