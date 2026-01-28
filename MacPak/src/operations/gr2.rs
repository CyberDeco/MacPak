//! GR2 (Granny2) file operations
//!
//! Operations for working with GR2 3D model files used in BG3/DOS2.
//! Core functionality is in maclarian; this module provides re-exports and
//! any MacPak-specific wrappers.

use crate::error::Result;
use std::path::Path;

// Re-export types from maclarian for convenience
pub use maclarian::formats::gr2::{
    Gr2BoneInfo, Gr2Info, Gr2MeshInfo, Gr2ModelInfo, Gr2SkeletonInfo, SectionInfo,
    extract_gr2_info, inspect_gr2,
};

// Re-export progress types
pub use maclarian::converter::{Gr2Phase, Gr2Progress, Gr2ProgressCallback};

/// Convert a GR2 file to GLB (binary glTF) format.
pub fn gr2_to_glb(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::convert_gr2_to_glb(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

/// Convert a GR2 file to GLB with progress callback.
pub fn gr2_to_glb_with_progress(
    source: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    progress: Gr2ProgressCallback,
) -> Result<()> {
    maclarian::converter::convert_gr2_to_glb_with_progress(source.as_ref(), dest.as_ref(), progress)
        .map_err(|e| e.into())
}

/// Convert a glTF/GLB file to GR2 format.
///
/// Note: Currently outputs uncompressed GR2 files.
pub fn gltf_to_gr2(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::convert_gltf_to_gr2(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

/// Convert a glTF/GLB file to GR2 with progress callback.
pub fn gltf_to_gr2_with_progress(
    source: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    progress: Gr2ProgressCallback,
) -> Result<()> {
    maclarian::converter::convert_gltf_to_gr2_with_progress(
        source.as_ref(),
        dest.as_ref(),
        progress,
    )
    .map_err(|e| e.into())
}

/// Decompress all BitKnit-compressed sections in a GR2 file.
pub fn decompress_gr2(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    let data = std::fs::read(source.as_ref())?;
    let decompressed = maclarian::formats::gr2::decompress_gr2(&data)?;
    std::fs::write(dest.as_ref(), decompressed)?;
    Ok(())
}
