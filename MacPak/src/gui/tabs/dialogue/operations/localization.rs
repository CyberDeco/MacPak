//! Localization operations - loading and resolving localized text
//!
//! These functions are used for on-demand localization loading and may be
//! called when switching languages or loading dialogs that need text resolution.
//!
//! Optimized for performance with parallel .loca file parsing.

use std::path::Path;
use floem::reactive::SignalUpdate;
use rayon::prelude::*;
use MacLarian::formats::dialog::{LocalizationCache, get_available_languages};
use MacLarian::formats::loca::parse_loca_bytes;
use MacLarian::pak::PakOperations;
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
            match load_localization_from_pak(&mut cache, &language_pak) {
                Ok(count) => {
                    println!("[Dialogue] Loaded {} localization strings", count);
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

/// Parsed localization entry for merging into cache
struct ParsedEntry {
    key: String,
    text: String,
    version: u16,
}

/// Load localization entries from a language PAK file (parallel parsing)
#[allow(dead_code)]
fn load_localization_from_pak(cache: &mut LocalizationCache, pak_path: &Path) -> Result<usize, String> {
    // List all .loca files in the PAK
    let entries = PakOperations::list(pak_path)
        .map_err(|e| format!("Failed to list PAK: {e}"))?;

    let loca_files: Vec<_> = entries
        .iter()
        .filter(|e| e.to_lowercase().ends_with(".loca"))
        .cloned()
        .collect();

    println!("[Dialogue] Found {} .loca files in language PAK", loca_files.len());
    let start = std::time::Instant::now();

    // Batch read all .loca files from PAK
    let file_data = PakOperations::read_files_bytes(pak_path, &loca_files)
        .map_err(|e| format!("Failed to batch read files: {e}"))?;

    // Parse .loca files in parallel
    let parsed_results: Vec<Result<Vec<ParsedEntry>, String>> = file_data
        .par_iter()
        .map(|(path, data)| {
            parse_loca_bytes(data)
                .map(|resource| {
                    resource
                        .entries
                        .into_iter()
                        .map(|e| ParsedEntry {
                            key: e.key,
                            text: e.text,
                            version: e.version,
                        })
                        .collect()
                })
                .map_err(|e| format!("Failed to parse {path}: {e}"))
        })
        .collect();

    // Merge results sequentially into cache
    let mut total_count = 0;
    for result in parsed_results {
        match result {
            Ok(entries) => {
                for entry in entries {
                    cache.insert(entry.key, entry.text, entry.version);
                    total_count += 1;
                }
            }
            Err(e) => println!("[Dialogue] {e}"),
        }
    }

    let elapsed = start.elapsed();
    println!(
        "[Dialogue] Parsed {} localization entries in {:.2}s",
        total_count,
        elapsed.as_secs_f64()
    );

    Ok(total_count)
}
