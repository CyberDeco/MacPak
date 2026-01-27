//! Types for DDS/PNG image conversion progress tracking
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`
//!
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

// ============================================================================
// Progress Types
// ============================================================================

/// Progress callback type for image conversion operations
pub type ImageProgressCallback<'a> = &'a (dyn Fn(&ImageProgress) + Sync + Send);

/// Progress information during image conversion operations
#[derive(Debug, Clone)]
pub struct ImageProgress {
    /// Current operation phase
    pub phase: ImagePhase,
    /// Current item number (1-indexed)
    pub current: usize,
    /// Total number of items
    pub total: usize,
    /// Current file or item being processed (if applicable)
    pub current_file: Option<String>,
}

impl ImageProgress {
    /// Create a new progress update
    #[must_use]
    pub fn new(phase: ImagePhase, current: usize, total: usize) -> Self {
        Self {
            phase,
            current,
            total,
            current_file: None,
        }
    }

    /// Create a progress update with a file/item name
    #[must_use]
    pub fn with_file(phase: ImagePhase, current: usize, total: usize, file: impl Into<String>) -> Self {
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

/// Phase of image conversion operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImagePhase {
    /// Reading the input file
    ReadingFile,
    /// Decoding the source format
    Decoding,
    /// Encoding to target format
    Encoding,
    /// Writing the output file
    WritingFile,
    /// Operation complete
    Complete,
}

impl ImagePhase {
    /// Get a human-readable description of this phase
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadingFile => "Reading file",
            Self::Decoding => "Decoding",
            Self::Encoding => "Encoding",
            Self::WritingFile => "Writing file",
            Self::Complete => "Complete",
        }
    }
}
