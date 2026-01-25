//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! Format conversion utilities
//!
//! This module handles conversions between different Larian file formats:
//! - LSF (binary) ↔ LSX (XML) ↔ LSJ (JSON) - Document formats
//! - LOCA ↔ XML - Localization formats
//! - GR2 (Granny2) ↔ glTF - 3D model conversion
//! - DDS ↔ PNG - Texture conversion

pub mod lsf_lsx_lsj;
pub mod loca;
pub mod gr2_gltf;
mod dds_png;

/// Progress callback type for conversion operations.
/// The callback receives a message describing the current step.
pub type ProgressCallback<'a> = &'a dyn Fn(&str);

// Re-export LSF/LSX/LSJ conversions from subdirectory
pub use lsf_lsx_lsj::{
    // Primary conversion functions
    convert_lsf_to_lsx, convert_lsx_to_lsf,
    convert_lsx_to_lsj, convert_lsj_to_lsx,
    convert_lsf_to_lsj, convert_lsj_to_lsf,
    // With-progress variants
    convert_lsx_to_lsj_with_progress, convert_lsj_to_lsx_with_progress,
    convert_lsf_to_lsj_with_progress, convert_lsj_to_lsf_with_progress,
    // Helper functions
    to_lsx, from_lsx, to_lsj, lsj_to_lsx_doc,
    // Convenience aliases
    lsf_to_lsx, lsx_to_lsf, lsx_to_lsj, lsj_to_lsx, lsf_to_lsj, lsj_to_lsf,
    // Convenience aliases with progress
    lsx_to_lsj_with_progress, lsj_to_lsx_with_progress,
    lsf_to_lsj_with_progress, lsj_to_lsf_with_progress,
};

// GR2/glTF conversion exports
pub use gr2_gltf::{convert_gr2_to_glb, convert_gr2_to_gltf, convert_gr2_bytes_to_glb};
pub use gr2_gltf::{convert_gltf_to_gr2, convert_gltf_bytes_to_gr2};

// LOCA conversion exports
pub use loca::{convert_loca_to_xml, loca_to_xml_string, convert_xml_to_loca, loca_from_xml};

// DDS/PNG conversion exports
pub use dds_png::{
    convert_dds_to_png, convert_png_to_dds, convert_png_to_dds_with_format,
    dds_bytes_to_png_bytes, png_image_to_dds_bytes, DdsFormat,
};
