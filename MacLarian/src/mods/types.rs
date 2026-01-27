//! Types for mod operations progress tracking

/// Progress callback type for mod operations
pub type ModProgressCallback<'a> = &'a (dyn Fn(&ModProgress) + Sync + Send);

/// Progress information during mod operations
#[derive(Debug, Clone)]
pub struct ModProgress {
    /// Current operation phase
    pub phase: ModPhase,
    /// Current item number (1-indexed)
    pub current: usize,
    /// Total number of items
    pub total: usize,
    /// Current file being processed (if applicable)
    pub current_file: Option<String>,
}

impl ModProgress {
    /// Create a new progress update
    #[must_use]
    pub fn new(phase: ModPhase, current: usize, total: usize) -> Self {
        Self {
            phase,
            current,
            total,
            current_file: None,
        }
    }

    /// Create a progress update with a file/item name
    #[must_use]
    pub fn with_file(phase: ModPhase, current: usize, total: usize, file: impl Into<String>) -> Self {
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

/// Phase of mod operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModPhase {
    /// Validating mod structure
    Validating,
    /// Calculating MD5 hash of PAK file
    CalculatingHash,
    /// Generating info.json content
    GeneratingJson,
    /// Operation complete
    Complete,
}

impl ModPhase {
    /// Get a human-readable description of this phase
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Validating => "Validating structure",
            Self::CalculatingHash => "Calculating MD5 hash",
            Self::GeneratingJson => "Generating info.json",
            Self::Complete => "Complete",
        }
    }
}
