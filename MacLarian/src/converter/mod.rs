//! Format conversion utilities
//!
//! This module handles conversions between different Larian file formats:
//! - LSF (binary) ↔ LSX (XML) ↔ LSJ (JSON) - Document formats
//! - LOCA ↔ XML - Localization formats
//! - GR2 (Granny2) ↔ glTF - 3D model conversion
//! - DDS ↔ PNG - Texture conversion

mod dds_png;
pub mod gr2_gltf;
pub mod loca;
pub(crate) mod lsf_lsx_lsj;

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
    pub fn with_file(
        phase: ConvertPhase,
        current: usize,
        total: usize,
        file: impl Into<String>,
    ) -> Self {
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
#[non_exhaustive]
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
    convert_lsf_to_lsj,
    // With-progress variants
    convert_lsf_to_lsj_with_progress,
    convert_lsf_to_lsx,
    convert_lsf_to_lsx_with_progress,
    convert_lsj_to_lsf,
    convert_lsj_to_lsf_with_progress,
    convert_lsj_to_lsx,
    convert_lsj_to_lsx_with_progress,
    convert_lsx_to_lsf,
    convert_lsx_to_lsf_with_progress,
    convert_lsx_to_lsj,
    convert_lsx_to_lsj_with_progress,
    // In-memory conversion functions
    from_lsx,
    // Convenience aliases (shorter names)
    lsf_to_lsj,
    // Convenience aliases with progress
    lsf_to_lsj_with_progress,
    lsf_to_lsx,
    lsf_to_lsx_with_progress,
    lsj_to_lsf,
    lsj_to_lsf_with_progress,
    lsj_to_lsx,
    lsj_to_lsx_with_progress,
    lsx_to_lsf,
    lsx_to_lsf_with_progress,
    lsx_to_lsj,
    lsx_to_lsj_with_progress,
    to_lsj,
    to_lsx,
};

// GR2/glTF conversion exports
pub use gr2_gltf::{Gr2Phase, Gr2Progress, Gr2ProgressCallback};
pub use gr2_gltf::{convert_gltf_bytes_to_gr2, convert_gltf_to_gr2};
pub use gr2_gltf::{
    convert_gltf_bytes_to_gr2_with_progress, convert_gltf_to_gr2_with_progress,
    convert_gr2_bytes_to_glb_with_progress, convert_gr2_to_glb_with_progress,
    convert_gr2_to_gltf_with_progress,
};
pub use gr2_gltf::{convert_gr2_bytes_to_glb, convert_gr2_to_glb, convert_gr2_to_gltf};

// LOCA conversion exports
pub use loca::{
    convert_loca_to_xml, convert_loca_to_xml_with_progress, convert_xml_to_loca,
    convert_xml_to_loca_with_progress, loca_from_xml, loca_to_xml_string,
};

// DDS/PNG conversion exports
pub use dds_png::{
    DdsFormat, ImagePhase, ImageProgress, ImageProgressCallback, convert_dds_to_png,
    convert_dds_to_png_with_progress, convert_png_to_dds, convert_png_to_dds_with_format,
    convert_png_to_dds_with_format_and_progress, convert_png_to_dds_with_progress,
    dds_bytes_to_png_bytes, png_image_to_dds_bytes,
};
