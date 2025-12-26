//! Browser status bar showing file/folder counts and status messages

use floem::prelude::*;

use crate::state::BrowserState;

pub fn browser_status_bar(state: BrowserState) -> impl IntoView {
    h_stack((
        label(move || {
            format!(
                "{} files, {} folders",
                state.file_count.get(),
                state.folder_count.get()
            )
        })
        .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0)),
        empty().style(|s| s.flex_grow(1.0)),
        label(move || state.total_size.get())
            .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0)),
        empty().style(|s| s.width(16.0)),
        label(move || state.status_message.get())
            .style(|s| s.color(Color::rgb8(76, 175, 80)).font_size(12.0)),
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
}
