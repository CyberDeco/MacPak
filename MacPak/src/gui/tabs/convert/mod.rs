//! Convert Tab - Unified conversion operations
//!
//! Subtabs:
//! - LSF/LSX/LSJ (subtab 0) — LSF/LSX/LSJ/LOCA conversion
//! - GR2 (subtab 1) — reuses existing GR2 tab
//! - Virtual Textures (subtab 2) — reuses existing Virtual Textures tab

pub mod lsf;

pub use lsf::open_lsf_file;

use floem::prelude::*;
use floem::text::Weight;

use super::gr2::gr2_tab;
use super::virtual_textures::virtual_textures_tab;
use crate::gui::shared::{ThemeColors, theme_signal};
use crate::gui::state::{AppState, ConfigState, Gr2State, LsfConvertState, VirtualTexturesState};
use lsf::lsf_subtab;

pub fn convert_tab(
    app_state: AppState,
    lsf_state: LsfConvertState,
    gr2_state: Gr2State,
    vt_state: VirtualTexturesState,
    config_state: ConfigState,
) -> impl IntoView {
    let active_subtab: RwSignal<usize> = RwSignal::new(0);

    v_stack((
        // Subtab bar
        subtab_bar(active_subtab),
        // Subtab content
        subtab_content(
            active_subtab,
            app_state,
            lsf_state,
            gr2_state,
            vt_state,
            config_state,
        ),
    ))
    .style(|s| s.width_full().height_full())
}

fn subtab_bar(active_subtab: RwSignal<usize>) -> impl IntoView {
    h_stack((
        subtab_button("LSF/LSX/LSJ", 0, active_subtab),
        subtab_button("GR2", 1, active_subtab),
        subtab_button("Virtual Textures", 2, active_subtab),
    ))
    .style(move |s| {
        let colors = theme_signal()
            .map(|t| ThemeColors::for_theme(t.get().effective()))
            .unwrap_or_else(ThemeColors::dark);
        s.width_full()
            .height(36.0)
            .padding_horiz(16.0)
            .gap(2.0)
            .items_center()
            .background(colors.bg_surface)
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn subtab_button(
    label_text: &'static str,
    index: usize,
    active_subtab: RwSignal<usize>,
) -> impl IntoView {
    button(label_text)
        .style(move |s| {
            let is_active = active_subtab.get() == index;
            let s = s
                .padding_horiz(14.0)
                .padding_vert(6.0)
                .font_size(12.0)
                .font_weight(Weight::MEDIUM)
                .border_radius(4.0)
                .cursor(floem::style::CursorStyle::Pointer);

            if is_active {
                s.background(Color::rgb8(66, 133, 244))
                    .color(Color::WHITE)
            } else {
                s.background(Color::TRANSPARENT)
                    .color(Color::rgb8(100, 100, 100))
                    .hover(|s| s.background(Color::rgb8(235, 235, 235)))
            }
        })
        .action(move || {
            active_subtab.set(index);
        })
}

fn subtab_content(
    active_subtab: RwSignal<usize>,
    app_state: AppState,
    lsf_state: LsfConvertState,
    gr2_state: Gr2State,
    vt_state: VirtualTexturesState,
    config_state: ConfigState,
) -> impl IntoView {
    dyn_container(
        move || active_subtab.get(),
        move |subtab_index| match subtab_index {
            0 => lsf_subtab(lsf_state.clone()).into_any(),
            1 => gr2_tab(app_state.clone(), gr2_state.clone(), config_state.clone()).into_any(),
            2 => virtual_textures_tab(app_state.clone(), vt_state.clone(), config_state.clone())
                .into_any(),
            _ => lsf_subtab(lsf_state.clone()).into_any(),
        },
    )
    .style(|s| s.width_full().flex_grow(1.0).flex_basis(0.0).min_height(0.0))
}
