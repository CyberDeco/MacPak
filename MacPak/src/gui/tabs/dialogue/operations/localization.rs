//! Localization operations - loading and resolving localized text
//!
//! These functions are used for on-demand localization loading and may be
//! called when switching languages or loading dialogs that need text resolution.
//!
//! Uses parallel .loca file parsing for optimal performance.

use std::path::Path;
use floem::reactive::SignalUpdate;
use crate::dialog::{get_available_languages, load_localization_from_pak_parallel};
use crate::gui::state::DialogueState;

/// Try to load localization from a game data folder
///
/// Looks for `Localization/<Language>.pak` relative to the given PAK's parent directory
/// Note: This runs on a background thread, so we use the default language "English"
#[allow(dead_code)]
pub fn try_load_localization(state: &DialogueState, pak_path: &Path) {
    // Try to find the Localization folder from the PAK path
    // Typical structure: <GameData>/Gustav.pak, <GameData>/Localization/English.pak
    let Some(data_dir) = pak_path.parent() else {
        println!("[Dialogue] Could not find parent directory of PAK");
        return;
    };

    let localization_dir = data_dir.join("Localization");
    if !localization_dir.exists() {
        println!("[Dialogue] Localization directory not found at {:?}", localization_dir);
        return;
    }

    // Update available languages
    let languages = get_available_languages(data_dir);
    if !languages.is_empty() {
        // Note: .set() works from background threads, .get() does not
        state.available_languages.set(languages.clone());
        println!("[Dialogue] Found languages: {:?}", languages);
    }

    // Load English localization (default language)
    // Note: Can't use state.language.get() from background thread
    let language = "English";
    let language_pak = localization_dir.join(format!("{}.pak", language));

    if !language_pak.exists() {
        println!("[Dialogue] Language PAK not found: {:?}", language_pak);
        return;
    }

    println!("[Dialogue] Loading localization from {:?}", language_pak);

    // Get mutable access to the cache
    let cache = state.localization_cache.clone();
    if let Ok(mut cache) = cache.write() {
        // Only load if not already loaded
        if cache.is_empty() {
            let start = std::time::Instant::now();
            // Use maclarian's parallel localization loader
            match load_localization_from_pak_parallel(&language_pak, &mut cache) {
                Ok(count) => {
                    let elapsed = start.elapsed();
                    println!(
                        "[Dialogue] Loaded {} localization strings in {:.2}s",
                        count,
                        elapsed.as_secs_f64()
                    );
                    state.localization_loaded.set(true);
                    state.status_message.set(format!("Loaded {} localization strings", count));
                }
                Err(e) => {
                    println!("[Dialogue] Failed to load localization: {}", e);
                }
            }
        } else {
            println!("[Dialogue] Localization cache already has {} entries", cache.len());
        }
    }
}
