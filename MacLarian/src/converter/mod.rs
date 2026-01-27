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

pub(crate) mod lsf_lsx_lsj;
pub mod loca;
pub mod gr2_gltf;
mod dds_png;

/// Progress callback type for conversion operations
pub type ConvertProgressCallback<'a> = &'a (dyn Fn(&ConvertProgress) + Sync + Send);

/// Progress information during conversion operations
#[derive(Debug, Clone)]
pub struct ConvertProgress {
    /// Current operation phase
    pub phase: ConvertPhase,
    /// Current item number (1-indexed)
    pub current: usize,
    /// Total number of items
    pub total: usize,
    /// Current file or item being processed (if applicable)
    pub current_file: Option<String>,
}

impl ConvertProgress {
    /// Create a new progress update
    #[must_use]
    pub fn new(phase: ConvertPhase, current: usize, total: usize) -> Self {
        Self {
            phase,
            current,
            total,
            current_file: None,
        }
    }

    /// Create a progress update with a file/item name
    #[must_use]
    pub fn with_file(phase: ConvertPhase, current: usize, total: usize, file: impl Into<String>) -> Self {
        Self {
            phase,
            current,
            total,
            current_file: Some(file.into()),
        }
    }

    /// Get the progress percentage (0.0 - 1.0)
    #[must_use]
    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            self.current as f32 / self.total as f32
        }
    }
}

/// Phase of conversion operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertPhase {
    /// Reading source file
    ReadingSource,
    /// Parsing source format
    Parsing,
    /// Converting data structures
    Converting,
    /// Writing output file
    WritingOutput,
    /// Operation complete
    Complete,
}

impl ConvertPhase {
    /// Get a human-readable description of this phase
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadingSource => "Reading source",
            Self::Parsing => "Parsing",
            Self::Converting => "Converting",
            Self::WritingOutput => "Writing output",
            Self::Complete => "Complete",
        }
    }
}

// Re-export LSF/LSX/LSJ conversions - primary API only
pub use lsf_lsx_lsj::{
    // Primary conversion functions
    convert_lsf_to_lsx, convert_lsx_to_lsf,
    convert_lsx_to_lsj, convert_lsj_to_lsx,
    convert_lsf_to_lsj, convert_lsj_to_lsf,
    // With-progress variants
    convert_lsx_to_lsj_with_progress, convert_lsj_to_lsx_with_progress,
    convert_lsf_to_lsj_with_progress, convert_lsj_to_lsf_with_progress,
    // Convenience aliases (shorter names)
    lsf_to_lsx, lsx_to_lsf, lsx_to_lsj, lsj_to_lsx, lsf_to_lsj, lsj_to_lsf,
    // Convenience aliases with progress
    lsx_to_lsj_with_progress, lsj_to_lsx_with_progress,
    lsf_to_lsj_with_progress, lsj_to_lsf_with_progress,
    // In-memory conversion functions
    to_lsx, from_lsx, to_lsj,
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
