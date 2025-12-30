//! Asset Browser Tab
//!
//! Browse directories, view file metadata, and preview contents.

mod context_menu;
mod file_list;
mod operations;
mod preview;
mod preview_3d;
mod raw_img;
mod status_bar;
mod toolbar;

use floem::prelude::*;

use crate::state::{AppState, BrowserState, EditorTabsState};
use file_list::file_list;
use preview::preview_panel;
use status_bar::browser_status_bar;
use toolbar::browser_toolbar;

pub fn browser_tab(
    _app_state: AppState,
    browser_state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    v_stack((
        browser_toolbar(browser_state.clone()),
        browser_content(browser_state.clone(), editor_tabs_state, active_tab),
        browser_status_bar(browser_state),
    ))
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)  // Allow shrinking for scroll to work
    })
}

fn browser_content(
    state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    h_stack((
        // File list (left side)
        file_list(state.clone(), editor_tabs_state, active_tab),
        // Preview panel (right side)
        preview_panel(state),
    ))
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)  // Allow content area to shrink for scroll
    })
}
