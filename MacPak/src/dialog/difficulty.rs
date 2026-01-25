//! Difficulty class cache for resolving DC UUIDs to numeric values
//!
//! Loads DifficultyClasses.lsx files from PAK sources to map DC GUIDs
//! to their difficulty values and names.

use maclarian::formats::lsx::parse_lsx;
use maclarian::pak::PakOperations;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Info about a difficulty class
#[derive(Debug, Clone)]
pub struct DifficultyClassInfo {
    /// Human-readable name (e.g., "`Act1_Medium`", "`Legacy_10`")
    pub name: String,
    /// Difficulty value (the actual DC number)
    pub difficulty: i32,
}

/// Cache for difficulty class UUID → info resolution
///
/// Uses pre-indexing - builds a complete index when `build_index()` is called,
/// then provides O(1) lookups.
#[derive(Debug, Clone, Default)]
pub struct DifficultyClassCache {
    /// DC info indexed by UUID
    entries: HashMap<String, DifficultyClassInfo>,
    /// Whether the index has been built
    indexed: bool,
    /// PAK files to index from
    pak_paths: Vec<PathBuf>,
}

impl DifficultyClassCache {
    /// Create a new empty cache
    #[must_use] 
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            indexed: false,
            pak_paths: Vec::new(),
        }
    }

    /// Get the number of cached entries
    #[must_use] 
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.pak_paths.is_empty()
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
        self.entries.clear();
        self.indexed = false;
        self.pak_paths.clear();
    }

    /// Add a PAK file as a source for DC lookups
    pub fn add_pak_source<P: AsRef<Path>>(&mut self, pak_path: P) {
        let path = pak_path.as_ref().to_path_buf();
        if !self.pak_paths.contains(&path) {
            self.pak_paths.push(path);
        }
    }

    /// Look up DC info by UUID (O(1) after indexing)
    #[must_use] 
    pub fn get_info(&self, uuid: &str) -> Option<&DifficultyClassInfo> {
        self.entries.get(uuid)
    }

    /// Get just the difficulty value for a UUID
    #[must_use] 
    pub fn get_difficulty(&self, uuid: &str) -> Option<i32> {
        self.entries.get(uuid).map(|info| info.difficulty)
    }

    /// Get a formatted DC string (e.g., "DC 10" or "DC 15 (`Act2_Hard`)")
    #[must_use] 
    pub fn get_formatted(&self, uuid: &str) -> Option<String> {
        self.entries.get(uuid).map(|info| {
            format!("DC {}", info.difficulty)
        })
    }

    /// Insert an entry directly (for testing or manual additions)
    pub fn insert(&mut self, uuid: String, name: String, difficulty: i32) {
        self.entries.insert(uuid, DifficultyClassInfo { name, difficulty });
    }

    /// Build the DC index from all configured PAK sources.
    ///
    /// Scans DifficultyClasses.lsx files for UUID → difficulty mappings.
    /// Returns the number of DCs indexed.
    ///
    /// # Errors
    /// Returns an error if PAK files cannot be read or parsed.
    pub fn build_index(&mut self) -> Result<usize, DifficultyClassError> {
        if self.indexed {
            return Ok(self.entries.len());
        }

        let mut total_count = 0;

        for pak_path in self.pak_paths.clone() {
            // List all files in the PAK
            let all_files = PakOperations::list(&pak_path)
                .map_err(|e| DifficultyClassError::PakError(e.to_string()))?;

            // Filter for DifficultyClasses.lsx files
            let dc_files: Vec<String> = all_files
                .into_iter()
                .filter(|path| {
                    let lower = path.to_lowercase();
                    lower.contains("difficultyclasses") && lower.ends_with(".lsx")
                })
                .collect();

            if dc_files.is_empty() {
                continue;
            }

            tracing::debug!("Found {} DC files in {}", dc_files.len(), pak_path.display());

            // Batch read all DC files
            let file_data = PakOperations::read_files_bytes(&pak_path, &dc_files)
                .map_err(|e| DifficultyClassError::PakError(e.to_string()))?;

            // Parse DC files in parallel and collect results
            let parsed_dcs: Vec<Vec<(String, String, i32)>> = file_data
                .par_iter()
                .map(|(_path, data)| Self::extract_dcs_from_lsx(data))
                .collect();

            // Merge results sequentially
            for dcs in parsed_dcs {
                for (uuid, name, difficulty) in dcs {
                    self.entries.insert(uuid, DifficultyClassInfo { name, difficulty });
                    total_count += 1;
                }
            }
        }

        self.indexed = true;
        tracing::info!("Difficulty class index built: {} DCs", total_count);
        Ok(total_count)
    }

    /// Extract (UUID, Name, Difficulty) tuples from DifficultyClasses.lsx bytes
    fn extract_dcs_from_lsx(data: &[u8]) -> Vec<(String, String, i32)> {
        // Process all regions and their nodes
        fn process_node(node: &maclarian::formats::lsx::LsxNode, results: &mut Vec<(String, String, i32)>) {
            // DifficultyClass nodes have UUID, Name, and Difficulties attributes
            if node.id == "DifficultyClass" {
                let mut uuid: Option<String> = None;
                let mut name: Option<String> = None;
                let mut difficulty: Option<i32> = None;

                for attr in &node.attributes {
                    match attr.id.as_str() {
                        "UUID" => {
                            if !attr.value.is_empty() {
                                uuid = Some(attr.value.clone());
                            }
                        }
                        "Name" => {
                            if !attr.value.is_empty() {
                                name = Some(attr.value.clone());
                            }
                        }
                        "Difficulties" => {
                            // Difficulties is stored as a string, parse it as i32
                            if let Ok(d) = attr.value.parse::<i32>() {
                                difficulty = Some(d);
                            }
                        }
                        _ => {}
                    }
                }

                if let (Some(u), Some(n), Some(d)) = (uuid, name, difficulty) {
                    results.push((u, n, d));
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
    pub fn configure_from_game_data<P: AsRef<Path>>(&mut self, data_path: P) {
        let data_path = data_path.as_ref();

        // Include PAK files that contain DifficultyClasses
        let pak_names = ["Gustav.pak", "Shared.pak"];
        for pak_name in pak_names {
            let pak_path = data_path.join(pak_name);
            if pak_path.exists() {
                self.add_pak_source(&pak_path);
            }
        }
    }
}

/// Error type for difficulty class cache operations
#[derive(Debug)]
pub enum DifficultyClassError {
    IoError(String),
    PakError(String),
    ParseError(String),
}

impl std::fmt::Display for DifficultyClassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DifficultyClassError::IoError(e) => write!(f, "IO error: {e}"),
            DifficultyClassError::PakError(e) => write!(f, "PAK error: {e}"),
            DifficultyClassError::ParseError(e) => write!(f, "Parse error: {e}"),
        }
    }
}

impl std::error::Error for DifficultyClassError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_cache_basic() {
        let mut cache = DifficultyClassCache::new();
        assert!(cache.is_empty());

        cache.insert(
            "fa621d38-6f83-4e42-a55c-6aa651a75d46".to_string(),
            "Act1_Medium".to_string(),
            10,
        );

        assert_eq!(cache.len(), 1);
        assert_eq!(
            cache.get_difficulty("fa621d38-6f83-4e42-a55c-6aa651a75d46"),
            Some(10)
        );
        assert_eq!(
            cache.get_formatted("fa621d38-6f83-4e42-a55c-6aa651a75d46"),
            Some("DC 10".to_string())
        );
    }
}
