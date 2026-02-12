//! LSF/LSX/LSJ/LOCA Conversion Subtab
//!
//! Convert between LSF, LSX, LSJ, and LOCA/XML formats:
//! - Single file conversion with auto-detection
//! - Batch conversion of directories
//! - Drag & drop support

mod conversion;
mod sections;
pub mod types;

pub use sections::open_lsf_file;

use floem::prelude::*;
use floem::style::Position;

use crate::gui::shared::{header_section, progress_overlay, results_section};
use crate::gui::state::LsfConvertState;
use sections::operations_row;

pub fn lsf_subtab(lsf_state: LsfConvertState) -> impl IntoView {
    let state = lsf_state.clone();

    v_stack((
        // Header with title and status message (using shared component)
        header_section("LSF / LSX / LSJ", lsf_state.clone()),
        // Main content area
        v_stack((
            // Operations row
            operations_row(lsf_state.clone()),
            // Results area (using shared component)
            results_section(lsf_state.clone()),
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
