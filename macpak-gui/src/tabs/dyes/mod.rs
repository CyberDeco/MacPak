//! Dyes Tab - Custom dye color creator for BG3 modding

mod color_row;
mod helpers;
mod sections;

use floem::prelude::*;
use floem::text::Weight;

use crate::state::{AppState, DyesState};
use sections::{common_section, recommended_section, required_section};

pub fn dyes_tab(_app_state: AppState, state: DyesState) -> impl IntoView {
    let status = state.status_message;

    v_stack((
        // Header - matches PAK Ops style
        header_section(status),

        // Three horizontal columns: Required, Commonly Used, Recommended
        h_stack((
            required_section(state.clone(), status),
            common_section(state.clone(), status),
            recommended_section(state, status),
        ))
        .style(|s| {
            s.width_full()
                .items_start()
                .padding(24.0)
                .gap(16.0)
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(250, 250, 250))
    })
}

fn header_section(status: RwSignal<String>) -> impl IntoView {
    h_stack((
        label(|| "Dye Lab")
            .style(|s| s.font_size(18.0).font_weight(Weight::BOLD)),
        empty().style(|s| s.flex_grow(1.0)),
        // Status message
        dyn_container(
            move || status.get(),
            move |msg| {
                if msg.is_empty() {
                    empty().into_any()
                } else {
                    label(move || msg.clone())
                        .style(|s| {
                            s.padding_horiz(12.0)
                                .padding_vert(6.0)
                                .border_radius(4.0)
                                .font_size(12.0)
                                .background(Color::rgb8(232, 245, 233))
                                .color(Color::rgb8(46, 125, 50))
                        })
                        .into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(16.0)
            .gap(8.0)
            .items_center()
            .background(Color::WHITE)
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}
