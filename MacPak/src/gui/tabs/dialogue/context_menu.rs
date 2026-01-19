//! Context menu for dialogue node operations

use std::cell::RefCell;
use std::io::Write as IoWrite;
use std::process::Command;

use floem::action::show_context_menu;
use floem::menu::{Menu, MenuItem};
use floem::reactive::SignalUpdate;

use crate::gui::state::{DialogueState, DisplayNode};
use super::operations::{AudioPlayer, play_node_audio};

// Thread-local audio player (OutputStream is !Send+!Sync, must stay on main thread)
thread_local! {
    static AUDIO_PLAYER: RefCell<Option<AudioPlayer>> = const { RefCell::new(None) };
}

/// Get or initialize the thread-local audio player
fn with_audio_player<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&AudioPlayer) -> R,
{
    AUDIO_PLAYER.with(|cell| {
        let mut player_opt = cell.borrow_mut();

        // Initialize if needed
        if player_opt.is_none() {
            match AudioPlayer::new() {
                Ok(player) => {
                    *player_opt = Some(player);
                }
                Err(e) => {
                    tracing::error!("Failed to initialize audio player: {}", e);
                    return None;
                }
            }
        }

        player_opt.as_ref().map(f)
    })
}

/// Show context menu for a dialogue node
pub fn show_node_context_menu(
    node: &DisplayNode,
    state: DialogueState,
) {
    let node_uuid = node.uuid.clone();
    // Use text_handle if available, otherwise fall back to jump_target_handle for Jump/Alias nodes
    let text_handle = node.text_handle.clone()
        .or_else(|| node.jump_target_handle.clone());
    let has_audio = text_handle.as_ref().is_some_and(|h| state.has_audio(h));

    let mut menu = Menu::new("");

    // Play Audio option (only if node has audio)
    if has_audio {
        let state_play = state.clone();
        let uuid_play = node_uuid.clone();
        let handle_play = text_handle.clone();

        menu = menu.entry(
            MenuItem::new("Play Audio")
                .action(move || {
                    if let Some(ref handle) = handle_play {
                        let result = with_audio_player(|player| {
                            play_node_audio(player, &state_play, handle, &uuid_play)
                        });

                        match result {
                            Some(Ok(())) => {}
                            Some(Err(e)) => {
                                state_play.error_message.set(Some(format!("Audio playback failed: {}", e)));
                            }
                            None => {
                                state_play.error_message.set(Some("Audio player not available".to_string()));
                            }
                        }
                    }
                })
        );

        // Stop Audio option
        let state_stop = state.clone();
        menu = menu.entry(
            MenuItem::new("Stop Audio")
                .action(move || {
                    with_audio_player(|player| {
                        player.stop();
                    });
                    state_stop.playing_audio_node.set(None);
                })
        );

        menu = menu.separator();
    }

    // Copy UUID
    {
        let uuid = node_uuid.clone();
        menu = menu.entry(
            MenuItem::new("Copy UUID")
                .action(move || {
                    copy_to_clipboard(&uuid);
                })
        );
    }

    // Copy Text Handle (if available)
    if let Some(ref handle) = text_handle {
        let handle_copy = handle.clone();
        menu = menu.entry(
            MenuItem::new("Copy Text Handle")
                .action(move || {
                    copy_to_clipboard(&handle_copy);
                })
        );
    }

    // Copy Node Text (if available)
    if !node.text.is_empty() {
        let text_copy = node.text.clone();
        menu = menu.entry(
            MenuItem::new("Copy Text")
                .action(move || {
                    copy_to_clipboard(&text_copy);
                })
        );
    }

    show_context_menu(menu, None);
}

/// Copy text to system clipboard (macOS)
fn copy_to_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
    }
}
