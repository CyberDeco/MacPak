//! Speaker cache for resolving speaker UUIDs to DisplayName handles
//!
//! Dynamically loads character templates from PAK files to map
//! speaker UUIDs to their DisplayName localization handles.

use crate::formats::lsf::parse_lsf_bytes;
use crate::formats::lsx::parse_lsx;
use crate::formats::common::{extract_value, extract_translated_string};
use crate::pak::PakOperations;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Cache for speaker UUID → DisplayName handle resolution
///
/// Uses pre-indexing - builds a complete index when `build_index()` is called,
/// then provides O(1) lookups. Similar to FlagCache.
#[derive(Debug, Clone, Default)]
pub struct SpeakerCache {
    /// Speaker DisplayName handles indexed by UUID (pre-loaded)
    handles: HashMap<String, String>,
    /// Whether the index has been built
    indexed: bool,
    /// PAK files to index from
    pak_paths: Vec<PathBuf>,
}

impl SpeakerCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            indexed: false,
            pak_paths: Vec::new(),
        }
    }

    /// Get the number of cached speakers
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty() && self.pak_paths.is_empty()
    }

    /// Check if PAK sources are configured
    pub fn has_sources(&self) -> bool {
        !self.pak_paths.is_empty()
    }

    /// Check if the index has been built
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Clear all cached data
    pub fn clear(&mut self) {
        self.handles.clear();
        self.indexed = false;
        self.pak_paths.clear();
    }

    /// Add a PAK file as a source for speaker lookups
    pub fn add_pak_source<P: AsRef<Path>>(&mut self, pak_path: P) {
        let path = pak_path.as_ref().to_path_buf();
        if !self.pak_paths.contains(&path) {
            self.pak_paths.push(path);
        }
    }

    /// Look up a DisplayName handle by speaker UUID (O(1) after indexing)
    pub fn get_handle(&self, uuid: &str) -> Option<&str> {
        self.handles.get(uuid).map(|s| s.as_str())
    }

    /// Insert a handle directly (for testing or manual additions)
    pub fn insert(&mut self, uuid: String, handle: String) {
        self.handles.insert(uuid, handle);
    }

    /// Build the speaker handle index from all configured PAK sources.
    ///
    /// Scans RootTemplates and Level character files for MapKey → DisplayName mappings.
    /// Returns the number of speakers indexed.
    pub fn build_index(&mut self) -> Result<usize, SpeakerCacheError> {
        if self.indexed {
            return Ok(self.handles.len());
        }

        // Add hardcoded speaker mappings for special cases not in template files
        // Player character uses a special UUID that's not defined in RootTemplates/Origins
        // Using __DIRECT__: prefix to signal that this is a direct name, not a loca handle
        self.handles.insert(
            "e0d1ff71-04a8-4340-ae64-9684d846eb83".to_string(),
            "__DIRECT__:Player".to_string(),
        );

        let mut total_count = 1; // Count the hardcoded entry

        for pak_path in self.pak_paths.clone() {
            // List all files in the PAK
            let all_files = PakOperations::list(&pak_path)
                .map_err(|e| SpeakerCacheError::PakError(e.to_string()))?;

            // Filter for character/template files
            // Look for: RootTemplates, Characters folders, and Origins
            let template_files: Vec<String> = all_files
                .into_iter()
                .filter(|path| {
                    let lower = path.to_lowercase();
                    // RootTemplates contain character definitions (.lsf binary)
                    (lower.contains("roottemplates") && lower.ends_with(".lsf"))
                        // Level Characters folders contain placed characters
                        || (lower.contains("/characters/") && lower.contains("_merged.lsf"))
                        // Origins contain companion/origin character definitions (.lsx XML)
                        || (lower.contains("/origins/") && (lower.ends_with(".lsf") || lower.ends_with(".lsx")))
                })
                .collect();

            if template_files.is_empty() {
                continue;
            }

            tracing::debug!("Found {} template files in {}", template_files.len(), pak_path.display());

            // Batch read all template files
            let file_data = PakOperations::read_files_bytes(&pak_path, &template_files)
                .map_err(|e| SpeakerCacheError::PakError(e.to_string()))?;

            // Parse template files in parallel and collect results
            // Handle both LSF (binary) and LSX (XML) formats
            let parsed_speakers: Vec<Vec<(String, String)>> = file_data
                .par_iter()
                .map(|(path, data)| {
                    let lower_path = path.to_lowercase();
                    if lower_path.ends_with(".lsx") {
                        Self::extract_speakers_from_lsx(data)
                    } else {
                        Self::extract_speakers_from_lsf(data)
                    }
                })
                .collect();

            // Merge results sequentially
            for speakers in parsed_speakers {
                for (uuid, handle) in speakers {
                    self.handles.insert(uuid, handle);
                    total_count += 1;
                }
            }
        }

        self.indexed = true;
        tracing::info!("Speaker index built: {} speakers", total_count);
        Ok(total_count)
    }

    /// Extract UUID and DisplayName (handle) pairs from LSF bytes
    /// Looks for both MapKey (RootTemplates) and GlobalTemplate (Origins) as UUID sources
    fn extract_speakers_from_lsf(data: &[u8]) -> Vec<(String, String)> {
        let mut results = Vec::new();

        let Ok(doc) = parse_lsf_bytes(data) else {
            return results;
        };

        // Scan all nodes for UUID + DisplayName attribute pairs
        // UUID can come from MapKey (RootTemplates) or GlobalTemplate (Origins)
        for node in doc.nodes.iter() {

            let mut map_key: Option<String> = None;
            let mut global_template: Option<String> = None;
            let mut display_name_handle: Option<String> = None;

            if node.first_attribute_index >= 0 {
                let mut attr_idx = node.first_attribute_index as usize;
                loop {
                    if attr_idx >= doc.attributes.len() {
                        break;
                    }

                    let attr = &doc.attributes[attr_idx];
                    let attr_name = doc
                        .get_name(attr.name_index_outer, attr.name_index_inner)
                        .unwrap_or("");
                    let type_id = attr.type_info & 0x3F;
                    let value_length = (attr.type_info >> 6) as usize;

                    match attr_name {
                        "MapKey" => {
                            if let Ok(val) = extract_value(&doc.values, attr.offset, value_length, type_id) {
                                if !val.is_empty() && val.contains('-') {
                                    map_key = Some(val);
                                }
                            }
                        }
                        "GlobalTemplate" => {
                            // GlobalTemplate in Origins files maps speaker UUIDs to characters
                            if let Ok(val) = extract_value(&doc.values, attr.offset, value_length, type_id) {
                                if !val.is_empty() && val.contains('-') {
                                    global_template = Some(val);
                                }
                            }
                        }
                        "DisplayName" => {
                            // DisplayName is a TranslatedString (type 28) - use special extraction
                            if type_id == 28 {
                                if let Ok((handle, _version, _value)) = extract_translated_string(&doc.values, attr.offset, value_length) {
                                    // Accept any non-empty handle - don't require 'h' prefix
                                    // as handle formats can vary
                                    if !handle.is_empty() {
                                        display_name_handle = Some(handle);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    if attr.next_index < 0 {
                        break;
                    }
                    attr_idx = attr.next_index as usize;
                }
            }

            // Add entries for both MapKey and GlobalTemplate if they have DisplayName
            if let Some(ref handle) = display_name_handle {
                if let Some(uuid) = map_key {
                    results.push((uuid, handle.clone()));
                }
                if let Some(uuid) = global_template {
                    results.push((uuid, handle.clone()));
                }
            }
        }

        results
    }

    /// Extract UUID and DisplayName (handle) pairs from LSX (XML) bytes
    /// Used for Origins files which define companion characters
    fn extract_speakers_from_lsx(data: &[u8]) -> Vec<(String, String)> {
        let mut results = Vec::new();

        // Parse as UTF-8 string first
        let Ok(xml_str) = std::str::from_utf8(data) else {
            return results;
        };

        let Ok(doc) = parse_lsx(xml_str) else {
            return results;
        };

        // Recursively process all nodes in all regions
        fn process_node(node: &crate::formats::lsx::LsxNode, results: &mut Vec<(String, String)>) {
            let mut global_template: Option<String> = None;
            let mut display_name_handle: Option<String> = None;

            // Check attributes for GlobalTemplate and DisplayName
            for attr in &node.attributes {
                match attr.id.as_str() {
                    "GlobalTemplate" => {
                        if !attr.value.is_empty() && attr.value.contains('-') {
                            global_template = Some(attr.value.clone());
                        }
                    }
                    "DisplayName" => {
                        // For TranslatedString, the handle is in the handle field
                        if let Some(ref handle) = attr.handle {
                            if !handle.is_empty() {
                                display_name_handle = Some(handle.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Add mapping if both GlobalTemplate and DisplayName are present
            if let (Some(uuid), Some(handle)) = (global_template, display_name_handle) {
                results.push((uuid, handle));
            }

            // Recursively process children
            for child in &node.children {
                process_node(child, results);
            }
        }

        // Process all regions and their nodes
        for region in &doc.regions {
            for node in &region.nodes {
                process_node(node, &mut results);
            }
        }

        results
    }

    /// Configure PAK sources from a game data directory
    pub fn configure_from_game_data<P: AsRef<Path>>(&mut self, data_path: P) {
        let data_path = data_path.as_ref();

        // Include main PAK files that contain character templates
        let pak_names = ["Gustav.pak", "Shared.pak"];
        for pak_name in pak_names {
            let pak_path = data_path.join(pak_name);
            if pak_path.exists() {
                self.add_pak_source(&pak_path);
            }
        }
    }
}

/// Error type for speaker cache operations
#[derive(Debug)]
pub enum SpeakerCacheError {
    IoError(String),
    PakError(String),
    ParseError(String),
}

impl std::fmt::Display for SpeakerCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpeakerCacheError::IoError(e) => write!(f, "IO error: {}", e),
            SpeakerCacheError::PakError(e) => write!(f, "PAK error: {}", e),
            SpeakerCacheError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for SpeakerCacheError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speaker_cache_basic() {
        let mut cache = SpeakerCache::new();
        assert!(cache.is_empty());

        cache.insert(
            "0e47fcb9-c0c4-4b0c-902b-2d13d209e760".to_string(),
            "h12345678g1234g1234g1234g123456789abc".to_string(),
        );

        assert_eq!(cache.len(), 1);
        assert_eq!(
            cache.get_handle("0e47fcb9-c0c4-4b0c-902b-2d13d209e760"),
            Some("h12345678g1234g1234g1234g123456789abc")
        );
    }
}
