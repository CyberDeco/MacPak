//! Resolver for building asset databases from _merged.lsf files

use crate::converter::convert_lsf_to_lsx;
use crate::error::{Error, Result};
use crate::pak::PakOperations;

use super::parser::{
    merge_databases, parse_material_bank, parse_texture_bank, parse_virtual_texture_bank,
    parse_visual_bank, resolve_references,
};
use super::paths::{path_with_tilde, virtual_textures_pak_path};
use super::types::{MergedDatabase, VisualAsset, DatabaseStats, GtpMatch};

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Resolver for extracting GR2-to-texture mappings from _merged files
pub struct MergedResolver {
    database: MergedDatabase,
}

impl MergedResolver {
    /// Create a resolver from an already-parsed database
    #[must_use] 
    pub fn from_database(database: MergedDatabase) -> Self {
        Self { database }
    }

    /// Create a resolver from an extracted folder containing _merged.lsf files
    ///
    /// # Errors
    /// Returns an error if no merged files are found or if parsing fails.
    pub fn from_folder<P: AsRef<Path>>(folder: P) -> Result<Self> {
        let folder = folder.as_ref();
        tracing::info!(
            "Building merged database from folder: {}",
            folder.display()
        );

        let merged_files = find_merged_files(folder)?;
        if merged_files.is_empty() {
            return Err(Error::FileNotFoundInPak(
                "No _merged.lsf files found".to_string(),
            ));
        }

        tracing::info!("Found {} _merged.lsf files", merged_files.len());

        let temp_dir = TempDir::new()?;
        let mut combined_db = MergedDatabase::new(folder.to_string_lossy());

        for lsf_path in &merged_files {
            let lsx_filename = lsf_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .replace(".lsf", ".lsx");
            let lsx_path = temp_dir.path().join(&lsx_filename);

            tracing::debug!(
                "Converting {} -> {}",
                lsf_path.display(),
                lsx_path.display()
            );
            convert_lsf_to_lsx(lsf_path, &lsx_path)?;

            let db = Self::parse_lsx_file(&lsx_path)?;
            merge_databases(&mut combined_db, db);
        }

        resolve_references(&mut combined_db);
        log_stats(&combined_db);

        Ok(Self {
            database: combined_db,
        })
    }

    /// Create a resolver from a specific _merged.lsf file inside a .pak
    ///
    /// # Errors
    /// Returns an error if the file is not found or if parsing fails.
    pub fn from_pak_file<P: AsRef<Path>>(pak_path: P, lsf_path: &str) -> Result<Self> {
        let pak_path = pak_path.as_ref();
        tracing::info!(
            "Building merged database from pak file: {} -> {}",
            pak_path.display(),
            lsf_path
        );

        let temp_dir = TempDir::new()?;
        PakOperations::extract_files(pak_path, temp_dir.path(), &[lsf_path])?;

        let extracted_lsf = temp_dir.path().join(lsf_path);
        if !extracted_lsf.exists() {
            return Err(Error::FileNotFoundInPak(lsf_path.to_string()));
        }

        let lsx_path = extracted_lsf.with_extension("lsx");
        convert_lsf_to_lsx(&extracted_lsf, &lsx_path)?;

        let mut database = Self::parse_lsx_file(&lsx_path)?;
        database.source_path = format!("{}:{}", path_with_tilde(pak_path), lsf_path);
        resolve_references(&mut database);
        log_stats(&database);

        Ok(Self { database })
    }

    /// Create a resolver from a .pak file (all _merged.lsf files)
    ///
    /// # Errors
    /// Returns an error if no merged files are found or if parsing fails.
    pub fn from_pak<P: AsRef<Path>>(pak_path: P) -> Result<Self> {
        let pak_path = pak_path.as_ref();
        tracing::info!("Building merged database from pak: {}", pak_path.display());

        let all_files = PakOperations::list(pak_path)?;
        let merged_paths: Vec<_> = all_files
            .iter()
            .filter(|p| p.ends_with("_merged.lsf"))
            .collect();

        if merged_paths.is_empty() {
            return Err(Error::FileNotFoundInPak(
                "No _merged.lsf files found in pak".to_string(),
            ));
        }

        tracing::info!("Found {} _merged.lsf files in pak", merged_paths.len());

        let temp_dir = TempDir::new()?;
        let merged_strs: Vec<&str> = merged_paths.iter().map(|s| s.as_str()).collect();
        PakOperations::extract_files(pak_path, temp_dir.path(), &merged_strs)?;

        let mut combined_db = MergedDatabase::new(pak_path.to_string_lossy());

        for merged_rel_path in &merged_paths {
            let lsf_path = temp_dir.path().join(merged_rel_path);
            if !lsf_path.exists() {
                tracing::warn!("Extracted file not found: {}", lsf_path.display());
                continue;
            }

            let lsx_path = lsf_path.with_extension("lsx");
            tracing::debug!(
                "Converting {} -> {}",
                lsf_path.display(),
                lsx_path.display()
            );
            convert_lsf_to_lsx(&lsf_path, &lsx_path)?;

            let db = Self::parse_lsx_file(&lsx_path)?;
            merge_databases(&mut combined_db, db);
        }

        resolve_references(&mut combined_db);
        log_stats(&combined_db);

        Ok(Self {
            database: combined_db,
        })
    }

    /// Create a resolver from a single _merged.lsx file (already converted)
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_lsx<P: AsRef<Path>>(lsx_path: P) -> Result<Self> {
        let lsx_path = lsx_path.as_ref();
        tracing::info!(
            "Building merged database from LSX: {}",
            lsx_path.display()
        );

        let mut database = Self::parse_lsx_file(lsx_path)?;
        resolve_references(&mut database);
        log_stats(&database);

        Ok(Self { database })
    }

    /// Parse a single LSX file and extract the banks
    fn parse_lsx_file<P: AsRef<Path>>(path: P) -> Result<MergedDatabase> {
        let path = path.as_ref();
        let doc = crate::formats::lsx::read_lsx(path)?;
        let mut db = MergedDatabase::new(path.to_string_lossy());

        for region in &doc.regions {
            match region.id.as_str() {
                "VisualBank" => parse_visual_bank(region, &mut db),
                "MaterialBank" => parse_material_bank(region, &mut db),
                "TextureBank" => parse_texture_bank(region, &mut db),
                "VirtualTextureBank" => parse_virtual_texture_bank(region, &mut db),
                _ => {}
            }
        }

        Ok(db)
    }

    // -------------------------------------------------------------------------
    // Query methods
    // -------------------------------------------------------------------------

    /// Get a visual asset by its exact name
    #[must_use] 
    pub fn get_by_visual_name(&self, visual_name: &str) -> Option<&VisualAsset> {
        self.database.get_by_visual_name(visual_name)
    }

    /// Get all visuals that use a specific GR2 file
    #[must_use] 
    pub fn get_visuals_for_gr2(&self, gr2_name: &str) -> Vec<&VisualAsset> {
        self.database.get_visuals_for_gr2(gr2_name)
    }

    /// Get all visual names in the database
    pub fn visual_names(&self) -> impl Iterator<Item = &str> {
        self.database.visual_names()
    }

    /// Get all GR2 filenames in the database
    pub fn gr2_files(&self) -> impl Iterator<Item = &str> {
        self.database.gr2_files()
    }

    /// Get database statistics
    #[must_use] 
    pub fn stats(&self) -> DatabaseStats {
        self.database.stats()
    }

    /// Get a reference to the underlying database
    #[must_use] 
    pub fn database(&self) -> &MergedDatabase {
        &self.database
    }

    /// Consume and return the database
    #[must_use] 
    pub fn into_database(self) -> MergedDatabase {
        self.database
    }

    // -------------------------------------------------------------------------
    // I/O methods
    // -------------------------------------------------------------------------

    /// Save the database to a JSON file
    ///
    /// # Errors
    /// Returns an error if serialization or file writing fails.
    pub fn save_to_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.database)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a database from a JSON file
    ///
    /// # Errors
    /// Returns an error if reading or deserialization fails.
    pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let database: MergedDatabase = serde_json::from_str(&json)?;
        Ok(Self { database })
    }

    // -------------------------------------------------------------------------
    // Virtual texture lookup
    // -------------------------------------------------------------------------

    /// Find .gtp files in VirtualTextures.pak for a given visual
    ///
    /// # Errors
    /// Returns an error if the pak path cannot be determined or listing fails.
    pub fn find_virtual_textures_for_visual(&self, visual_name: &str) -> Result<Vec<GtpMatch>> {
        let pak_path = virtual_textures_pak_path().ok_or_else(|| {
            Error::ConversionError("Could not determine VirtualTextures.pak path".to_string())
        })?;
        self.find_virtual_textures_for_visual_in_pak(visual_name, &pak_path)
    }

    /// Find .gtp files in a specific pak for a given visual
    ///
    /// # Errors
    /// Returns an error if the visual is not found or listing fails.
    pub fn find_virtual_textures_for_visual_in_pak<P: AsRef<Path>>(
        &self,
        visual_name: &str,
        pak_path: P,
    ) -> Result<Vec<GtpMatch>> {
        let asset = self.get_by_visual_name(visual_name).ok_or_else(|| {
            Error::FileNotFoundInPak(format!("Visual not found: {visual_name}"))
        })?;

        if asset.virtual_textures.is_empty() {
            return Ok(Vec::new());
        }

        let hashes: Vec<&str> = asset
            .virtual_textures
            .iter()
            .map(|vt| vt.gtex_hash.as_str())
            .collect();

        self.find_gtp_by_hashes_in_pak(&hashes, pak_path)
    }

    /// Find .gtp files matching the given `GTex` hashes
    ///
    /// # Errors
    /// Returns an error if the pak path cannot be determined or listing fails.
    pub fn find_gtp_by_hashes(&self, hashes: &[&str]) -> Result<Vec<GtpMatch>> {
        let pak_path = virtual_textures_pak_path().ok_or_else(|| {
            Error::ConversionError("Could not determine VirtualTextures.pak path".to_string())
        })?;
        self.find_gtp_by_hashes_in_pak(hashes, &pak_path)
    }

    /// Find .gtp files in a specific pak matching the given `GTex` hashes
    ///
    /// # Errors
    /// Returns an error if the PAK file cannot be read.
    pub fn find_gtp_by_hashes_in_pak<P: AsRef<Path>>(
        &self,
        hashes: &[&str],
        pak_path: P,
    ) -> Result<Vec<GtpMatch>> {
        let pak_path = pak_path.as_ref();

        if !pak_path.exists() {
            return Err(Error::ConversionError(format!(
                "VirtualTextures.pak not found: {}",
                pak_path.display()
            )));
        }

        tracing::debug!(
            "Searching {} for {} GTex hashes",
            pak_path.display(),
            hashes.len()
        );

        let all_files = PakOperations::list(pak_path)?;
        let mut matches = Vec::new();

        for file_path in &all_files {
            if !file_path.ends_with(".gtp") {
                continue;
            }

            let filename = Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let stem = filename.strip_suffix(".gtp").unwrap_or(filename);

            for hash in hashes {
                if stem.ends_with(hash) {
                    matches.push(GtpMatch {
                        gtex_hash: (*hash).to_string(),
                        gtp_path: file_path.clone(),
                        pak_path: pak_path.to_path_buf(),
                    });
                    break;
                }
            }
        }

        tracing::debug!("Found {} matching .gtp files", matches.len());
        Ok(matches)
    }
}

// -----------------------------------------------------------------------------
// Helper functions
// -----------------------------------------------------------------------------

/// Find all _merged.lsf files recursively in a folder
fn find_merged_files(folder: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    find_merged_files_recursive(folder, &mut results)?;
    Ok(results)
}

fn find_merged_files_recursive(folder: &Path, results: &mut Vec<PathBuf>) -> Result<()> {
    if !folder.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            find_merged_files_recursive(&path, results)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name == "_merged.lsf" {
                results.push(path);
            }
    }

    Ok(())
}

/// Log database statistics
fn log_stats(db: &MergedDatabase) {
    let stats = db.stats();
    tracing::info!(
        "Built database: {} visuals, {} materials, {} textures, {} virtual textures",
        stats.visual_count,
        stats.material_count,
        stats.texture_count,
        stats.virtual_texture_count
    );
}
