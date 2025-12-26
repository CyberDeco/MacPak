//! Preview panel for displaying file contents and images

use floem::prelude::*;
use floem::text::Weight;

use crate::state::BrowserState;
use super::raw_img::raw_img;

pub fn preview_panel(state: BrowserState) -> impl IntoView {
    let preview_name = state.preview_name;
    let preview_info = state.preview_info;
    let preview_content = state.preview_content;
    let preview_image = state.preview_image;

    dyn_container(
        move || preview_name.get().is_empty(),
        move |is_empty| {
            if is_empty {
                // Placeholder when no file is selected
                v_stack((
                    // Header matching the actual preview header style
                    label(|| "Preview")
                        .style(|s| {
                            s.font_size(16.0)
                                .font_weight(Weight::BOLD)
                                .width_full()
                                .padding(12.0)
                                .background(Color::rgb8(248, 248, 248))
                                .border_bottom(1.0)
                                .border_color(Color::rgb8(220, 220, 220))
                        }),
                    // Placeholder content
                    v_stack((
                        label(|| "📄").style(|s| s.font_size(64.0)),
                        label(|| "Select a file to preview")
                            .style(|s| s.font_size(16.0).color(Color::rgb8(120, 120, 120))),
                        label(|| "Click on a file in the list to see its contents")
                            .style(|s| s.font_size(13.0).color(Color::rgb8(160, 160, 160))),
                    ))
                    .style(|s| {
                        s.flex_grow(1.0)
                            .width_full()
                            .items_center()
                            .justify_center()
                            .gap(8.0)
                            .background(Color::WHITE)
                    }),
                ))
                .style(|s| {
                    s.width_full()
                        .height_full()
                })
                .into_any()
            } else {
                preview_content_view(preview_name, preview_info, preview_content, preview_image)
                    .into_any()
            }
        },
    )
    .style(|s| {
        s.width_pct(40.0)
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)
    })
}

fn preview_content_view(
    preview_name: RwSignal<String>,
    preview_info: RwSignal<String>,
    preview_content: RwSignal<String>,
    preview_image: RwSignal<(u64, Option<crate::state::RawImageData>)>,
) -> impl IntoView {
    v_stack((
        // Preview header
        v_stack((
            label(move || preview_name.get())
                .style(|s| s.font_size(16.0).font_weight(Weight::BOLD)),
            label(move || preview_info.get())
                .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
        ))
        .style(|s| {
            s.width_full()
                .padding(12.0)
                .gap(4.0)
                .background(Color::rgb8(248, 248, 248))
                .border_bottom(1.0)
                .border_color(Color::rgb8(220, 220, 220))
        }),
        // Preview content (either image or text)
        // Uses dyn_stack with version as key to force complete view recreation on each image change.
        scroll(
            dyn_stack(
                move || {
                    let (version, data) = preview_image.get();
                    vec![(version, data)]
                },
                |(version, _)| *version,  // Use version as unique key to force new view creation
                move |(version, img_data)| {
                    if let Some(data) = img_data.clone() {
                        // Display image using custom RawImg view (no PNG encoding needed)
                        container(
                            raw_img(data.width, data.height, data.rgba_data, version)
                                .style(|s| s.max_width_full().max_height_full())
                        )
                        .style(|s| {
                            s.width_full()
                                .height_full()
                                .padding(12.0)
                                .items_center()
                                .justify_center()
                        })
                        .into_any()
                    } else {
                        // Display text
                        label(move || preview_content.get())
                            .style(|s| {
                                s.width_full()
                                    .padding(12.0)
                                    .font_family("monospace".to_string())
                                    .font_size(12.0)
                            })
                            .into_any()
                    }
                },
            )
            .style(|s| s.width_full().flex_col()),
        )
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
                .background(Color::WHITE)
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .min_height(0.0)
    })
}
