//! Flag cache for resolving flag UUIDs to human-readable names
//!
//! Uses pre-indexing - all flag names are loaded once when PAK sources are
//! configured, then lookups are O(1) `HashMap` access.

use maclarian::formats::common::extract_value;
use maclarian::formats::lsf::parse_lsf_bytes;
use maclarian::formats::lsx::parse_lsx;
use maclarian::pak::PakOperations;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Cache for flag UUID → name resolution
///
/// Uses pre-indexing - builds a complete index when `build_index()` is called,
/// then provides O(1) lookups. Similar to how `LocalizationCache` works.
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            indexed: false,
            pak_paths: Vec::new(),
        }
    }

    /// Get the number of cached flags
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Check if cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.pak_paths.is_empty()
    }

    /// Check if PAK sources are configured
    #[must_use]
    pub fn has_sources(&self) -> bool {
        !self.pak_paths.is_empty()
    }

    /// Check if the index has been built
    #[must_use]
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
    #[must_use]
    pub fn get_name(&self, uuid: &str) -> Option<&str> {
        self.names.get(uuid).map(std::string::String::as_str)
    }

    /// Insert a flag name directly (for testing or manual additions)
    pub fn insert(&mut self, uuid: String, name: String) {
        self.names.insert(uuid, name);
    }

    /// Build the flag name index from all configured PAK sources.
    ///
    /// Call this once after `configure_from_game_data()` to pre-load all flag names.
    /// Returns the number of flags indexed.
    ///
    /// # Errors
    /// Returns an error if PAK files cannot be read or parsed.
    pub fn build_index(&mut self) -> Result<usize, FlagCacheError> {
        if self.indexed {
            return Ok(self.names.len());
        }

        let mut total_count = 0;

        for pak_path in self.pak_paths.clone() {
            // List all files in the PAK
            let all_files = PakOperations::list(&pak_path)
                .map_err(|e| FlagCacheError::PakError(e.to_string()))?;

            // Filter for flag, tag, and script flag files
            let flag_files: Vec<String> = all_files
                .into_iter()
                .filter(|path| {
                    let lower = path.to_lowercase();
                    // Individual flag/tag definition files (.lsf)
                    ((lower.contains("/flags/") || lower.contains("/tags/")) && lower.ends_with(".lsf"))
                        // ScriptFlags.lsx contains script-based flags
                        || (lower.contains("/scriptflags/") && lower.ends_with(".lsx"))
                        // Quest prototypes contain DialogFlagGUID → ID mappings
                        || (lower.contains("/journal/") && lower.contains("quest") && lower.ends_with(".lsx"))
                })
                .collect();

            if flag_files.is_empty() {
                continue;
            }

            tracing::debug!(
                "Found {} flag files in {}",
                flag_files.len(),
                pak_path.display()
            );

            // Batch read all flag files
            let file_data = PakOperations::read_files_bytes(&pak_path, &flag_files)
                .map_err(|e| FlagCacheError::PakError(e.to_string()))?;

            // Parse flag files in parallel and collect results
            // Handle both LSF (individual flags/tags) and LSX (ScriptFlags) formats
            let parsed_flags: Vec<Vec<(String, String)>> = file_data
                .par_iter()
                .map(|(path, data)| {
                    let lower_path = path.to_lowercase();
                    if lower_path.ends_with(".lsx") {
                        Self::extract_flags_from_lsx(data)
                    } else {
                        // LSF returns single flag, wrap in Vec
                        Self::extract_flag_name_from_lsf_static(data)
                            .into_iter()
                            .collect()
                    }
                })
                .collect();

            // Merge results sequentially
            for flags in parsed_flags {
                for (uuid, name) in flags {
                    self.names.insert(uuid, name);
                    total_count += 1;
                }
            }
        }

        self.indexed = true;
        tracing::info!("Flag index built: {} flags", total_count);
        Ok(total_count)
    }

    /// Extract just UUID and Name from flag LSF bytes (static version for parallel use)
    fn extract_flag_name_from_lsf_static(data: &[u8]) -> Option<(String, String)> {
        let doc = parse_lsf_bytes(data).ok()?;

        for node in &doc.nodes {
            let node_name = doc
                .get_name(node.name_index_outer, node.name_index_inner)
                .unwrap_or("");

            // Handle both Flags and Tags nodes
            if node_name != "Flags" && node_name != "Tags" {
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
                    let attr_name = doc
                        .get_name(attr.name_index_outer, attr.name_index_inner)
                        .unwrap_or("");
                    let type_id = attr.type_info & 0x3F;
                    let value_length = (attr.type_info >> 6) as usize;

                    match attr_name {
                        "Name" => {
                            if let Ok(val) =
                                extract_value(&doc.values, attr.offset, value_length, type_id)
                                && !val.is_empty()
                            {
                                flag_name = Some(val);
                            }
                        }
                        "UUID" => {
                            if let Ok(val) =
                                extract_value(&doc.values, attr.offset, value_length, type_id)
                                && !val.is_empty()
                            {
                                flag_uuid = Some(val);
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

    /// Extract UUID and name pairs from `ScriptFlags` LSX (XML) bytes
    fn extract_flags_from_lsx(data: &[u8]) -> Vec<(String, String)> {
        // Recursively process all nodes in all regions
        fn process_node(
            node: &maclarian::formats::lsx::LsxNode,
            results: &mut Vec<(String, String)>,
        ) {
            // ScriptFlags.lsx uses "ScriptFlag" nodes with "name" and "UUID" attributes
            if node.id == "ScriptFlag" {
                let mut flag_name: Option<String> = None;
                let mut flag_uuid: Option<String> = None;

                for attr in &node.attributes {
                    match attr.id.as_str() {
                        "name" => {
                            if !attr.value.is_empty() {
                                flag_name = Some(attr.value.clone());
                            }
                        }
                        "UUID" => {
                            if !attr.value.is_empty() {
                                flag_uuid = Some(attr.value.clone());
                            }
                        }
                        _ => {}
                    }
                }

                if let (Some(name), Some(uuid)) = (flag_name, flag_uuid) {
                    results.push((uuid, name));
                }
            }

            // Quest prototypes use "QuestStep" nodes with "DialogFlagGUID" and "ID" attributes
            if node.id == "QuestStep" {
                let mut flag_name: Option<String> = None;
                let mut flag_uuid: Option<String> = None;

                for attr in &node.attributes {
                    match attr.id.as_str() {
                        "ID" => {
                            if !attr.value.is_empty() {
                                flag_name = Some(attr.value.clone());
                            }
                        }
                        "DialogFlagGUID" => {
                            // Skip empty/null GUIDs
                            if !attr.value.is_empty()
                                && attr.value != "00000000-0000-0000-0000-000000000000"
                            {
                                flag_uuid = Some(attr.value.clone());
                            }
                        }
                        _ => {}
                    }
                }

                if let (Some(name), Some(uuid)) = (flag_name, flag_uuid) {
                    results.push((uuid, name));
                }
            }

            // Recursively process children
            for child in &node.children {
                process_node(child, results);
            }
        }

        let mut results = Vec::new();

        // Parse as UTF-8 string first
        let Ok(xml_str) = std::str::from_utf8(data) else {
            return results;
        };

        let Ok(doc) = parse_lsx(xml_str) else {
            return results;
        };

        // Process all regions and their nodes
        for region in &doc.regions {
            for node in &region.nodes {
                process_node(node, &mut results);
            }
        }

        results
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
            FlagCacheError::IoError(e) => write!(f, "IO error: {e}"),
            FlagCacheError::PakError(e) => write!(f, "PAK error: {e}"),
            FlagCacheError::ParseError(e) => write!(f, "Parse error: {e}"),
        }
    }
}

impl std::error::Error for FlagCacheError {}
