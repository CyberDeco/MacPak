//! Index Search Tab
//!
//! Search across indexed PAK files for assets by name, type, or content.
//!
//! Two-phase search architecture:
//! - Quick search: Filename/path matching (instant, no extraction)
//! - Deep search: Content matching (extracts and searches directly)

mod all_matches_dialog;
mod context_menu;
mod extract_dialog;
mod operations;
mod results;
mod toolbar;

use floem::prelude::*;

use crate::gui::state::{AppState, ConfigState, DialogueState, EditorTabsState, SearchState};

use all_matches_dialog::all_matches_dialog;
use extract_dialog::extract_dialog;
use operations::{auto_load_cached_index, progress_overlay, search_overlay};
use results::{search_results, search_status_bar};
use toolbar::search_toolbar;

pub fn search_tab(
    _app_state: AppState,
    search_state: SearchState,
    config_state: ConfigState,
    editor_tabs_state: EditorTabsState,
    dialogue_state: DialogueState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    // Attempt to auto-load cached index on first visit
    auto_load_cached_index(search_state.clone());

    let active_filter = search_state.active_filter;
    v_stack((
        search_toolbar(search_state.clone(), config_state.clone()),
        search_results(
            search_state.clone(),
            active_filter,
            editor_tabs_state,
            dialogue_state,
            active_tab,
        ),
        search_status_bar(search_state.clone()),
        // Progress dialog overlay for indexing - absolutely positioned
        progress_overlay(search_state.clone()),
        // Search in progress overlay - absolutely positioned
        search_overlay(search_state.clone()),
        // All matches dialog - absolutely positioned
        all_matches_dialog(search_state.clone()),
        // Extraction options dialog - absolutely positioned
        extract_dialog(search_state, config_state),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .position(floem::style::Position::Relative)
    })
}
