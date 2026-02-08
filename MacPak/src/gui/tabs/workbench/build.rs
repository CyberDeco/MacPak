//! Build controls for workbench projects

use floem::prelude::*;

use crate::gui::shared::{ThemeColors, theme_signal};
use crate::gui::state::WorkbenchState;

/// Build panel sidebar
pub fn build_panel(state: WorkbenchState) -> impl IntoView {
    let ws = state.workbench;
    let build_progress = state.build_progress;
    let state_for_build = state.clone();
    let state_for_validate = state.clone();

    v_stack((
        // Section title
        label(|| "Build").style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(15.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(colors.text_primary)
                .margin_bottom(12.0)
        }),
        // Build settings (read from manifest)
        dyn_container(
            move || ws.get(),
            move |maybe_ws| {
                if let Some(w) = maybe_ws {
                    let compression = w.manifest.build.compression.clone();
                    let output_dir = w.manifest.build.output_dir.clone();
                    let info_json = w.manifest.build.generate_info_json;

                    v_stack((
                        setting_row("Compression", compression),
                        setting_row("Output", output_dir),
                        setting_row(
                            "info.json",
                            if info_json { "Yes" } else { "No" }.to_string(),
                        ),
                    ))
                    .style(|s| s.width_full().gap(6.0).margin_bottom(16.0))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        // Validate button
        button("Validate")
            .action(move || {
                if let Some(ref w) = state_for_validate.workbench.get() {
                    let result = w.validate();
                    if result.valid {
                        state_for_validate
                            .result_message
                            .set(Some("Validation passed".to_string()));
                        state_for_validate.error_message.set(None);
                    } else {
                        let msg = result.warnings.join("\n");
                        state_for_validate.result_message.set(None);
                        state_for_validate
                            .error_message
                            .set(Some(format!("Validation issues:\n{}", msg)));
                    }
                }
            })
            .style(move |s| {
                let colors = theme_signal()
                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                    .unwrap_or_else(ThemeColors::dark);
                s.width_full()
                    .padding_vert(10.0)
                    .background(colors.bg_elevated)
                    .color(colors.text_primary)
                    .border(1.0)
                    .border_color(colors.border)
                    .border_radius(6.0)
                    .margin_bottom(8.0)
                    .hover(|s| s.background(colors.bg_hover))
            }),
        // Build button
        button("Build PAK")
            .action(move || {
                state_for_build
                    .build_progress
                    .set(Some("Building...".to_string()));
                state_for_build.error_message.set(None);
                state_for_build.result_message.set(None);

                let ws = state_for_build.workbench.get();
                if let Some(w) = ws {
                    match w.build() {
                        Ok(pak_path) => {
                            state_for_build.build_progress.set(None);
                            state_for_build.result_message.set(Some(format!(
                                "Built: {}",
                                pak_path.file_name().unwrap_or_default().to_string_lossy()
                            )));
                        }
                        Err(e) => {
                            state_for_build.build_progress.set(None);
                            state_for_build
                                .error_message
                                .set(Some(format!("Build failed: {}", e)));
                        }
                    }
                }
            })
            .style(move |s| {
                let colors = theme_signal()
                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                    .unwrap_or_else(ThemeColors::dark);
                s.width_full()
                    .padding_vert(10.0)
                    .background(colors.accent)
                    .color(colors.text_inverse)
                    .border_radius(6.0)
                    .hover(|s| s.background(colors.accent_hover))
            }),
        // Build progress
        dyn_container(
            move || build_progress.get(),
            move |msg| {
                if let Some(msg) = msg {
                    label(move || msg.clone())
                        .style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.font_size(12.0).color(colors.text_muted).margin_top(8.0)
                        })
                        .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
    .style(move |s| {
        let colors = theme_signal()
            .map(|t| ThemeColors::for_theme(t.get().effective()))
            .unwrap_or_else(ThemeColors::dark);
        s.width_full()
            .padding(16.0)
            .background(colors.bg_surface)
            .border(1.0)
            .border_color(colors.border)
            .border_radius(6.0)
    })
}

fn setting_row(name: &'static str, value: String) -> impl IntoView {
    h_stack((
        label(move || name).style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(12.0).color(colors.text_muted).min_width(80.0)
        }),
        label(move || value.clone()).style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(12.0).color(colors.text_primary)
        }),
    ))
    .style(|s| s.width_full().gap(8.0))
}
