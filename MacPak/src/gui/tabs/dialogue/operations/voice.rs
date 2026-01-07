//! Voice metadata operations - loading voice meta cache for audio playback
//!
//! Optimized for performance:
//! - Direct LSF parsing (no XML conversion overhead)
//! - Parallel processing with rayon

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use rayon::prelude::*;
use MacLarian::pak::PakOperations;
use MacLarian::formats::lsf::{parse_lsf_bytes, LsfDocument};
use crate::gui::state::{VoiceMetaCache, VoiceMetaEntry};

/// Load voice metadata from VoiceMeta.pak or extracted Soundbanks folder
/// Populates the cache with text_handle -> VoiceMetaEntry mappings
pub fn load_voice_meta(
    cache: &Arc<RwLock<VoiceMetaCache>>,
    data_path: &Path,
) -> usize {
    // Try VoiceMeta.pak first (in Localization folder)
    let localization_dir = data_path.join("Localization");
    let voice_meta_pak = localization_dir.join("VoiceMeta.pak");

    if voice_meta_pak.exists() {
        return load_voice_meta_from_pak(cache, &voice_meta_pak);
    }

    // Try extracted VoiceMeta folder
    let voice_meta_dir = localization_dir.join("VoiceMeta");
    if voice_meta_dir.exists() {
        return load_voice_meta_from_folder(cache, &voice_meta_dir);
    }

    tracing::warn!("No VoiceMeta.pak found");
    0
}

/// Load voice metadata from VoiceMeta.pak
fn load_voice_meta_from_pak(cache: &Arc<RwLock<VoiceMetaCache>>, pak_path: &Path) -> usize {
    // Check if already loaded
    {
        let Ok(cache_read) = cache.read() else {
            tracing::error!("Failed to acquire voice meta cache read lock");
            return 0;
        };
        if !cache_read.is_empty() {
            return cache_read.len();
        }
    }

    // List all .lsf files in the PAK
    let entries = match PakOperations::list(pak_path) {
        Ok(e) => e,
        Err(e) => {
            tracing::error!("Failed to list VoiceMeta.pak: {}", e);
            return 0;
        }
    };

    // Filter to Soundbanks .lsf files
    let soundbank_files: Vec<_> = entries
        .iter()
        .filter(|e| e.contains("/Soundbanks/") && e.ends_with(".lsf"))
        .cloned()
        .collect();

    tracing::info!("Found {} soundbank files in VoiceMeta.pak", soundbank_files.len());
    let start = std::time::Instant::now();

    // Process soundbank files in parallel
    let pak_path_owned = pak_path.to_path_buf();
    let all_entries: Vec<HashMap<String, VoiceMetaEntry>> = soundbank_files
        .par_iter()
        .filter_map(|internal_path| {
            match load_soundbank_from_pak(&pak_path_owned, internal_path) {
                Ok(entries) => {
                    if entries.is_empty() {
                        tracing::debug!("Soundbank {} yielded 0 entries", internal_path);
                    }
                    Some(entries)
                }
                Err(e) => {
                    tracing::warn!("Failed to load soundbank {}: {}", internal_path, e);
                    None
                }
            }
        })
        .collect();

    tracing::debug!("Processed {} soundbank files, got {} result sets",
        soundbank_files.len(), all_entries.len());

    // Merge results into cache
    let Ok(mut cache_write) = cache.write() else {
        tracing::error!("Failed to acquire voice meta cache write lock");
        return 0;
    };

    let mut total_count = 0;
    for entries in all_entries {
        for (handle, entry) in entries {
            cache_write.insert(handle, entry);
            total_count += 1;
        }
    }

    let elapsed = start.elapsed();
    tracing::info!("Loaded {} voice meta entries in {:.2}s", total_count, elapsed.as_secs_f64());
    total_count
}

/// Load voice metadata from extracted VoiceMeta folder
fn load_voice_meta_from_folder(cache: &Arc<RwLock<VoiceMetaCache>>, folder: &Path) -> usize {
    // Check if already loaded
    {
        let Ok(cache_read) = cache.read() else {
            tracing::error!("Failed to acquire voice meta cache read lock");
            return 0;
        };
        if !cache_read.is_empty() {
            return cache_read.len();
        }
    }

    // Find all Soundbanks .lsf files recursively
    let soundbank_files = find_soundbank_files(folder);
    tracing::info!("Found {} soundbank files in extracted folder", soundbank_files.len());
    let start = std::time::Instant::now();

    // Process soundbank files in parallel
    let all_entries: Vec<HashMap<String, VoiceMetaEntry>> = soundbank_files
        .par_iter()
        .filter_map(|file_path| {
            load_soundbank_from_file(file_path).ok()
        })
        .collect();

    // Merge results into cache
    let Ok(mut cache_write) = cache.write() else {
        tracing::error!("Failed to acquire voice meta cache write lock");
        return 0;
    };

    let mut total_count = 0;
    for entries in all_entries {
        for (handle, entry) in entries {
            cache_write.insert(handle, entry);
            total_count += 1;
        }
    }

    let elapsed = start.elapsed();
    tracing::info!("Loaded {} voice meta entries from folder in {:.2}s", total_count, elapsed.as_secs_f64());
    total_count
}

/// Find all Soundbanks .lsf files in a folder recursively
fn find_soundbank_files(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    find_soundbank_files_recursive(folder, &mut files);
    files
}

fn find_soundbank_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            find_soundbank_files_recursive(&path, files);
        } else if path.extension().map(|e| e == "lsf").unwrap_or(false) {
            // Check if it's in a Soundbanks folder
            if path.to_string_lossy().contains("Soundbanks") {
                files.push(path);
            }
        }
    }
}

/// Load a soundbank file from within a PAK and extract voice meta entries
/// Uses direct LSF parsing (no XML conversion) for performance
fn load_soundbank_from_pak(pak_path: &Path, internal_path: &str) -> Result<HashMap<String, VoiceMetaEntry>, String> {
    // Read the LSF file from PAK
    let lsf_data = PakOperations::read_file_bytes(pak_path, internal_path)
        .map_err(|e| format!("Failed to read from PAK: {}", e))?;

    // Parse LSF directly (no XML conversion)
    let lsf_doc = parse_lsf_bytes(&lsf_data)
        .map_err(|e| format!("Failed to parse LSF: {}", e))?;

    // Extract voice meta entries directly from LSF structure
    parse_soundbank_lsf(&lsf_doc)
}

/// Load a soundbank file from disk and extract voice meta entries
/// Uses direct LSF parsing (no XML conversion) for performance
fn load_soundbank_from_file(file_path: &Path) -> Result<HashMap<String, VoiceMetaEntry>, String> {
    // Read the LSF file
    let lsf_data = std::fs::read(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse LSF directly (no XML conversion)
    let lsf_doc = parse_lsf_bytes(&lsf_data)
        .map_err(|e| format!("Failed to parse LSF: {}", e))?;

    // Extract voice meta entries directly from LSF structure
    parse_soundbank_lsf(&lsf_doc)
}

/// Parse soundbank LSF document directly (no XML conversion)
///
/// This is much faster than converting to XML and re-parsing.
/// Navigates the LSF node tree directly to extract voice metadata.
fn parse_soundbank_lsf(doc: &LsfDocument) -> Result<HashMap<String, VoiceMetaEntry>, String> {
    let mut entries = HashMap::new();

    // Find all VoiceSpeakerMetaData nodes by scanning all nodes
    for (node_idx, _) in doc.nodes.iter().enumerate() {
        if doc.node_name(node_idx) != Some("VoiceSpeakerMetaData") {
            continue;
        }

        // Get speaker UUID from MapKey attribute
        let speaker_uuid = doc.get_fixed_string_attr(node_idx, "MapKey")
            .unwrap_or_default();

        // Find MapValue child (contains VoiceTextMetaData children)
        for map_value_idx in doc.find_children_by_name(node_idx, "MapValue") {
            // Find VoiceTextMetaData children
            for text_meta_idx in doc.find_children_by_name(map_value_idx, "VoiceTextMetaData") {
                // Get text handle from MapKey attribute
                let text_handle = match doc.get_fixed_string_attr(text_meta_idx, "MapKey") {
                    Some(h) if !h.is_empty() => h,
                    _ => continue,
                };

                // Find the inner MapValue with Source, Length, Codec
                for inner_map_value_idx in doc.find_children_by_name(text_meta_idx, "MapValue") {
                    let source_file = doc.get_fixed_string_attr(inner_map_value_idx, "Source")
                        .unwrap_or_default();
                    let length = doc.get_float_attr(inner_map_value_idx, "Length")
                        .unwrap_or(0.0);
                    let codec = doc.get_fixed_string_attr(inner_map_value_idx, "Codec")
                        .unwrap_or_default();

                    if !source_file.is_empty() {
                        entries.insert(text_handle.clone(), VoiceMetaEntry {
                            source_file,
                            length,
                            codec,
                            speaker_uuid: speaker_uuid.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(entries)
}

/// Find the path to voice files (extracted folder or Voice.pak)
///
/// Checks for extracted Voice folder first (faster), then falls back to Voice.pak.
pub fn find_voice_files_path(data_path: &Path) -> Option<PathBuf> {
    let localization_dir = data_path.join("Localization");

    // Check for extracted Voice folder first (faster to index and load)
    let voice_dir = localization_dir.join("Voice");
    if voice_dir.exists() {
        tracing::info!("Found extracted Voice folder: {:?}", voice_dir);
        return Some(voice_dir);
    }

    // Check for Voice.pak (supports on-demand extraction)
    let voice_pak = localization_dir.join("Voice.pak");
    if voice_pak.exists() {
        tracing::info!("Found Voice.pak: {:?}", voice_pak);
        return Some(voice_pak);
    }

    tracing::warn!("No voice files found in {:?}", data_path);
    None
}
