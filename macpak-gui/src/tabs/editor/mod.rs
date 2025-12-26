//! Universal Editor Tab
//!
//! Text editor for LSX, LSJ, and LSF files using Floem's text_editor
//! (the same component that powers Lapce).

mod components;
mod formatting;
mod operations;
mod search;
mod syntax;

use floem::prelude::*;

use crate::state::{AppState, EditorState};
use components::{editor_content, editor_status_bar, editor_toolbar, search_panel};

// Re-export load_file for external use
pub use operations::load_file;

pub fn editor_tab(_app_state: AppState, editor_state: EditorState) -> impl IntoView {
    v_stack((
        editor_toolbar(editor_state.clone()),
        search_panel(editor_state.clone()),
        editor_content(editor_state.clone()),
        editor_status_bar(editor_state),
    ))
    .style(|s| s.width_full().height_full())
}
