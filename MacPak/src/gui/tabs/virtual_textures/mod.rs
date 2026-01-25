//! Virtual Textures Extraction Tab
//!
//! Extract individual DDS textures from GTS/GTP virtual texture files:
//! - Single file extraction with operation buttons
//! - Batch extraction of directories
//! - Drag & drop support

mod extraction;
mod sections;
pub mod types;

pub use sections::open_gts_file;

use floem::prelude::*;
use floem::style::Position;

use crate::gui::shared::{header_section, progress_overlay, results_section};
use crate::gui::state::{AppState, ConfigState, VirtualTexturesState};
use sections::operations_row;

pub fn virtual_textures_tab(_app_state: AppState, vt_state: VirtualTexturesState, config_state: ConfigState) -> impl IntoView {
    let state = vt_state.clone();

    v_stack((
        // Header with title and status message (using shared component)
        header_section("Virtual Textures", vt_state.clone()),
        // Main content area
        v_stack((
            // Operations row
            operations_row(vt_state.clone(), config_state),
            // Results area (using shared component)
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
        // Progress overlay (using shared component)
        progress_overlay(state),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(250, 250, 250))
            .position(Position::Relative)
    })
}
