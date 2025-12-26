//! UUID Generator

use floem::prelude::*;
use floem::text::Weight;

use crate::state::{ToolsState, UuidFormat};
use super::widgets::{copy_button, format_button, tool_card_style};

pub fn uuid_section(state: ToolsState) -> impl IntoView {
    let uuid = state.generated_uuid;
    let format = state.uuid_format;
    let history = state.uuid_history;
    let status = state.status_message;

    v_stack((
        label(|| "UUID Generator").style(|s| s.font_size(16.0).font_weight(Weight::BOLD).margin_bottom(12.0)),

        // Format selection
        h_stack((
            label(|| "Format:").style(|s| s.margin_right(8.0).font_size(12.0)),
            format_button("Standard", UuidFormat::Standard, format),
            format_button("Compact", UuidFormat::Compact, format),
            format_button("Larian", UuidFormat::Larian, format),
        ))
        .style(|s| s.gap(4.0).items_center().margin_bottom(12.0)),

        // Generated UUID display
        h_stack((
            label(move || {
                let u = uuid.get();
                if u.is_empty() {
                    "Click Generate".to_string()
                } else {
                    u
                }
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(10.0)
                    .font_size(13.0)
                    .font_family("monospace".to_string())
                    .background(Color::rgb8(245, 245, 245))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
            copy_button(uuid, status),
        ))
        .style(|s| s.width_full().gap(6.0)),

        // Generate button
        button("Generate UUID")
            .style(|s| {
                s.width_full()
                    .padding_vert(10.0)
                    .margin_top(10.0)
                    .font_size(13.0)
                    .background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(25, 118, 210)))
            })
            .action(move || {
                let new_uuid = generate_uuid(format.get());
                uuid.set(new_uuid.clone());

                let mut hist = history.get();
                hist.insert(0, new_uuid);
                if hist.len() > 20 {
                    hist.truncate(20);
                }
                history.set(hist);
            }),
    ))
    .style(|s| tool_card_style(s))
}

pub fn generate_uuid(format: UuidFormat) -> String {
    let uuid = uuid::Uuid::new_v4();

    match format {
        UuidFormat::Standard => uuid.to_string().to_uppercase(),
        UuidFormat::Compact => uuid.simple().to_string().to_uppercase(),
        UuidFormat::Larian => {
            let simple = uuid.simple().to_string();
            format!(
                "h{}g{}g{}g{}g{}",
                &simple[0..8],
                &simple[8..12],
                &simple[12..16],
                &simple[16..20],
                &simple[20..32]
            )
        }
    }
}
