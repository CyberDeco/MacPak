//! Editor status bar component

use floem::prelude::*;

use crate::gui::state::EditorTabsState;

pub fn editor_status_bar(tabs_state: EditorTabsState) -> impl IntoView {
    dyn_container(
        move || tabs_state.active_tab(),
        move |maybe_tab| {
            if let Some(tab) = maybe_tab {
                let file_path = tab.file_path;
                let modified = tab.modified;

                h_stack((
                    // File path
                    label(move || {
                        file_path
                            .get()
                            .unwrap_or_else(|| "No file loaded".to_string())
                    })
                    .style(|s| {
                        s.color(Color::rgb8(100, 100, 100))
                            .font_size(12.0)
                            .text_ellipsis()
                            .max_width(500.0)
                    }),
                    empty().style(|s| s.flex_grow(1.0)),
                    // Modified indicator
                    label(move || if modified.get() { "● Modified" } else { "" }.to_string())
                        .style(|s| {
                            s.color(Color::rgb8(255, 152, 0))
                                .font_size(12.0)
                                .margin_right(12.0)
                        }),
                ))
                .style(|s| {
                    s.width_full()
                        .height(32.0)
                        .padding_horiz(12.0)
                        .items_center()
                        .background(Color::rgb8(248, 248, 248))
                        .border_top(1.0)
                        .border_color(Color::rgb8(220, 220, 220))
                })
                .into_any()
            } else {
                h_stack((label(|| "No file loaded".to_string())
                    .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0)),))
                .style(|s| {
                    s.width_full()
                        .height(32.0)
                        .padding_horiz(12.0)
                        .items_center()
                        .background(Color::rgb8(248, 248, 248))
                        .border_top(1.0)
                        .border_color(Color::rgb8(220, 220, 220))
                })
                .into_any()
            }
        },
    )
}
