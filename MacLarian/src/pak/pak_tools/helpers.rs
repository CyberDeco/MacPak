//! Helper utilities for PAK operations

use std::path::{Path, PathBuf};

/// Get the path for a specific archive part file
///
/// For part 0, returns the base path unchanged.
/// For part N > 0, returns `{stem}_{N}.{ext}` (e.g., `Textures_1.pak`)
pub fn get_part_path(base_path: &Path, part: u8) -> Option<PathBuf> {
    if part == 0 {
        return Some(base_path.to_path_buf());
    }

    let stem = base_path.file_stem()?.to_str()?;
    let ext = base_path.extension()?.to_str()?;
    let parent = base_path.parent()?;

    Some(parent.join(format!("{stem}_{part}.{ext}")))
}

/// Check if a filename is a virtual texture file (.gts or .gtp)
pub fn is_virtual_texture_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".gts") || lower.ends_with(".gtp")
}

/// Extract the subfolder name for a virtual texture file
/// e.g., "`Albedo_Normal_Physical_0.gts`" -> "`Albedo_Normal_Physical_0`"
/// e.g., "`Albedo_Normal_Physical_0_abc123def.gtp`" -> "`Albedo_Normal_Physical_0`"
pub fn get_virtual_texture_subfolder(filename: &str) -> Option<String> {
    let stem = filename.strip_suffix(".gts")
        .or_else(|| filename.strip_suffix(".gtp"))
        .or_else(|| filename.strip_suffix(".GTS"))
        .or_else(|| filename.strip_suffix(".GTP"))?;

    // For .gts files, the stem is already the subfolder name
    // e.g., "Albedo_Normal_Physical_0" from "Albedo_Normal_Physical_0.gts"
    if filename.to_lowercase().ends_with(".gts") {
        return Some(stem.to_string());
    }

    // For .gtp files, strip the hash suffix
    // e.g., "Albedo_Normal_Physical_0_abc123...def" -> "Albedo_Normal_Physical_0"
    if let Some(last_underscore) = stem.rfind('_') {
        let suffix = &stem[last_underscore + 1..];
        // Hash is 32 hex characters
        if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(stem[..last_underscore].to_string());
        }
    }

    // Fallback: use the full stem
    Some(stem.to_string())
}
