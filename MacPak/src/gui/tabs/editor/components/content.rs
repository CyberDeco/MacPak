//! Main editor content component

use std::time::Duration;

use floem::action::exec_after;
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::prelude::*;
use floem::views::editor::command::CommandExecuted;
use floem::views::editor::keypress::{default_key_handler, key::KeyInput, press::KeyPress};
use floem::views::text_editor_keys;

use crate::gui::state::{EditorTab, EditorTabsState};

use super::super::operations::{open_file_dialog, save_file, save_file_as_dialog};
use super::super::syntax::SyntaxStyling;

pub fn editor_content(
    tab: EditorTab,
    tabs_state: EditorTabsState,
    show_line_numbers: RwSignal<bool>,
) -> impl IntoView {
    let content = tab.content;
    let live_content = tab.live_content;
    let modified = tab.modified;
    let file_format = tab.file_format;
    let goto_offset = tab.goto_offset;
    let search_visible = tab.search_visible;
    let converted_from_lsf = tab.converted_from_lsf;
    let tab_for_save = tab.clone();
    let tabs_state_for_open = tabs_state.clone();

    // Recreate editor only when format changes (for syntax highlighting)
    // Width/resize and line numbers are handled reactively
    // Loading state is now handled by the overlay in mod.rs
    dyn_container(
        move || file_format.get(),
        move |format| {
            let show_lines = show_line_numbers;
            // Use get_untracked to avoid creating a reactive subscription
            let text = content.get_untracked();
            let state_change = modified;
            // Create syntax highlighting based on file format
            let styling = SyntaxStyling::new(&text, &format);

            // Clone tab and state for the key handler
            let tab_for_keys = tab_for_save.clone();
            let tabs_state_for_keys = tabs_state_for_open.clone();

            // Custom key handler that intercepts shortcuts before the default handler
            let key_handler =
                move |editor_sig: floem::prelude::RwSignal<floem::views::editor::Editor>,
                      keypress: &KeyPress,
                      mods: Modifiers| {
                    // Check for CMD/Ctrl modifier
                    let is_cmd_or_ctrl = mods.meta() || mods.control();

                    // Determine if this key might modify content
                    let might_edit = if is_cmd_or_ctrl {
                        // CMD+Z, CMD+X, CMD+V modify content
                        matches!(&keypress.key, KeyInput::Keyboard(Key::Character(c), _)
                        if c.as_str().eq_ignore_ascii_case("z")
                        || c.as_str().eq_ignore_ascii_case("x")
                        || c.as_str().eq_ignore_ascii_case("v"))
                    } else {
                        // Non-modifier keys that edit content
                        matches!(
                            &keypress.key,
                            KeyInput::Keyboard(Key::Character(_), _)
                                | KeyInput::Keyboard(
                                    Key::Named(
                                        NamedKey::Backspace
                                            | NamedKey::Delete
                                            | NamedKey::Enter
                                            | NamedKey::Tab
                                    ),
                                    _
                                )
                        )
                    };

                    if is_cmd_or_ctrl {
                        if let KeyInput::Keyboard(Key::Character(c), _) = &keypress.key {
                            // CMD+F - Find
                            if c.as_str().eq_ignore_ascii_case("f") {
                                search_visible.set(!search_visible.get());
                                return CommandExecuted::Yes;
                            }
                            // CMD+S - Save (sync content from editor first)
                            if c.as_str().eq_ignore_ascii_case("s") {
                                // Sync editor content to live_content before saving
                                let new_text = editor_sig.get_untracked().doc().text().to_string();
                                live_content.set(new_text);

                                if modified.get() {
                                    if converted_from_lsf.get() {
                                        let tab_clone = tab_for_keys.clone();
                                        exec_after(Duration::from_millis(50), move |_| {
                                            save_file_as_dialog(tab_clone);
                                        });
                                    } else {
                                        save_file(tab_for_keys.clone());
                                    }
                                }
                                return CommandExecuted::Yes;
                            }
                            // CMD+O - Open
                            if c.as_str().eq_ignore_ascii_case("o") {
                                let tabs_clone = tabs_state_for_keys.clone();
                                exec_after(Duration::from_millis(50), move |_| {
                                    open_file_dialog(tabs_clone);
                                });
                                return CommandExecuted::Yes;
                            }
                        }
                    }

                    // Process the key through the default handler
                    let result = default_key_handler(editor_sig)(keypress, mods);

                    // If this was an editing key, sync to live_content and mark as modified
                    // live_content is NOT watched by dyn_container, so this won't cause cascades
                    if might_edit {
                        let new_text = editor_sig.get_untracked().doc().text().to_string();
                        live_content.set(new_text);
                        state_change.set(true);
                    }

                    result
                };

            text_editor_keys(text, key_handler)
                .styling(styling)
                .editor_style(move |s| s.hide_gutter(!show_lines.get()))
                .style(move |s| {
                    let s = s.width_full().height_full();
                    if show_lines.get() {
                        s
                    } else {
                        s.padding_left(12.0)
                    }
                })
                .placeholder("Open a file to start editing...")
                .with_editor(move |editor| {
                    let cursor = editor.cursor;

                    // NOTE: Effects disabled due to leak in dyn_container
                    // floem's create_effect doesn't get cleaned up when views are destroyed

                    // Workaround for floem text_editor not re-wrapping on width expansion.
                    // DISABLED - causes effect leak
                    let _ = (editor.parent_size, editor.viewport);

                    // Jump to offset when goto_offset changes
                    // DISABLED - causes effect leak
                    // TODO: Find alternative approach for search navigation
                    let _ = (cursor, goto_offset);

                    // TODO: Content sync disabled due to effect leak in dyn_container
                    // The floem create_effect doesn't get cleaned up when the view is destroyed,
                    // causing multiple effects to accumulate and cascade.
                    // For now, skip auto-sync. The editor maintains its own state.
                    // Content is synced on-demand (e.g., when saving).
                    let _ = (content, state_change); // Suppress unused warnings
                })
                .style(|s| s.size_full().flex_grow(1.0))
                .into_any()
        },
    )
    .style(|s| s.size_full().flex_grow(1.0))
}
