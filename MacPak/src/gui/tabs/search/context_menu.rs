//! Context menu for search result operations

use std::path::{Path, PathBuf};
use std::sync::Arc;
use floem::action::show_context_menu;
use floem::ext_event::create_ext_action;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;
use floem_reactive::Scope;
use crate::dialog::{LocalizationCache, FlagCache, SpeakerCache, DifficultyClassCache};
use crate::formats::wem::AudioCache;
use maclarian::pak::PakOperations;
use crate::gui::tabs::dialogue::operations::{load_voice_meta, find_voice_files_path};

use crate::gui::state::{DialogueState, DialogSource, EditorTabsState, SearchResult, SearchState};

use super::operations::{copy_to_clipboard, extract_single_result};

/// Show context menu for a search result
pub fn show_search_result_context_menu(
    result: &SearchResult,
    state: SearchState,
    editor_tabs_state: EditorTabsState,
    dialogue_state: DialogueState,
    active_tab: RwSignal<usize>,
) {
    let result_clone = result.clone();
    let result_for_open = result.clone();
    let result_for_dialogue = result.clone();
    let result_for_extract = result.clone();
    let result_for_matches = result.clone();
    let state_for_matches = state.clone();

    let mut menu = Menu::new("");

    // Open in Editor (text files only)
    let file_type = result.file_type.to_lowercase();
    if matches!(file_type.as_str(), "lsx" | "lsf" | "lsj" | "xml" | "json" | "txt") {
        let editor_tabs = editor_tabs_state.clone();
        menu = menu.entry(
            MenuItem::new("Open in Editor")
                .action(move || {
                    open_result_in_editor(&result_for_open, editor_tabs.clone(), active_tab);
                })
        );
    }

    // Open in Dialogue (LSJ files only)
    if file_type == "lsj" {
        let dialogue = dialogue_state.clone();
        let pak_path = result_for_dialogue.pak_path.clone();
        let internal_path = result_for_dialogue.path.clone();
        menu = menu.entry(
            MenuItem::new("Open in Dialogue")
                .action(move || {
                    open_in_dialogue(dialogue.clone(), pak_path.clone(), internal_path.clone(), active_tab);
                })
        );
    }

    // Show All Matches in File (only if there are content matches)
    let has_content_matches = result.match_count.map_or(false, |n| n > 0);
    if has_content_matches {
        menu = menu.entry(
            MenuItem::new("Show All Matches")
                .action(move || {
                    state_for_matches.all_matches_file.set(Some(result_for_matches.clone()));
                    state_for_matches.show_all_matches.set(true);
                })
        );
    }

    menu = menu.separator();

    // Extract File (shows options dialog)
    let state_for_extract = state.clone();
    let internal_path = result_for_extract.path.clone();
    let pak_path = result_for_extract.pak_path.clone();
    menu = menu.entry(
        MenuItem::new("Extract File...")
            .action(move || {
                extract_single_result(state_for_extract.clone(), internal_path.clone(), pak_path.clone());
            })
    );

    // Copy Path
    {
        let path = result_clone.path.clone();
        menu = menu.entry(
            MenuItem::new("Copy Path")
                .action(move || {
                    copy_to_clipboard(&path);
                })
        );
    }

    show_context_menu(menu, None);
}

/// Open a search result in the Editor tab
fn open_result_in_editor(
    result: &SearchResult,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) {
    use std::env::temp_dir;
    use floem::ext_event::create_ext_action;
    use floem_reactive::Scope;
    use maclarian::pak::PakOperations;
    use crate::gui::tabs::load_file_in_tab;

    let result = result.clone();
    let pak_path = result.pak_path.clone();
    let file_path = result.path.clone();

    // Run extraction in background thread
    let send = create_ext_action(Scope::new(), move |extracted_path: Result<std::path::PathBuf, String>| {
        match extracted_path {
            Ok(path) => {
                load_file_in_tab(&path, editor_tabs_state.clone());
                active_tab.set(1); // Switch to Editor tab
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Extraction Failed")
                    .set_description(&e)
                    .show();
            }
        }
    });

    std::thread::spawn(move || {
        // Create temp directory for extracted file
        let temp_base = temp_dir().join("macpak_search_preview");
        let pak_name = pak_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let temp_dir = temp_base.join(&pak_name);

        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            send(Err(format!("Failed to create temp directory: {}", e)));
            return;
        }

        // Extract the single file
        match PakOperations::extract_files_with_progress(
            &pak_path,
            &temp_dir,
            &[file_path.as_str()],
            &|_, _, _| {},
        ) {
            Ok(_) => {
                let extracted = temp_dir.join(&file_path);
                send(Ok(extracted));
            }
            Err(e) => {
                send(Err(format!("Extraction failed: {}", e)));
            }
        }
    });
}

/// Open a dialog file in the Dialogue tab, initializing caches if needed
fn open_in_dialogue(
    state: DialogueState,
    pak_path: PathBuf,
    internal_path: String,
    active_tab: RwSignal<usize>,
) {
    // Check if caches need initialization
    let needs_init = {
        let loca_empty = state.localization_cache.read().map(|c| c.is_empty()).unwrap_or(true);
        let speaker_empty = state.speaker_cache.read().map(|c| !c.is_indexed()).unwrap_or(true);
        loca_empty || speaker_empty
    };

    // Set up pending load - the Dialogue tab will pick this up
    let source = DialogSource::PakFile {
        pak_path: pak_path.clone(),
        internal_path,
    };

    if needs_init {
        // Set pending load info
        state.pending_load.set(Some(source));
        state.pending_caches_ready.set(false);

        // Show loading overlay
        state.flag_index_message.set("Loading metadata from game files...".to_string());
        state.is_building_flag_index.set(true);

        // Switch to Dialogue tab to show overlay
        active_tab.set(7);

        // Init caches in background
        let loca_cache = state.localization_cache.clone();
        let speaker_cache = state.speaker_cache.clone();
        let flag_cache = state.flag_cache.clone();
        let dc_cache = state.difficulty_class_cache.clone();
        let voice_meta_cache = state.voice_meta_cache.clone();
        let audio_cache = state.audio_cache.clone();

        // When done, hide overlay and set ready flag
        let state_for_done = state.clone();
        let send_done = create_ext_action(Scope::new(), move |(voice_loaded, voice_path): (bool, Option<std::path::PathBuf>)| {
            state_for_done.is_building_flag_index.set(false);
            state_for_done.voice_meta_loaded.set(voice_loaded);
            if let Some(path) = voice_path {
                state_for_done.voice_files_path.set(Some(path));
            }
            state_for_done.pending_caches_ready.set(true);
        });

        std::thread::spawn(move || {
            if let Some(data_dir) = pak_path.parent() {
                init_localization_cache(&loca_cache, data_dir);
                init_speaker_cache(&speaker_cache, data_dir);
                init_flag_cache(&flag_cache, data_dir);
                init_dc_cache(&dc_cache, data_dir);

                let voice_count = load_voice_meta(&voice_meta_cache, data_dir);
                let voice_path = find_voice_files_path(data_dir);

                if let Some(ref path) = voice_path {
                    init_audio_cache(&audio_cache, path);
                }

                send_done((voice_count > 0, voice_path));
            } else {
                send_done((false, None));
            }
        });
    } else {
        // Caches already initialized - set pending load and ready immediately
        state.pending_load.set(Some(source));
        state.pending_caches_ready.set(true);
        active_tab.set(7);
    }
}

/// Initialize localization cache from English.pak
fn init_localization_cache(cache: &Arc<std::sync::RwLock<LocalizationCache>>, data_dir: &Path) {
    let localization_dir = data_dir.join("Localization");
    if !localization_dir.exists() {
        return;
    }

    let language_pak = localization_dir.join("English.pak");
    if !language_pak.exists() {
        return;
    }

    let Ok(mut cache) = cache.write() else { return };
    if !cache.is_empty() {
        return;
    }

    let entries = match PakOperations::list(&language_pak) {
        Ok(e) => e,
        Err(_) => return,
    };

    for path in entries.iter().filter(|p| p.to_lowercase().ends_with(".loca")) {
        let _ = cache.load_from_pak(&language_pak, path);
    }
}

/// Initialize speaker cache
fn init_speaker_cache(cache: &Arc<std::sync::RwLock<SpeakerCache>>, data_dir: &Path) {
    let Ok(mut cache) = cache.write() else { return };
    if cache.is_indexed() {
        return;
    }
    cache.configure_from_game_data(data_dir);
    let _ = cache.build_index();
}

/// Initialize flag cache
fn init_flag_cache(cache: &Arc<std::sync::RwLock<FlagCache>>, data_dir: &Path) {
    let Ok(mut cache) = cache.write() else { return };
    if cache.is_indexed() {
        return;
    }
    cache.configure_from_game_data(data_dir);
    let _ = cache.build_index();
}

/// Initialize difficulty class cache
fn init_dc_cache(cache: &Arc<std::sync::RwLock<DifficultyClassCache>>, data_dir: &Path) {
    let Ok(mut cache) = cache.write() else { return };
    if cache.is_indexed() {
        return;
    }
    cache.configure_from_game_data(data_dir);
    let _ = cache.build_index();
}

/// Initialize audio cache for WEM file lookups
fn init_audio_cache(cache: &Arc<std::sync::RwLock<AudioCache>>, voice_path: &Path) {
    let Ok(mut cache) = cache.write() else { return };
    if cache.is_indexed() {
        return;
    }
    if !cache.is_configured() {
        cache.configure(voice_path);
    }
    cache.build_index();
}
