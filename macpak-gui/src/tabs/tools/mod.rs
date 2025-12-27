//! Tools Tab
//!
//! Utilities for BG3 modding:
//! - UUID Generator
//! - Handle Generator
//! - Color Picker
//! - Version Calculator

mod color;
mod handle;
mod helpers;
mod history;
pub mod meta_generator;
mod uuid;
mod version;
mod widgets;

use floem::prelude::*;

use crate::state::{AppState, ToolsState};
use color::color_picker_section;
use handle::handle_section;
use helpers::{clear_all, export_history};
use history::{color_history_section, handle_history_section, uuid_history_section};
use uuid::uuid_section;
use version::version_calculator_section;

pub fn tools_tab(_app_state: AppState, tools_state: ToolsState) -> impl IntoView {
    let state_export = tools_state.clone();
    let state_clear = tools_state.clone();

    scroll(
        v_stack((
            // Title and actions
            h_stack((

                // Status message
                status_bar(tools_state.status_message),
                empty().style(|s| s.flex_grow(1.0)),

                button("Export History").action(move || {
                    export_history(state_export.clone());
                }),
                button("Clear All").action(move || {
                    clear_all(state_clear.clone());
                }),
            ))
            .style(|s| s.width_full().gap(8.0).items_center().margin_bottom(12.0)),

            // Row 1: UUID and Handle generators
            h_stack((
                uuid_section(tools_state.clone()),
                handle_section(tools_state.clone()),
            ))
            .style(|s| s.width_full().gap(16.0)),

            // Row 2: Color Picker and Version Calculator
            h_stack((
                color_picker_section(tools_state.clone()),
                version_calculator_section(tools_state.clone()),
            ))
            .style(|s| s.width_full().gap(16.0).margin_top(10.0)),

            // Row 3: History sections
            h_stack((
                uuid_history_section(tools_state.clone()),
                handle_history_section(tools_state.clone()),
            ))
            .style(|s| s.width_full().gap(16.0).margin_top(10.0)),

            // Row 4: Color history
            color_history_section(tools_state),
        ))
        .style(|s| s.width_full().padding(24.0)),
    )
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(250, 250, 250))
    })
}

fn status_bar(status: RwSignal<String>) -> impl IntoView {
    dyn_container(
        move || status.get(),
        move |msg| {
            if msg.is_empty() {
                empty().into_any()
            } else {
                label(move || msg.clone())
                    .style(|s| {
                        s.width_full()
                            .padding(4.0)
                            .margin_top(2.0)
                            .margin_bottom(2.0)
                            .background(Color::rgb8(232, 245, 233))
                            .border_radius(4.0)
                            .color(Color::rgb8(46, 125, 50))
                            .font_size(12.0)
                    })
                    .into_any()
            }
        },
    )
}
