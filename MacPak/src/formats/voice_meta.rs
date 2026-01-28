//! Voice metadata loading from VoiceMeta.pak
//!
//! Optimized for performance with:
//! - Direct LSF parsing (no XML conversion overhead)
//! - Parallel processing with rayon

use maclarian::formats::lsf::{LsfDocument, parse_lsf_bytes};
use maclarian::pak::PakOperations;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Voice metadata entry for audio playback
#[derive(Clone, Debug)]
pub struct VoiceMetaEntry {
    /// Source .wem filename (e.g., "v518fab8f2d1d46c8..._h35f3e7db....wem")
    pub source_file: String,
    /// Audio length in seconds
    pub length: f32,
    /// Audio codec (typically "VORBIS")
    pub codec: String,
    /// Speaker UUID this audio belongs to
    pub speaker_uuid: String,
}

/// Cache mapping text handles to voice metadata
pub type VoiceMetaCache = HashMap<String, VoiceMetaEntry>;

/// Load voice metadata from VoiceMeta.pak (parallel processing)
///
/// Returns a map of text handle -> `VoiceMetaEntry`
///
/// # Errors
/// Returns an error if the PAK file cannot be read or parsed.
pub fn load_voice_meta_from_pak(pak_path: &Path) -> Result<VoiceMetaCache, String> {
    // List all .lsf files in the PAK
    let entries =
        PakOperations::list(pak_path).map_err(|e| format!("Failed to list VoiceMeta.pak: {e}"))?;

    // Filter to Soundbanks .lsf files
    let soundbank_files: Vec<_> = entries
        .iter()
        .filter(|e| e.contains("/Soundbanks/") && e.to_lowercase().ends_with(".lsf"))
        .cloned()
        .collect();

    tracing::info!(
        "Found {} soundbank files in VoiceMeta.pak",
        soundbank_files.len()
    );
    let start = std::time::Instant::now();

    // Process soundbank files in parallel
    let pak_path_owned = pak_path.to_path_buf();
    let all_entries: Vec<HashMap<String, VoiceMetaEntry>> = soundbank_files
        .par_iter()
        .filter_map(
            |internal_path| match load_soundbank_from_pak(&pak_path_owned, internal_path) {
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
            },
        )
        .collect();

    tracing::debug!(
        "Processed {} soundbank files, got {} result sets",
        soundbank_files.len(),
        all_entries.len()
    );

    // Merge results
    let mut cache = HashMap::new();
    for entries in all_entries {
        for (handle, entry) in entries {
            cache.insert(handle, entry);
        }
    }

    let elapsed = start.elapsed();
    tracing::info!(
        "Loaded {} voice meta entries in {:.2}s",
        cache.len(),
        elapsed.as_secs_f64()
    );
    Ok(cache)
}

/// Load voice metadata from extracted `VoiceMeta` folder (parallel processing)
///
/// Returns a map of text handle -> `VoiceMetaEntry`
///
/// # Errors
/// Returns an error if the folder cannot be read or files cannot be parsed.
pub fn load_voice_meta_from_folder(folder: &Path) -> Result<VoiceMetaCache, String> {
    // Find all Soundbanks .lsf files recursively
    let soundbank_files = find_soundbank_files(folder);
    tracing::info!(
        "Found {} soundbank files in extracted folder",
        soundbank_files.len()
    );
    let start = std::time::Instant::now();

    // Process soundbank files in parallel
    let all_entries: Vec<HashMap<String, VoiceMetaEntry>> = soundbank_files
        .par_iter()
        .filter_map(|file_path| load_soundbank_from_file(file_path).ok())
        .collect();

    // Merge results
    let mut cache = HashMap::new();
    for entries in all_entries {
        for (handle, entry) in entries {
            cache.insert(handle, entry);
        }
    }

    let elapsed = start.elapsed();
    tracing::info!(
        "Loaded {} voice meta entries from folder in {:.2}s",
        cache.len(),
        elapsed.as_secs_f64()
    );
    Ok(cache)
}

/// Find all Soundbanks .lsf files in a folder recursively
fn find_soundbank_files(folder: &Path) -> Vec<PathBuf> {
    use walkdir::WalkDir;

    WalkDir::new(folder)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let path = e.path();
            path.extension().is_some_and(|ext| ext == "lsf")
                && path.to_string_lossy().contains("Soundbanks")
        })
        .map(walkdir::DirEntry::into_path)
        .collect()
}

/// Load a soundbank file from within a PAK and extract voice meta entries
fn load_soundbank_from_pak(
    pak_path: &Path,
    internal_path: &str,
) -> Result<HashMap<String, VoiceMetaEntry>, String> {
    let lsf_data = PakOperations::read_file_bytes(pak_path, internal_path)
        .map_err(|e| format!("Failed to read from PAK: {e}"))?;

    let lsf_doc = parse_lsf_bytes(&lsf_data).map_err(|e| format!("Failed to parse LSF: {e}"))?;

    parse_soundbank_lsf(&lsf_doc)
}

/// Load a soundbank file from disk and extract voice meta entries
fn load_soundbank_from_file(file_path: &Path) -> Result<HashMap<String, VoiceMetaEntry>, String> {
    let lsf_data = std::fs::read(file_path).map_err(|e| format!("Failed to read file: {e}"))?;

    let lsf_doc = parse_lsf_bytes(&lsf_data).map_err(|e| format!("Failed to parse LSF: {e}"))?;

    parse_soundbank_lsf(&lsf_doc)
}

/// Parse soundbank LSF document directly (no XML conversion)
///
/// Navigates the LSF node tree to extract voice metadata.
fn parse_soundbank_lsf(doc: &LsfDocument) -> Result<HashMap<String, VoiceMetaEntry>, String> {
    let mut entries = HashMap::new();

    // Find all VoiceSpeakerMetaData nodes
    for (node_idx, _) in doc.nodes.iter().enumerate() {
        if doc.node_name(node_idx) != Some("VoiceSpeakerMetaData") {
            continue;
        }

        // Get speaker UUID from MapKey attribute
        let speaker_uuid = doc
            .get_fixed_string_attr(node_idx, "MapKey")
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
                    let source_file = doc
                        .get_fixed_string_attr(inner_map_value_idx, "Source")
                        .unwrap_or_default();
                    let length = doc
                        .get_float_attr(inner_map_value_idx, "Length")
                        .unwrap_or(0.0);
                    let codec = doc
                        .get_fixed_string_attr(inner_map_value_idx, "Codec")
                        .unwrap_or_default();

                    if !source_file.is_empty() {
                        entries.insert(
                            text_handle.clone(),
                            VoiceMetaEntry {
                                source_file,
                                length,
                                codec,
                                speaker_uuid: speaker_uuid.clone(),
                            },
                        );
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

/// Find the path to VoiceMeta.pak or extracted `VoiceMeta` folder
#[must_use]
pub fn find_voice_meta_path(data_path: &Path) -> Option<PathBuf> {
    let localization_dir = data_path.join("Localization");

    // Try VoiceMeta.pak first
    let voice_meta_pak = localization_dir.join("VoiceMeta.pak");
    if voice_meta_pak.exists() {
        return Some(voice_meta_pak);
    }

    // Try extracted VoiceMeta folder
    let voice_meta_dir = localization_dir.join("VoiceMeta");
    if voice_meta_dir.exists() {
        return Some(voice_meta_dir);
    }

    None
}
