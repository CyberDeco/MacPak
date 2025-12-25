//! UUID and Handle Generator Tab
//!
//! Generate UUIDs and Handles in various formats for BG3 modding.

use floem::prelude::*;
use floem::text::Weight;
use rand::Rng;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::state::{AppState, UuidGenState, UuidFormat};

pub fn uuid_gen_tab(_app_state: AppState, uuid_state: UuidGenState) -> impl IntoView {
    let state_export = uuid_state.clone();
    let state_clear = uuid_state.clone();

    v_stack((
        // Title and actions
        h_stack((
            label(|| "UUID & Handle Generator")
                .style(|s| s.font_size(24.0).font_weight(Weight::BOLD)),
            empty().style(|s| s.flex_grow(1.0)),
            button("📤 Export").action(move || {
                export_history(state_export.clone());
            }),
            button("🗑️ Clear All").action(move || {
                clear_history(state_clear.clone());
            }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center().margin_bottom(20.0)),

        // Status message
        status_bar(uuid_state.status_message),

        // Two-column layout
        h_stack((
            // UUID section
            uuid_section(uuid_state.clone()),
            // Handle section
            handle_section(uuid_state.clone()),
        ))
        .style(|s| s.width_full().gap(20.0)),

        // History sections
        h_stack((
            uuid_history_section(uuid_state.clone()),
            handle_history_section(uuid_state),
        ))
        .style(|s| s.width_full().gap(20.0).margin_top(20.0)),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .padding(30.0)
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
                            .padding(8.0)
                            .margin_bottom(12.0)
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

fn uuid_section(state: UuidGenState) -> impl IntoView {
    let uuid = state.generated_uuid;
    let format = state.uuid_format;
    let history = state.uuid_history;
    let status = state.status_message;

    v_stack((
        label(|| "UUID Generator").style(|s| s.font_size(18.0).font_weight(Weight::BOLD).margin_bottom(12.0)),

        // Format selection
        h_stack((
            label(|| "Format:").style(|s| s.margin_right(8.0)),
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
                    "Click 'Generate' to create a UUID".to_string()
                } else {
                    u
                }
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(12.0)
                    .font_size(14.0)
                    .font_family("monospace".to_string())
                    .background(Color::rgb8(250, 250, 250))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
            button("📋")
                .style(|s| s.padding(8.0))
                .action(move || {
                    let u = uuid.get();
                    if !u.is_empty() {
                        copy_to_clipboard(&u);
                        status.set("Copied!".to_string());
                    }
                }),
        ))
        .style(|s| s.width_full().gap(8.0)),

        // Generate button
        button("🎲 Generate UUID")
            .style(|s| {
                s.width_full()
                    .padding_vert(12.0)
                    .margin_top(12.0)
                    .font_size(14.0)
                    .background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(25, 118, 210)))
            })
            .action(move || {
                let new_uuid = generate_uuid(format.get());
                uuid.set(new_uuid.clone());

                // Add to history
                let mut hist = history.get();
                hist.insert(0, new_uuid);
                if hist.len() > 20 {
                    hist.truncate(20);
                }
                history.set(hist);
            }),
    ))
    .style(|s| {
        s.width_pct(50.0)
            .padding(20.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn handle_section(state: UuidGenState) -> impl IntoView {
    let handle = state.generated_handle;
    let history = state.handle_history;
    let status = state.status_message;

    v_stack((
        label(|| "Handle Generator").style(|s| s.font_size(18.0).font_weight(Weight::BOLD).margin_bottom(12.0)),

        // Info text
        label(|| "Generates random u64 handles for BG3 modding")
            .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100)).margin_bottom(12.0)),

        // Generated handle display
        h_stack((
            label(move || {
                let h = handle.get();
                if h.is_empty() {
                    "Click 'Generate' to create a handle".to_string()
                } else {
                    format!("h{}", h)
                }
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(12.0)
                    .font_size(14.0)
                    .font_family("monospace".to_string())
                    .background(Color::rgb8(250, 250, 250))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
            button("📋")
                .style(|s| s.padding(8.0))
                .action(move || {
                    let h = handle.get();
                    if !h.is_empty() {
                        copy_to_clipboard(&format!("h{}", h));
                        status.set("Copied!".to_string());
                    }
                }),
        ))
        .style(|s| s.width_full().gap(8.0)),

        // Generate button
        button("🎲 Generate Handle")
            .style(|s| {
                s.width_full()
                    .padding_vert(12.0)
                    .margin_top(12.0)
                    .font_size(14.0)
                    .background(Color::rgb8(156, 39, 176))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(123, 31, 162)))
            })
            .action(move || {
                let new_handle = generate_handle();
                handle.set(new_handle.clone());

                // Add to history
                let mut hist = history.get();
                hist.insert(0, new_handle);
                if hist.len() > 20 {
                    hist.truncate(20);
                }
                history.set(hist);
            }),
    ))
    .style(|s| {
        s.width_pct(50.0)
            .padding(20.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn uuid_history_section(state: UuidGenState) -> impl IntoView {
    let history = state.uuid_history;
    let status = state.status_message;

    v_stack((
        label(|| "UUID History").style(|s| s.font_size(14.0).font_weight(Weight::BOLD).margin_bottom(8.0)),

        scroll(
            dyn_stack(
                move || history.get(),
                |uuid| uuid.clone(),
                move |uuid| {
                    let status_copy = status;
                    history_row(uuid, status_copy)
                },
            )
            .style(|s| s.width_full()),
        )
        .style(|s| {
            s.width_full()
                .height(200.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_pct(50.0)
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn handle_history_section(state: UuidGenState) -> impl IntoView {
    let history = state.handle_history;
    let status = state.status_message;

    v_stack((
        label(|| "Handle History").style(|s| s.font_size(14.0).font_weight(Weight::BOLD).margin_bottom(8.0)),

        scroll(
            dyn_stack(
                move || history.get(),
                |handle| handle.clone(),
                move |handle| {
                    let status_copy = status;
                    let display = format!("h{}", handle);
                    history_row(display, status_copy)
                },
            )
            .style(|s| s.width_full()),
        )
        .style(|s| {
            s.width_full()
                .height(200.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_pct(50.0)
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn format_button(
    label_text: &'static str,
    btn_format: UuidFormat,
    current_format: RwSignal<UuidFormat>,
) -> impl IntoView {
    button(label_text)
        .style(move |s| {
            let is_selected = current_format.get() == btn_format;
            let s = s
                .padding_horiz(12.0)
                .padding_vert(6.0)
                .border_radius(4.0)
                .font_size(12.0);

            if is_selected {
                s.background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
            } else {
                s.background(Color::rgb8(240, 240, 240))
                    .color(Color::rgb8(60, 60, 60))
                    .hover(|s| s.background(Color::rgb8(220, 220, 220)))
            }
        })
        .action(move || {
            current_format.set(btn_format);
        })
}

fn history_row(value: String, status: RwSignal<String>) -> impl IntoView {
    let value_copy = value.clone();

    h_stack((
        label(move || value.clone())
            .style(|s| {
                s.flex_grow(1.0)
                    .font_family("monospace".to_string())
                    .font_size(12.0)
            }),
        button("📋")
            .style(|s| s.font_size(10.0).padding(4.0))
            .action(move || {
                copy_to_clipboard(&value_copy);
                status.set("Copied!".to_string());
            }),
    ))
    .style(|s| {
        s.width_full()
            .padding(6.0)
            .border_bottom(1.0)
            .border_color(Color::rgb8(240, 240, 240))
            .hover(|s| s.background(Color::rgb8(245, 245, 245)))
    })
}

// ============================================================================
// Generation Functions
// ============================================================================

fn generate_uuid(format: UuidFormat) -> String {
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

fn generate_handle() -> String {
    let handle: u64 = rand::thread_rng().gen();
    handle.to_string()
}

fn copy_to_clipboard(value: &str) {
    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(value.as_bytes());
            }
            let _ = child.wait();
        }
    }
}

fn export_history(state: UuidGenState) {
    let uuids = state.uuid_history.get();
    let handles = state.handle_history.get();

    let json = serde_json::json!({
        "uuids": uuids,
        "handles": handles.iter().map(|h| format!("h{}", h)).collect::<Vec<_>>()
    });

    let dialog = rfd::FileDialog::new()
        .set_title("Export History")
        .add_filter("JSON", &["json"])
        .set_file_name("macpak_ids.json");

    if let Some(path) = dialog.save_file() {
        match fs::write(&path, serde_json::to_string_pretty(&json).unwrap()) {
            Ok(_) => {
                state.status_message.set("Exported successfully!".to_string());
            }
            Err(e) => {
                state.status_message.set(format!("Export failed: {}", e));
            }
        }
    }
}

fn clear_history(state: UuidGenState) {
    state.uuid_history.set(Vec::new());
    state.handle_history.set(Vec::new());
    state.generated_uuid.set(String::new());
    state.generated_handle.set(String::new());
    state.status_message.set("History cleared".to_string());
}
