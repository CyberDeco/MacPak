//! Native menu placeholder
//!
//! CMD+F handling is done via Floem's event system instead.

use floem::prelude::*;
use crate::state::EditorTabsState;

pub fn setup_native_menu(_editor_tabs_state: EditorTabsState, _active_tab: RwSignal<usize>) {
    // No-op - keyboard shortcuts handled via Floem events
}
