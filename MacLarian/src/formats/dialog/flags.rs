//! Flag cache for resolving flag UUIDs to human-readable names
//!
//! Uses pre-indexing - all flag names are loaded once when PAK sources are
//! configured, then lookups are O(1) HashMap access.

use crate::formats::lsf::parse_lsf_bytes;
use crate::formats::common::extract_value;
use crate::pak::PakOperations;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Cache for flag UUID → name resolution
///
/// Uses pre-indexing - builds a complete index when `build_index()` is called,
/// then provides O(1) lookups. Similar to how LocalizationCache works.
#[derive(Debug, Clone, Default)]
pub struct FlagCache {
    /// Flag names indexed by UUID (pre-loaded)
    names: HashMap<String, String>,
    /// Whether the index has been built
    indexed: bool,
    /// PAK files to index from
    pak_paths: Vec<PathBuf>,
}

impl FlagCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            indexed: false,
            pak_paths: Vec::new(),
        }
    }

    /// Get the number of cached flags
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.pak_paths.is_empty()
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
        self.names.clear();
        self.indexed = false;
        self.pak_paths.clear();
    }

    /// Add a PAK file as a source for flag lookups
    pub fn add_pak_source<P: AsRef<Path>>(&mut self, pak_path: P) {
        let path = pak_path.as_ref().to_path_buf();
        if !self.pak_paths.contains(&path) {
            self.pak_paths.push(path);
        }
    }

    /// Look up a flag name by UUID (O(1) after indexing)
    pub fn get_name(&self, uuid: &str) -> Option<&str> {
        self.names.get(uuid).map(|s| s.as_str())
    }

    /// Insert a flag name directly (for testing or manual additions)
    pub fn insert(&mut self, uuid: String, name: String) {
        self.names.insert(uuid, name);
    }

    /// Build the flag name index from all configured PAK sources.
    ///
    /// Call this once after `configure_from_game_data()` to pre-load all flag names.
    /// Returns the number of flags indexed.
    pub fn build_index(&mut self) -> Result<usize, FlagCacheError> {
        if self.indexed {
            return Ok(self.names.len());
        }

        let mut total_count = 0;

        for pak_path in self.pak_paths.clone() {
            // List all files in the PAK
            let all_files = PakOperations::list(&pak_path)
                .map_err(|e| FlagCacheError::PakError(e.to_string()))?;

            // Filter for flag files: */Flags/*.lsf
            let flag_files: Vec<String> = all_files
                .into_iter()
                .filter(|path| {
                    let lower = path.to_lowercase();
                    lower.contains("/flags/") && lower.ends_with(".lsf")
                })
                .collect();

            if flag_files.is_empty() {
                continue;
            }

            tracing::debug!("Found {} flag files in {}", flag_files.len(), pak_path.display());

            // Batch read all flag files
            let file_data = PakOperations::read_files_bytes(&pak_path, &flag_files)
                .map_err(|e| FlagCacheError::PakError(e.to_string()))?;

            // Parse each flag file and extract UUID + Name
            for (_path, data) in file_data {
                if let Some((uuid, name)) = self.extract_flag_name_from_lsf(&data) {
                    self.names.insert(uuid, name);
                    total_count += 1;
                }
            }
        }

        self.indexed = true;
        tracing::info!("Flag index built: {} flags", total_count);
        Ok(total_count)
    }

    /// Extract just UUID and Name from flag LSF bytes
    fn extract_flag_name_from_lsf(&self, data: &[u8]) -> Option<(String, String)> {
        let doc = parse_lsf_bytes(data).ok()?;

        for node in doc.nodes.iter() {
            let node_name = doc.get_name(node.name_index_outer, node.name_index_inner)
                .unwrap_or("");

            if node_name != "Flags" {
                continue;
            }

            let mut flag_name: Option<String> = None;
            let mut flag_uuid: Option<String> = None;

            if node.first_attribute_index >= 0 {
                let mut attr_idx = node.first_attribute_index as usize;
                loop {
                    if attr_idx >= doc.attributes.len() {
                        break;
                    }

                    let attr = &doc.attributes[attr_idx];
                    let attr_name = doc.get_name(attr.name_index_outer, attr.name_index_inner)
                        .unwrap_or("");
                    let type_id = attr.type_info & 0x3F;
                    let value_length = (attr.type_info >> 6) as usize;

                    match attr_name {
                        "Name" => {
                            if let Ok(val) = extract_value(&doc.values, attr.offset, value_length, type_id) {
                                if !val.is_empty() {
                                    flag_name = Some(val);
                                }
                            }
                        }
                        "UUID" => {
                            if let Ok(val) = extract_value(&doc.values, attr.offset, value_length, type_id) {
                                if !val.is_empty() {
                                    flag_uuid = Some(val);
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

            if let (Some(name), Some(uuid)) = (flag_name, flag_uuid) {
                return Some((uuid, name));
            }
        }

        None
    }

    /// Configure PAK sources from a game data directory
    ///
    /// Call this once when loading a PAK - it just stores the paths,
    /// actual flag loading happens on-demand.
    pub fn configure_from_game_data<P: AsRef<Path>>(&mut self, data_path: P) {
        let data_path = data_path.as_ref();

        let pak_names = ["Gustav.pak", "Shared.pak"];
        for pak_name in pak_names {
            let pak_path = data_path.join(pak_name);
            if pak_path.exists() {
                self.add_pak_source(&pak_path);
            }
        }
    }
}

/// Error type for flag cache operations
#[derive(Debug)]
pub enum FlagCacheError {
    IoError(String),
    PakError(String),
    ParseError(String),
}

impl std::fmt::Display for FlagCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagCacheError::IoError(e) => write!(f, "IO error: {}", e),
            FlagCacheError::PakError(e) => write!(f, "PAK error: {}", e),
            FlagCacheError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for FlagCacheError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_cache_basic() {
        let mut cache = FlagCache::new();
        assert!(cache.is_empty());

        cache.insert(
            "7e84b35a-5a77-456d-4dd7-4c96e1247bc1".to_string(),
            "CAMP_GoblinHunt_HasMet_Astarion".to_string(),
        );

        assert_eq!(cache.len(), 1);
        assert_eq!(
            cache.get_name("7e84b35a-5a77-456d-4dd7-4c96e1247bc1"),
            Some("CAMP_GoblinHunt_HasMet_Astarion")
        );
    }

    #[test]
    fn test_flag_cache_not_indexed_returns_none() {
        let cache = FlagCache::new();
        assert!(!cache.is_indexed());
        assert_eq!(cache.get_name("nonexistent-uuid"), None);
    }
}
