//! Texture loading helpers for GR2 to glTF conversion.

use std::collections::HashSet;
use std::path::Path;
use crate::converter::dds_png::dds_bytes_to_png_bytes;
use crate::virtual_texture::VirtualTextureExtractor;
use crate::merged::{GameDataResolver, MergedDatabase};
use crate::pak::PakOperations;
use super::gltf::GltfBuilder;

/// Texture type for glTF PBR mapping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum TextureType {
    Albedo,
    Normal,
    Physical,
    Other,
}

/// Load textures for a GR2 file and add them to the builder.
/// Returns the material index if textures were successfully loaded.
pub(super) fn load_textures_for_gr2(
    gr2_filename: &str,
    textures_pak_path: &Path,
    builder: &mut GltfBuilder,
    warnings: &mut Vec<String>,
) -> Option<usize> {
    tracing::debug!("Loading textures for GR2: {}", gr2_filename);

    // Build resolver from game data path (parent of Textures.pak)
    let game_data_path = textures_pak_path.parent()?;
    let resolver = match GameDataResolver::new(game_data_path) {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("Could not create resolver: {e}"));
            return None;
        }
    };
    let db = resolver.database();

    // Look up visuals for this GR2
    let visuals = db.get_visuals_for_gr2(gr2_filename);
    tracing::debug!("Found {} visuals for GR2", visuals.len());

    if visuals.is_empty() {
        warnings.push(format!("No texture info found in database for: {gr2_filename}"));
        return None;
    }

    // Track texture indices
    let mut albedo_texture_idx: Option<usize> = None;
    let mut normal_texture_idx: Option<usize> = None;
    let mut physical_texture_idx: Option<usize> = None;

    // First, try to load regular DDS textures
    let regular_result = load_regular_textures(&visuals, textures_pak_path, builder, warnings);
    if let Some((albedo, normal, physical)) = regular_result {
        albedo_texture_idx = albedo;
        normal_texture_idx = normal;
        physical_texture_idx = physical;
    }

    // If no PBR textures found, try virtual textures
    if albedo_texture_idx.is_none() && normal_texture_idx.is_none() && physical_texture_idx.is_none() {
        tracing::debug!("No regular PBR textures found, trying virtual textures");

        // Find VirtualTextures.pak (sibling to Textures.pak)
        if let Some(parent) = textures_pak_path.parent() {
            let vt_pak_path = parent.join("VirtualTextures.pak");
            if vt_pak_path.exists() {
                if let Some((albedo, normal, physical)) = load_virtual_textures(&visuals, &vt_pak_path, db, builder, warnings) {
                    albedo_texture_idx = albedo;
                    normal_texture_idx = normal;
                    physical_texture_idx = physical;
                }
            } else {
                tracing::debug!("VirtualTextures.pak not found at {}", vt_pak_path.display());
            }
        }
    }

    tracing::debug!(
        "Final texture indices - Albedo: {:?}, Normal: {:?}, Physical: {:?}",
        albedo_texture_idx, normal_texture_idx, physical_texture_idx
    );

    // Create material if there's at least one texture
    if albedo_texture_idx.is_some() || normal_texture_idx.is_some() || physical_texture_idx.is_some() {
        let material_idx = builder.add_material(
            Some(gr2_filename.to_string()),
            albedo_texture_idx,
            normal_texture_idx,
            physical_texture_idx,
            None, // occlusion - not separately accounted for
        );
        tracing::debug!("Created material with index {}", material_idx);
        Some(material_idx)
    } else {
        warnings.push("No usable textures (Albedo/Normal/Physical) found".to_string());
        None
    }
}

/// Load regular DDS textures from Textures.pak.
fn load_regular_textures(
    visuals: &[&crate::merged::VisualAsset],
    textures_pak_path: &Path,
    builder: &mut GltfBuilder,
    warnings: &mut Vec<String>,
) -> Option<(Option<usize>, Option<usize>, Option<usize>)> {
    // Collect unique textures from all visuals
    let mut seen_paths: HashSet<String> = HashSet::new();
    let mut textures_to_load: Vec<(&str, &str)> = Vec::new(); // (dds_path, parameter_name)

    for visual in visuals {
        tracing::debug!("Visual '{}' has {} regular textures", visual.name, visual.textures.len());
        for texture in &visual.textures {
            if seen_paths.insert(texture.dds_path.clone()) {
                let param_name = texture.parameter_name.as_deref().unwrap_or("");
                tracing::debug!("  Texture: {} (param: {})", texture.dds_path, param_name);
                textures_to_load.push((&texture.dds_path, param_name));
            }
        }
    }

    if textures_to_load.is_empty() {
        return None;
    }

    // Check if Textures.pak exists
    if !textures_pak_path.exists() {
        warnings.push(format!("Textures.pak not found at: {}", textures_pak_path.display()));
        return None;
    }

    // Read DDS files from PAK
    let dds_paths: Vec<&str> = textures_to_load.iter().map(|(p, _)| *p).collect();
    let dds_bytes = match PakOperations::read_files_bytes(textures_pak_path, &dds_paths) {
        Ok(bytes) => {
            tracing::debug!("Read {} DDS files from PAK", bytes.len());
            bytes
        }
        Err(e) => {
            warnings.push(format!("Failed to read textures from PAK: {e}"));
            return None;
        }
    };

    if dds_bytes.is_empty() {
        return None;
    }

    // Convert DDS to PNG and categorize by type
    let mut albedo_texture_idx: Option<usize> = None;
    let mut normal_texture_idx: Option<usize> = None;
    let mut physical_texture_idx: Option<usize> = None;

    for (dds_path, param_name) in &textures_to_load {
        let Some(dds_data) = dds_bytes.get(*dds_path) else {
            continue;
        };

        tracing::debug!("Converting DDS: {} ({} bytes)", dds_path, dds_data.len());

        // Convert DDS to PNG
        let png_data = match dds_bytes_to_png_bytes(dds_data) {
            Ok(png) => {
                tracing::debug!("  -> PNG: {} bytes", png.len());
                png
            }
            Err(e) => {
                tracing::debug!("  -> Failed to convert: {}", e);
                continue;
            }
        };

        // Determine texture type from parameter name or filename
        let texture_type = categorize_texture(dds_path, param_name);
        tracing::debug!("  -> Type: {:?}", texture_type);

        // Add to builder and track index
        let texture_idx = builder.add_image_as_texture(&png_data, Some((*dds_path).to_string()));

        match texture_type {
            TextureType::Albedo => albedo_texture_idx = Some(texture_idx),
            TextureType::Normal => normal_texture_idx = Some(texture_idx),
            TextureType::Physical => physical_texture_idx = Some(texture_idx),
            TextureType::Other => {
                tracing::debug!("  -> Skipped (non-PBR texture type)");
            }
        }
    }

    // Only return if there's at least one PBR texture
    if albedo_texture_idx.is_some() || normal_texture_idx.is_some() || physical_texture_idx.is_some() {
        Some((albedo_texture_idx, normal_texture_idx, physical_texture_idx))
    } else {
        None
    }
}

/// Load virtual textures from VirtualTextures.pak.
fn load_virtual_textures(
    visuals: &[&crate::merged::VisualAsset],
    vt_pak_path: &Path,
    db: &MergedDatabase,
    builder: &mut GltfBuilder,
    warnings: &mut Vec<String>,
) -> Option<(Option<usize>, Option<usize>, Option<usize>)> {
    // Collect virtual textures from visuals
    let mut vt_hashes: Vec<&str> = Vec::new();
    for visual in visuals {
        tracing::debug!("Visual '{}' has {} virtual textures", visual.name, visual.virtual_textures.len());
        for vt in &visual.virtual_textures {
            if !vt.gtex_hash.is_empty() {
                tracing::debug!("  Virtual texture: {} (hash: {})", vt.name, vt.gtex_hash);
                vt_hashes.push(&vt.gtex_hash);
            }
        }
    }

    if vt_hashes.is_empty() {
        tracing::debug!("No virtual textures found");
        return None;
    }

    // Use first virtual texture (they typically contain all layers)
    let hash = vt_hashes[0];
    let gtp_path = db.pak_paths.gtp_path_from_hash(hash);
    let gts_path = derive_gts_path_from_gtp(&gtp_path);

    tracing::debug!("Virtual texture paths: GTP={}, GTS={}", gtp_path, gts_path);

    // Extract GTP and GTS from PAK to temp directory
    let temp_dir = std::env::temp_dir().join(format!("macpak_vt_{hash}"));
    let _ = std::fs::create_dir_all(&temp_dir);

    // Read files from VirtualTextures.pak
    let files_to_read = vec![gtp_path.as_str(), gts_path.as_str()];
    let file_bytes = match PakOperations::read_files_bytes(vt_pak_path, &files_to_read) {
        Ok(bytes) => bytes,
        Err(e) => {
            warnings.push(format!("Failed to read virtual textures from PAK: {e}"));
            return None;
        }
    };

    let gtp_data = file_bytes.get(&gtp_path)?;
    let gts_data = file_bytes.get(&gts_path)?;

    tracing::debug!("Read GTP ({} bytes) and GTS ({} bytes)", gtp_data.len(), gts_data.len());

    // Write to temp files for the extractor
    // VirtualTextureExtractor expects filename format "SomeName_{hash}.gtp" so use full basename
    let gtp_fallback = format!("{hash}.gtp");
    let gtp_filename = std::path::Path::new(&gtp_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&gtp_fallback);
    let gts_filename = std::path::Path::new(&gts_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("texture.gts");
    let temp_gtp = temp_dir.join(gtp_filename);
    let temp_gts = temp_dir.join(gts_filename);

    if std::fs::write(&temp_gtp, gtp_data).is_err() {
        warnings.push("Failed to write temp GTP file".to_string());
        return None;
    }
    if std::fs::write(&temp_gts, gts_data).is_err() {
        warnings.push("Failed to write temp GTS file".to_string());
        return None;
    }

    // Extract virtual textures
    if let Err(e) = VirtualTextureExtractor::extract_with_gts(&temp_gtp, &temp_gts, &temp_dir) {
        warnings.push(format!("Failed to extract virtual textures: {e}"));
        let _ = std::fs::remove_dir_all(&temp_dir);
        return None;
    }

    // Read and convert the extracted DDS files
    let mut albedo_texture_idx: Option<usize> = None;
    let mut normal_texture_idx: Option<usize> = None;
    let mut physical_texture_idx: Option<usize> = None;

    for (layer_name, texture_slot) in [("Albedo", &mut albedo_texture_idx), ("Normal", &mut normal_texture_idx), ("Physical", &mut physical_texture_idx)] {
        let dds_path = temp_dir.join(format!("{layer_name}.dds"));
        if dds_path.exists()
            && let Ok(dds_data) = std::fs::read(&dds_path) {
                tracing::debug!("Converting virtual texture {}.dds ({} bytes)", layer_name, dds_data.len());
                match dds_bytes_to_png_bytes(&dds_data) {
                    Ok(png_data) => {
                        tracing::debug!("  -> PNG: {} bytes", png_data.len());
                        let idx = builder.add_image_as_texture(&png_data, Some(format!("{layer_name}.dds")));
                        *texture_slot = Some(idx);
                    }
                    Err(e) => {
                        warnings.push(format!("Failed to convert {layer_name}.dds: {e}"));
                    }
                }
            }
    }

    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Return if there's at least one texture
    if albedo_texture_idx.is_some() || normal_texture_idx.is_some() || physical_texture_idx.is_some() {
        Some((albedo_texture_idx, normal_texture_idx, physical_texture_idx))
    } else {
        None
    }
}

/// Derive GTS path from GTP path.
fn derive_gts_path_from_gtp(gtp_path: &str) -> String {
    // GTP: "Generated/Public/VirtualTextures/Albedo_Normal_Physical_[0-9|a-f]_[GTex].gtp"
    // GTS: "Generated/Public/VirtualTextures/Albedo_Normal_Physical_[0-9|a-f].gts"
    let without_ext = gtp_path.trim_end_matches(".gtp");

    if let Some(last_underscore) = without_ext.rfind('_') {
        let suffix = &without_ext[last_underscore + 1..];
        if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            return format!("{}.gts", &without_ext[..last_underscore]);
        }
    }

    format!("{without_ext}.gts")
}

/// Categorize a texture by its parameter name or filename.
fn categorize_texture(dds_path: &str, param_name: &str) -> TextureType {
    let path_lower = dds_path.to_lowercase();
    let param_lower = param_name.to_lowercase();

    // Check parameter name first (more reliable)
    // Database uses: basecolor, normalmap, physicalmap, mskcolor
    if param_lower == "basecolor" || param_lower.contains("albedo") || param_lower.contains("diffuse") {
        return TextureType::Albedo;
    }
    if param_lower == "normalmap" || param_lower.contains("normal") {
        return TextureType::Normal;
    }
    if param_lower == "physicalmap" || param_lower.contains("physical") || param_lower.contains("metallic") || param_lower.contains("roughness") {
        return TextureType::Physical;
    }
    if param_lower == "mskcolor" || param_lower.contains("msk") {
        return TextureType::Other; // Skip dye masks
    }

    // Fall back to filename patterns (BG3 naming: _BMA = basecolor, _NM = normal, _PM = physical)
    if path_lower.contains("_bma.") || path_lower.contains("_bm.") || path_lower.contains("_albedo") || path_lower.contains("_diffuse") {
        return TextureType::Albedo;
    }
    if path_lower.contains("_nm.") || path_lower.contains("_normal") {
        return TextureType::Normal;
    }
    if path_lower.contains("_pm.") || path_lower.contains("_physical") {
        return TextureType::Physical;
    }
    if path_lower.contains("_msk") {
        return TextureType::Other; // Skip dye masks
    }

    // Default to Other for unknown types
    TextureType::Other
}
