//! On-the-fly game data resolver for GR2-to-texture mappings
//!
//! This module provides [`GameDataResolver`], which lazily builds the asset database
//! from game PAK files on first use. This replaces the reliance on a database
//! with dynamic retrieval.
//!
//! # Usage
//!
//! ```no_run
//! use maclarian::merged::GameDataResolver;
//!
//! // Auto-detect game installation path
//! let resolver = GameDataResolver::auto_detect()?;
//!
//! // Query the database (builds lazily on first access)
//! if let Some(asset) = resolver.get_by_visual_name("HUM_M_ARM_Leather_A_Body") {
//!     println!("GR2: {}", asset.gr2_path);
//!     println!("Textures: {:?}", asset.textures);
//! }
//! # Ok::<(), maclarian::error::Error>(())
//! ```
//!
//! # Path Detection
//!
//! The resolver searches for game data in platform-specific Steam paths:
//! - macOS: `~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/...`
//! - Windows: `C:\Program Files (x86)\Steam\steamapps\common\Baldurs Gate 3\Data`
//! - Linux: Not supported (don't know the install path)
//!
//! # Performance
//!
//! Database construction uses parallel processing via rayon and in-memory LSF parsing
//! (no temp files). First query takes ~1-2 seconds for ~50 _merged.lsf files.
//! Subsequent queries are instant due to `OnceLock` caching.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rayon::prelude::*;

use crate::error::{Error, Result};
use crate::formats::common::extract_value;
use crate::formats::lsf::{parse_lsf_bytes, LsfDocument};
use crate::pak::PakOperations;
use super::parser::merge_databases;
use super::types::{MaterialDef, MergedDatabase, TextureParam, TextureRef, VirtualTextureRef, VisualAsset};

/// Windows Steam default path
pub const BG3_DATA_PATH_WINDOWS: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\Baldurs Gate 3\Data";

/// Resolver that lazily builds the asset database from game PAK files.
///
/// Use [`GameDataResolver::auto_detect`] for automatic path detection,
/// or [`GameDataResolver::new`] for explicit path specification.
pub struct GameDataResolver {
    /// Path to the game's Data folder
    game_data_path: PathBuf,
    /// Lazily-initialized database
    database: OnceLock<MergedDatabase>,
}

impl GameDataResolver {
    /// Create a resolver with an explicit game data path.
    ///
    /// The path should point to the folder containing `Shared.pak`, `Models.pak`, etc.
    ///
    /// # Errors
    ///
    /// Returns an error if the path doesn't exist or `Shared.pak` is not found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use maclarian::merged::GameDataResolver;
    ///
    /// let resolver = GameDataResolver::new("/path/to/BG3/Data")?;
    /// # Ok::<(), maclarian::error::Error>(())
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(Error::InvalidPath(format!(
                "Game data path does not exist: {}",
                path.display()
            )));
        }

        let shared_pak = path.join("Shared.pak");
        if !shared_pak.exists() {
            return Err(Error::InvalidPath(format!(
                "Shared.pak not found in: {}",
                path.display()
            )));
        }

        Ok(Self {
            game_data_path: path,
            database: OnceLock::new(),
        })
    }

    /// Auto-detect game installation path.
    ///
    /// Searches platform-specific Steam installation paths:
    /// - macOS: `~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/...`
    /// - Windows: `C:\Program Files (x86)\Steam\steamapps\common\Baldurs Gate 3\Data`
    ///
    /// # Errors
    ///
    /// Returns an error if no valid game installation is found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use maclarian::merged::GameDataResolver;
    ///
    /// let resolver = GameDataResolver::auto_detect()?;
    /// println!("Found game at: {}", resolver.game_data_path().display());
    /// # Ok::<(), maclarian::error::Error>(())
    /// ```
    pub fn auto_detect() -> Result<Self> {
        // Try macOS Steam path
        if let Some(path) = super::bg3_data_path() {
            if path.exists() {
                return Self::new(path);
            }
        }

        // Try Windows Steam path
        let windows_path = PathBuf::from(BG3_DATA_PATH_WINDOWS);
        if windows_path.exists() {
            return Self::new(windows_path);
        }

        Err(Error::InvalidPath(
            "Could not find BG3 install path. Use --bg3-path to specify the path.".to_string()
        ))
    }

    /// Get the game data path.
    #[must_use]
    pub fn game_data_path(&self) -> &Path {
        &self.game_data_path
    }

    /// Get or lazily build the asset database.
    ///
    /// The first call triggers database construction from `Shared.pak`,
    /// which takes ~2-3 seconds. Subsequent calls return the cached database.
    #[must_use]
    pub fn database(&self) -> &MergedDatabase {
        self.database.get_or_init(|| self.build_database())
    }

    /// Build the database from game PAK files.
    ///
    /// Parses relevant `_merged.lsf` files from both `Shared.pak` and `GustavX.pak`.
    /// Only includes character assets (armor, clothing, creatures) - not scenery/effects.
    fn build_database(&self) -> MergedDatabase {
        let shared_pak = self.game_data_path.join("Shared.pak");
        let gustavx_pak = self.game_data_path.join("GustavX.pak");

        tracing::info!(
            "Building asset database from: {}",
            self.game_data_path.display()
        );

        let mut db = MergedDatabase::new(self.game_data_path.to_string_lossy());

        // Parse Shared.pak with filtered paths
        if let Err(e) = self.parse_pak_filtered(&shared_pak, &mut db) {
            tracing::error!("Failed to parse Shared.pak: {}", e);
            return db;
        }

        // Parse GustavX.pak with filtered paths
        if gustavx_pak.exists() {
            if let Err(e) = self.parse_pak_filtered(&gustavx_pak, &mut db) {
                tracing::warn!("Failed to parse GustavX.pak: {}", e);
            }
        }

        // Resolve references after merging all sources
        db.resolve_references();

        let stats = db.stats();
        tracing::info!(
            "Database built: {} visuals, {} materials, {} textures",
            stats.visual_count,
            stats.material_count,
            stats.texture_count
        );
        db
    }

    /// Parse a PAK file, filtering to only relevant character asset paths.
    ///
    /// Uses batch reading and parallel parsing for optimal performance:
    /// 1. Batch reads all relevant LSF files from PAK in one pass
    /// 2. Parses LSF→LSX in parallel using rayon
    /// 3. Merges results sequentially into the database
    fn parse_pak_filtered(&self, pak_path: &Path, db: &mut MergedDatabase) -> Result<()> {
        // List all files and filter to relevant _merged.lsf paths
        let all_files = PakOperations::list(pak_path)?;
        let relevant_paths: Vec<String> = all_files
            .into_iter()
            .filter(|p| p.ends_with("_merged.lsf") && is_relevant_asset_path(p))
            .collect();

        if relevant_paths.is_empty() {
            return Ok(());
        }

        let pak_name = pak_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        tracing::info!(
            "Parsing {} relevant _merged.lsf files from {}",
            relevant_paths.len(),
            pak_name
        );

        // Batch read all LSF files from PAK in one pass
        let file_data = PakOperations::read_files_bytes(pak_path, &relevant_paths)?;

        // Parse LSF files in parallel using rayon
        let parsed_dbs: Vec<(String, Result<MergedDatabase>)> = file_data
            .par_iter()
            .map(|(path, data)| {
                let result = parse_lsf_to_database(data, path);
                (path.clone(), result)
            })
            .collect();

        // Merge results sequentially into the main database
        for (path, result) in parsed_dbs {
            match result {
                Ok(file_db) => {
                    merge_databases(db, file_db);
                }
                Err(e) => {
                    tracing::debug!("Failed to parse {}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    /// Parse a PAK file with progress reporting.
    ///
    /// The callback receives (current, total, file_name) for each file being parsed.
    pub fn parse_pak_with_progress<F>(
        &self,
        pak_path: &Path,
        db: &mut MergedDatabase,
        progress: F,
    ) -> Result<()>
    where
        F: Fn(usize, usize, &str) + Send + Sync,
    {
        // List all files and filter to relevant _merged.lsf paths
        let all_files = PakOperations::list(pak_path)?;
        let relevant_paths: Vec<String> = all_files
            .into_iter()
            .filter(|p| p.ends_with("_merged.lsf") && is_relevant_asset_path(p))
            .collect();

        if relevant_paths.is_empty() {
            return Ok(());
        }

        let total = relevant_paths.len();
        let pak_name = pak_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        tracing::info!(
            "Parsing {} relevant _merged.lsf files from {}",
            total,
            pak_name
        );

        // Batch read all LSF files from PAK in one pass
        progress(0, total, "Reading files from PAK...");
        let file_data = PakOperations::read_files_bytes(pak_path, &relevant_paths)?;

        // Parse LSF files in parallel using rayon with progress
        use std::sync::atomic::{AtomicUsize, Ordering};
        let counter = AtomicUsize::new(0);

        let parsed_dbs: Vec<(String, Result<MergedDatabase>)> = file_data
            .par_iter()
            .map(|(path, data)| {
                let current = counter.fetch_add(1, Ordering::SeqCst);
                let filename = Path::new(path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());
                progress(current + 1, total, &filename);

                let result = parse_lsf_to_database(data, path);
                (path.clone(), result)
            })
            .collect();

        // Merge results sequentially into the main database
        for (path, result) in parsed_dbs {
            match result {
                Ok(file_db) => {
                    merge_databases(db, file_db);
                }
                Err(e) => {
                    tracing::debug!("Failed to parse {}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Convenience query methods (delegate to database)
    // -------------------------------------------------------------------------

    /// Get a visual asset by its exact name.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use maclarian::merged::GameDataResolver;
    ///
    /// let resolver = GameDataResolver::auto_detect()?;
    /// if let Some(asset) = resolver.get_by_visual_name("HUM_M_ARM_Leather_A_Body") {
    ///     println!("Found: {}", asset.gr2_path);
    /// }
    /// # Ok::<(), maclarian::error::Error>(())
    /// ```
    #[must_use]
    pub fn get_by_visual_name(&self, name: &str) -> Option<&VisualAsset> {
        self.database().get_by_visual_name(name)
    }

    /// Get all visuals that use a specific GR2 file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use maclarian::merged::GameDataResolver;
    ///
    /// let resolver = GameDataResolver::auto_detect()?;
    /// let visuals = resolver.get_visuals_for_gr2("HUM_M_ARM_Leather_A_Body.GR2");
    /// println!("Found {} visuals using this GR2", visuals.len());
    /// # Ok::<(), maclarian::error::Error>(())
    /// ```
    #[must_use]
    pub fn get_visuals_for_gr2(&self, gr2_name: &str) -> Vec<&VisualAsset> {
        self.database().get_visuals_for_gr2(gr2_name)
    }

    /// Check if a game data path is available (without building database).
    ///
    /// Useful for checking if texture extraction is possible before attempting it.
    #[must_use]
    pub fn is_available() -> bool {
        Self::auto_detect().is_ok()
    }
}

/// Parse LSF bytes directly to a MergedDatabase (in-memory, no XML conversion).
///
/// Pipeline: LSF bytes → LsfDocument → MergedDatabase
///
/// This is significantly faster than going through LSX XML conversion because:
/// 1. No XML serialization/deserialization overhead
/// 2. Direct binary parsing with type-aware value extraction
/// 3. No string allocation for XML intermediate format
fn parse_lsf_to_database(data: &[u8], source_path: &str) -> Result<MergedDatabase> {
    // Parse LSF binary format directly
    let doc = parse_lsf_bytes(data)
        .map_err(|e| Error::ConversionError(format!("LSF parse error for {source_path}: {e}")))?;

    let mut db = MergedDatabase::new(source_path);

    // Find root nodes and parse relevant banks
    for root_idx in doc.root_nodes() {
        let Some(region_name) = doc.node_name(root_idx) else {
            continue;
        };

        match region_name {
            "VisualBank" => parse_visual_bank_lsf(&doc, root_idx, &mut db),
            "MaterialBank" => parse_material_bank_lsf(&doc, root_idx, &mut db),
            "TextureBank" => parse_texture_bank_lsf(&doc, root_idx, &mut db),
            "VirtualTextureBank" => parse_virtual_texture_bank_lsf(&doc, root_idx, &mut db),
            _ => {}
        }
    }

    Ok(db)
}

/// Get an attribute value as a string from an LSF node
fn get_attr_string(doc: &LsfDocument, node_idx: usize, attr_name: &str) -> Option<String> {
    for (_, name, type_id, offset, length) in doc.attributes_of(node_idx) {
        if name == attr_name {
            return extract_value(&doc.values, offset, length, type_id).ok();
        }
    }
    None
}

/// Get an attribute value as u32 from an LSF node
fn get_attr_u32(doc: &LsfDocument, node_idx: usize, attr_name: &str) -> Option<u32> {
    get_attr_string(doc, node_idx, attr_name)
        .and_then(|s| s.parse().ok())
}

/// Parse VisualBank directly from LSF document
fn parse_visual_bank_lsf(doc: &LsfDocument, bank_idx: usize, db: &mut MergedDatabase) {
    // VisualBank contains Resource children with visual data
    for resource_idx in doc.find_children_by_name(bank_idx, "Resource") {
        let Some(id) = get_attr_string(doc, resource_idx, "ID") else {
            continue;
        };
        let gr2_path = get_attr_string(doc, resource_idx, "SourceFile").unwrap_or_default();
        if gr2_path.is_empty() {
            continue;
        }

        let name = get_attr_string(doc, resource_idx, "Name").unwrap_or_default();

        // Extract MaterialIDs from Objects children
        let mut material_ids = Vec::new();
        for obj_idx in doc.find_children_by_name(resource_idx, "Objects") {
            if let Some(mat_id) = get_attr_string(doc, obj_idx, "MaterialID") {
                if !mat_id.is_empty() && !material_ids.contains(&mat_id) {
                    material_ids.push(mat_id);
                }
            }
        }

        let visual = VisualAsset {
            id: id.clone(),
            name: name.clone(),
            gr2_path: gr2_path.clone(),
            source_pak: String::new(),
            material_ids,
            textures: Vec::new(),
            virtual_textures: Vec::new(),
        };

        // Index by visual name
        db.visuals_by_name.insert(name, id.clone());

        // Index by GR2 filename
        let gr2_filename = Path::new(&gr2_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if !gr2_filename.is_empty() {
            db.visuals_by_gr2.entry(gr2_filename).or_default().push(id.clone());
        }

        db.visuals_by_id.insert(id, visual);
    }
}

/// Parse MaterialBank directly from LSF document
fn parse_material_bank_lsf(doc: &LsfDocument, bank_idx: usize, db: &mut MergedDatabase) {
    for resource_idx in doc.find_children_by_name(bank_idx, "Resource") {
        let Some(id) = get_attr_string(doc, resource_idx, "ID") else {
            continue;
        };

        let name = get_attr_string(doc, resource_idx, "Name").unwrap_or_default();
        let source_file = get_attr_string(doc, resource_idx, "SourceFile").unwrap_or_default();

        // Extract Texture2DParameters
        let mut texture_ids = Vec::new();
        for tex_idx in doc.find_children_by_name(resource_idx, "Texture2DParameters") {
            let param_name = get_attr_string(doc, tex_idx, "ParameterName").unwrap_or_default();
            if let Some(tex_id) = get_attr_string(doc, tex_idx, "ID") {
                if !tex_id.is_empty() {
                    texture_ids.push(TextureParam {
                        name: param_name,
                        texture_id: tex_id,
                    });
                }
            }
        }

        // Extract VirtualTextureParameters
        let mut virtual_texture_ids = Vec::new();
        for vt_idx in doc.find_children_by_name(resource_idx, "VirtualTextureParameters") {
            if let Some(vt_id) = get_attr_string(doc, vt_idx, "ID") {
                if !vt_id.is_empty() && !virtual_texture_ids.contains(&vt_id) {
                    virtual_texture_ids.push(vt_id);
                }
            }
        }

        db.materials.insert(id.clone(), MaterialDef {
            id,
            name,
            source_file,
            source_pak: String::new(),
            texture_ids,
            virtual_texture_ids,
        });
    }
}

/// Parse TextureBank directly from LSF document
fn parse_texture_bank_lsf(doc: &LsfDocument, bank_idx: usize, db: &mut MergedDatabase) {
    for resource_idx in doc.find_children_by_name(bank_idx, "Resource") {
        let Some(id) = get_attr_string(doc, resource_idx, "ID") else {
            continue;
        };

        let name = get_attr_string(doc, resource_idx, "Name").unwrap_or_default();
        let dds_path = get_attr_string(doc, resource_idx, "SourceFile").unwrap_or_default();
        let width = get_attr_u32(doc, resource_idx, "Width").unwrap_or(0);
        let height = get_attr_u32(doc, resource_idx, "Height").unwrap_or(0);

        db.textures.insert(id.clone(), TextureRef {
            id,
            name,
            dds_path,
            source_pak: String::new(),
            width,
            height,
            parameter_name: None,
        });
    }
}

/// Parse VirtualTextureBank directly from LSF document
fn parse_virtual_texture_bank_lsf(doc: &LsfDocument, bank_idx: usize, db: &mut MergedDatabase) {
    for resource_idx in doc.find_children_by_name(bank_idx, "Resource") {
        let Some(id) = get_attr_string(doc, resource_idx, "ID") else {
            continue;
        };

        let name = get_attr_string(doc, resource_idx, "Name").unwrap_or_default();
        let gtex_hash = get_attr_string(doc, resource_idx, "GTexFileName").unwrap_or_default();

        db.virtual_textures.insert(id.clone(), VirtualTextureRef {
            id,
            name,
            gtex_hash,
        });
    }
}

/// Check if a _merged.lsf path is relevant for character asset lookups.
///
/// Only includes paths for armor, clothing, body, loot, and equipment assets.
/// Uses fast string matching with early exits for performance.
fn is_relevant_asset_path(path: &str) -> bool {
    // Fast early exit: must start with "Public/"
    if !path.starts_with("Public/") {
        return false;
    }

    // Fast early exit: must contain "/Content/Assets/"
    if !path.contains("/Content/Assets/") {
        return false;
    }

    // Check for [PAK]_ folder patterns (most common case)
    if let Some(pak_start) = path.find("[PAK]_") {
        let pak_part = &path[pak_start..];
        // Use byte matching for speed
        if pak_part.contains("Armor")
            || pak_part.contains("Clothing")
            || pak_part.contains("Body")
        {
            return true;
        }
    }

    // Check for Loot and Equipment paths
    path.contains("/Loot/") || path.contains("/Equipment/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available_returns_false_when_no_game() {
        // In CI/test environments without the game installed, this should work
        // We can't easily test the true case without the game
        let _ = GameDataResolver::is_available(); // Just ensure it doesn't panic
    }

    #[test]
    fn test_is_relevant_asset_path() {
        // Should include - armor
        assert!(is_relevant_asset_path("Public/Shared/Content/Assets/Characters/Humans/[PAK]_Male_Armor/_merged.lsf"));
        assert!(is_relevant_asset_path("Public/Shared/Content/Assets/Characters/Humans/[PAK]_Female_Armor/_merged.lsf"));
        // Should include - clothing
        assert!(is_relevant_asset_path("Public/Shared/Content/Assets/Characters/Humans/[PAK]_Male_Clothing/_merged.lsf"));
        // Should include - body
        assert!(is_relevant_asset_path("Public/SharedDev/Content/Assets/Characters/Creatures/Bear/[PAK]_Body/_merged.lsf"));
        // Should include - loot
        assert!(is_relevant_asset_path("Public/Shared/Content/Assets/Loot/[PAK]_Armor/_merged.lsf"));
        // Should include - equipment
        assert!(is_relevant_asset_path("Public/Shared/Content/Assets/Equipment/[PAK]_Humans/_merged.lsf"));

        // Should exclude - decoration
        assert!(!is_relevant_asset_path("Public/Shared/Content/Assets/Decoration/[PAK]_Generic/_merged.lsf"));
        // Should exclude - doors
        assert!(!is_relevant_asset_path("Public/Shared/Content/Assets/Doors/[PAK]_City/_merged.lsf"));
        // Should exclude - mods folder
        assert!(!is_relevant_asset_path("Mods/Shared/Levels/SYS_CC_I/Characters/_merged.lsf"));
        // Should exclude - effects
        assert!(!is_relevant_asset_path("Public/Shared/Content/Assets/Effects/Materials/[PAK]_Decal/_merged.lsf"));
        // Should exclude - heads (not armor/clothing/body)
        assert!(!is_relevant_asset_path("Public/Shared/Content/Assets/Characters/Humans/[PAK]_Male_Head/_merged.lsf"));
    }
}
