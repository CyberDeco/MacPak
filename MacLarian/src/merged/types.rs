//! Type definitions for merged LSX asset database

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A visual asset (mesh) with its associated textures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualAsset {
    /// The visual resource ID (GUID)
    pub id: String,
    /// Human-readable name (e.g., "HUM_M_ARM_Robe_C_Bracers_0")
    pub name: String,
    /// Path to the GR2 source file (e.g., "Generated/Public/Shared/Assets/...")
    pub gr2_path: String,
    /// Pak file where the GR2 mesh is located (e.g., "Models.pak")
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_pak: String,
    /// Material IDs referenced by this visual
    pub material_ids: Vec<String>,
    /// Resolved DDS texture paths (from TextureBank)
    pub textures: Vec<TextureRef>,
    /// Resolved virtual texture references (from VirtualTextureBank)
    pub virtual_textures: Vec<VirtualTextureRef>,
}

/// Reference to a DDS texture in TextureBank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureRef {
    /// Texture resource ID (GUID)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Path to the DDS source file (e.g., "Generated/Public/Shared/Assets/...")
    pub dds_path: String,
    /// Pak file where this texture is located (e.g., "Textures.pak")
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_pak: String,
    /// Texture dimensions
    pub width: u32,
    pub height: u32,
    /// Parameter name in material (e.g., "MSKColor", "NormalMap")
    pub parameter_name: Option<String>,
}

/// Reference to a virtual/streaming texture in VirtualTextureBank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualTextureRef {
    /// Virtual texture resource ID (GUID)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// GTex filename hash (32-char hex)
    pub gtex_hash: String,
}

/// Pak file paths for resolving assets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PakPaths {
    /// Pak containing GR2 mesh files (typically "Models.pak")
    pub models: String,
    /// Pak containing DDS texture files (typically "Textures.pak")
    pub textures: String,
    /// Pak containing GTP virtual texture files (typically "VirtualTextures.pak")
    pub virtual_textures: String,
    /// Pattern for deriving GTP path from gtex_hash
    /// e.g., "Generated/Public/VirtualTextures/Albedo_Normal_Physical_{hash[0]}_{hash}.gtp"
    pub gtp_path_pattern: String,
}

impl PakPaths {
    /// Default BG3 pak paths
    pub fn bg3_default() -> Self {
        Self {
            models: "Models.pak".to_string(),
            textures: "Textures.pak".to_string(),
            virtual_textures: "VirtualTextures.pak".to_string(),
            gtp_path_pattern: "Generated/Public/VirtualTextures/Albedo_Normal_Physical_{first}_{hash}.gtp".to_string(),
        }
    }

    /// Derive the GTP path from a gtex hash
    pub fn gtp_path_from_hash(&self, gtex_hash: &str) -> String {
        if gtex_hash.is_empty() {
            return String::new();
        }
        let first = &gtex_hash[0..1];
        self.gtp_path_pattern
            .replace("{first}", first)
            .replace("{hash}", gtex_hash)
    }
}

/// A material definition from MaterialBank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDef {
    /// Material resource ID (GUID)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Path to base material template file (e.g., "Public/Shared/Assets/Materials/...")
    pub source_file: String,
    /// Pak file where this material definition was found (e.g., "Shared.pak")
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_pak: String,
    /// Texture parameter IDs (GUID references to TextureBank)
    pub texture_ids: Vec<TextureParam>,
    /// Virtual texture parameter IDs (GUID references to VirtualTextureBank)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub virtual_texture_ids: Vec<String>,
}

/// A texture parameter within a material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureParam {
    /// Parameter name (e.g., "MSKColor", "NormalMap")
    pub name: String,
    /// Texture resource ID (GUID)
    pub texture_id: String,
}

/// The complete asset database built from a _merged.lsx file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MergedDatabase {
    /// Source file this database was built from
    pub source_path: String,
    /// Pak file paths for resolving assets
    pub pak_paths: PakPaths,
    /// Visual assets indexed by their ID (GUID)
    pub visuals_by_id: HashMap<String, VisualAsset>,
    /// Visual ID indexed by visual name (e.g., "HUM_M_ARM_Robe_C_Bracers_1" -> ID)
    pub visuals_by_name: HashMap<String, String>,
    /// Visual IDs indexed by GR2 filename (one GR2 can have multiple visuals)
    pub visuals_by_gr2: HashMap<String, Vec<String>>,
    /// Materials indexed by their ID
    pub materials: HashMap<String, MaterialDef>,
    /// Textures indexed by their ID
    pub textures: HashMap<String, TextureRef>,
    /// Virtual textures indexed by their ID
    pub virtual_textures: HashMap<String, VirtualTextureRef>,
}

impl MergedDatabase {
    pub fn new(source_path: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            pak_paths: PakPaths::bg3_default(),
            ..Default::default()
        }
    }

    /// Get a visual asset by its exact name (e.g., "HUM_M_ARM_Robe_C_Bracers_1")
    pub fn get_by_visual_name(&self, visual_name: &str) -> Option<&VisualAsset> {
        let id = self.visuals_by_name.get(visual_name)?;
        self.visuals_by_id.get(id)
    }

    /// Get all visuals that use a specific GR2 file
    pub fn get_visuals_for_gr2(&self, gr2_name: &str) -> Vec<&VisualAsset> {
        // Try exact match first
        let ids = self.visuals_by_gr2.get(gr2_name).or_else(|| {
            // Try just the filename
            let filename = std::path::Path::new(gr2_name)
                .file_name()
                .and_then(|s| s.to_str())?;
            self.visuals_by_gr2.get(filename)
        });

        ids.map(|ids| {
            ids.iter()
                .filter_map(|id| self.visuals_by_id.get(id))
                .collect()
        })
        .unwrap_or_default()
    }

    /// Get all visual names in the database
    pub fn visual_names(&self) -> impl Iterator<Item = &str> {
        self.visuals_by_name.keys().map(|s| s.as_str())
    }

    /// Get all GR2 files in the database
    pub fn gr2_files(&self) -> impl Iterator<Item = &str> {
        self.visuals_by_gr2.keys().map(|s| s.as_str())
    }

    /// Get count statistics
    pub fn stats(&self) -> DatabaseStats {
        DatabaseStats {
            visual_count: self.visuals_by_id.len(),
            material_count: self.materials.len(),
            texture_count: self.textures.len(),
            virtual_texture_count: self.virtual_textures.len(),
        }
    }

    /// Import materials and textures from another database
    ///
    /// This is useful when one database (like Loot) references materials
    /// that are defined in another database (like Humans_Male_Armor).
    /// Only imports materials/textures that don't already exist.
    pub fn import_materials_from(&mut self, other: &MergedDatabase) {
        // Import materials that don't exist locally
        for (id, material) in &other.materials {
            if !self.materials.contains_key(id) {
                self.materials.insert(id.clone(), material.clone());
            }
        }

        // Import textures that don't exist locally
        for (id, texture) in &other.textures {
            if !self.textures.contains_key(id) {
                self.textures.insert(id.clone(), texture.clone());
            }
        }

        // Import virtual textures that don't exist locally
        for (id, vt) in &other.virtual_textures {
            if !self.virtual_textures.contains_key(id) {
                self.virtual_textures.insert(id.clone(), vt.clone());
            }
        }
    }

    /// Re-resolve texture references for all visuals using current materials/textures
    ///
    /// Call this after importing materials from external databases to populate
    /// the textures and virtual_textures fields on each visual.
    pub fn resolve_references(&mut self) {
        let materials = self.materials.clone();
        let textures = self.textures.clone();
        let virtual_textures = self.virtual_textures.clone();

        for visual in self.visuals_by_id.values_mut() {
            let mut resolved_textures = Vec::new();
            let mut resolved_vts = Vec::new();

            for mat_id in &visual.material_ids {
                if let Some(material) = materials.get(mat_id) {
                    // Resolve texture references
                    for tex_param in &material.texture_ids {
                        if let Some(texture) = textures.get(&tex_param.texture_id) {
                            let mut tex_ref = texture.clone();
                            tex_ref.parameter_name = Some(tex_param.name.clone());
                            if !resolved_textures.iter().any(|t: &TextureRef| t.id == tex_ref.id) {
                                resolved_textures.push(tex_ref);
                            }
                        }
                    }
                }
            }

            // Resolve virtual textures through material's virtual_texture_ids
            for mat_id in &visual.material_ids {
                if let Some(material) = materials.get(mat_id) {
                    for vt_id in &material.virtual_texture_ids {
                        if let Some(vt) = virtual_textures.get(vt_id) {
                            if !resolved_vts.iter().any(|v: &VirtualTextureRef| v.id == vt.id) {
                                resolved_vts.push(vt.clone());
                            }
                        }
                    }
                }
            }

            visual.textures = resolved_textures;
            visual.virtual_textures = resolved_vts;
        }
    }
}

/// Statistics about a merged database
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub visual_count: usize,
    pub material_count: usize,
    pub texture_count: usize,
    pub virtual_texture_count: usize,
}

/// A matched .gtp file from VirtualTextures.pak
#[derive(Debug, Clone)]
pub struct GtpMatch {
    /// The GTex hash that was searched for
    pub gtex_hash: String,
    /// Path to the .gtp file inside the pak
    pub gtp_path: String,
    /// Path to the VirtualTextures.pak file
    pub pak_path: PathBuf,
}
