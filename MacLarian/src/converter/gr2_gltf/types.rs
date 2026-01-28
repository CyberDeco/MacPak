//! Types for GR2/glTF conversion progress tracking
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

// ============================================================================
// Progress Types
// ============================================================================

/// Progress callback type for GR2/glTF conversion operations
pub type Gr2ProgressCallback<'a> = &'a (dyn Fn(&Gr2Progress) + Sync + Send);

/// Progress information during GR2/glTF conversion operations
#[derive(Debug, Clone)]
pub struct Gr2Progress {
    /// Current operation phase
    pub phase: Gr2Phase,
    /// Current item number (1-indexed)
    pub current: usize,
    /// Total number of items
    pub total: usize,
    /// Current file or item being processed (if applicable)
    pub current_file: Option<String>,
}

impl Gr2Progress {
    /// Create a new progress update
    #[must_use]
    pub fn new(phase: Gr2Phase, current: usize, total: usize) -> Self {
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
        phase: Gr2Phase,
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

/// Phase of GR2/glTF conversion operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gr2Phase {
    // === GR2 -> glTF phases ===
    /// Reading the GR2 file
    ReadingFile,
    /// Parsing skeleton/bones from GR2
    ParsingSkeleton,
    /// Parsing mesh data from GR2
    ParsingMeshes,
    /// Building glTF document structure
    BuildingDocument,
    /// Writing glTF/GLB output
    WritingOutput,

    // === glTF -> GR2 phases ===
    /// Loading glTF/GLB file
    LoadingFile,
    /// Parsing glTF model data
    ParsingModel,
    /// Building GR2 data structures
    BuildingGr2,
    /// Writing GR2 file
    WritingFile,

    // === Common ===
    /// Operation complete
    Complete,
}

impl Gr2Phase {
    /// Get a human-readable description of this phase
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            // GR2 -> glTF
            Self::ReadingFile => "Reading file",
            Self::ParsingSkeleton => "Parsing skeleton",
            Self::ParsingMeshes => "Parsing meshes",
            Self::BuildingDocument => "Building document",
            Self::WritingOutput => "Writing output",
            // glTF -> GR2
            Self::LoadingFile => "Loading file",
            Self::ParsingModel => "Parsing model",
            Self::BuildingGr2 => "Building GR2",
            Self::WritingFile => "Writing file",
            // Common
            Self::Complete => "Complete",
        }
    }
}
