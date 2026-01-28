//! Types for GR2 extraction operations

use std::path::PathBuf;

// ============================================================================
// Progress Types
// ============================================================================

/// Progress callback type for GR2 extraction operations
pub type Gr2ExtractionProgressCallback<'a> = &'a (dyn Fn(&Gr2ExtractionProgress) + Sync + Send);

/// Progress information during GR2 extraction operations
#[derive(Debug, Clone)]
pub struct Gr2ExtractionProgress {
    /// Current operation phase
    pub phase: Gr2ExtractionPhase,
    /// Current item number (1-indexed)
    pub current: usize,
    /// Total number of items
    pub total: usize,
    /// Current file being processed (if applicable)
    pub current_file: Option<String>,
}

impl Gr2ExtractionProgress {
    /// Create a new progress update
    #[must_use]
    pub fn new(phase: Gr2ExtractionPhase, current: usize, total: usize) -> Self {
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
        phase: Gr2ExtractionPhase,
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

/// Phase of GR2 extraction operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gr2ExtractionPhase {
    /// Converting GR2 to GLB format
    ConvertingGr2,
    /// Building texture database from merged files
    BuildingDatabase,
    /// Looking up textures for GR2 file
    LookingUpTextures,
    /// Extracting DDS textures from PAK
    ExtractingDdsTextures,
    /// Extracting virtual textures
    ExtractingVirtualTextures,
    /// Converting DDS to PNG format
    ConvertingToPng,
    /// Operation complete
    Complete,
}

impl Gr2ExtractionPhase {
    /// Get a human-readable description of this phase
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConvertingGr2 => "Converting GR2 to GLB",
            Self::BuildingDatabase => "Building texture database",
            Self::LookingUpTextures => "Looking up textures",
            Self::ExtractingDdsTextures => "Extracting DDS textures",
            Self::ExtractingVirtualTextures => "Extracting virtual textures",
            Self::ConvertingToPng => "Converting to PNG",
            Self::Complete => "Complete",
        }
    }
}

// ============================================================================
// Result and Options Types
// ============================================================================

/// Result of a smart GR2 extraction
#[derive(Debug, Clone)]
pub struct Gr2ExtractionResult {
    /// Path to the extracted GR2 file
    pub gr2_path: PathBuf,
    /// Path to the converted GLB file (if conversion succeeded)
    pub glb_path: Option<PathBuf>,
    /// Paths to extracted texture files
    pub texture_paths: Vec<PathBuf>,
    /// Any warnings or errors that occurred during extraction
    pub warnings: Vec<String>,
}

/// Options for smart GR2 extraction
#[derive(Debug, Clone)]
pub struct Gr2ExtractionOptions {
    /// Convert GR2 to GLB automatically
    pub convert_to_glb: bool,
    /// Extract associated textures
    pub extract_textures: bool,
    /// Extract virtual textures (GTex files) associated with each GR2 file
    pub extract_virtual_textures: bool,
    /// Path to BG3 install folder (for finding Textures.pak, etc.)
    /// If None, auto-detects using GameDataResolver
    pub game_data_path: Option<PathBuf>,
    /// Path to pre-extracted virtual textures (GTP/GTS files)
    /// If None, virtual textures will be skipped
    pub virtual_textures_path: Option<PathBuf>,
    /// Keep the original GR2 file after conversion to GLB (default: true)
    pub keep_original_gr2: bool,
    /// Convert extracted DDS textures to PNG format
    pub convert_to_png: bool,
    /// Keep the original DDS files when converting to PNG
    pub keep_original_dds: bool,
}

impl Default for Gr2ExtractionOptions {
    fn default() -> Self {
        Self {
            convert_to_glb: true,
            extract_textures: true,
            extract_virtual_textures: false,
            game_data_path: None,
            virtual_textures_path: None,
            keep_original_gr2: true,
            convert_to_png: false,
            keep_original_dds: false,
        }
    }
}

impl Gr2ExtractionOptions {
    /// Create new options with all processing disabled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            convert_to_glb: false,
            extract_textures: false,
            extract_virtual_textures: false,
            game_data_path: None,
            virtual_textures_path: None,
            keep_original_gr2: true,
            convert_to_png: false,
            keep_original_dds: false,
        }
    }

    /// Create options with all GR2 processing enabled (bundle mode).
    ///
    /// This is equivalent to the `--bundle` CLI flag.
    #[must_use]
    pub fn bundle() -> Self {
        Self {
            convert_to_glb: true,
            extract_textures: true,
            extract_virtual_textures: true,
            game_data_path: None,
            virtual_textures_path: None,
            keep_original_gr2: true,
            convert_to_png: false,
            keep_original_dds: false,
        }
    }

    /// Check if any GR2 processing options are enabled.
    #[must_use]
    pub fn has_gr2_processing(&self) -> bool {
        self.convert_to_glb || self.extract_textures || self.extract_virtual_textures
    }

    /// Create options with custom game data path
    #[must_use]
    pub fn with_game_data_path<P: Into<PathBuf>>(mut self, path: Option<P>) -> Self {
        self.game_data_path = path.map(Into::into);
        self
    }

    /// Set path to pre-extracted virtual textures (GTP/GTS files)
    #[must_use]
    pub fn with_virtual_textures_path<P: Into<PathBuf>>(mut self, path: Option<P>) -> Self {
        self.virtual_textures_path = path.map(Into::into);
        self
    }

    /// Disable GLB conversion
    #[must_use]
    pub fn no_conversion(mut self) -> Self {
        self.convert_to_glb = false;
        self
    }

    /// Disable texture extraction
    #[must_use]
    pub fn no_textures(mut self) -> Self {
        self.extract_textures = false;
        self
    }

    /// Enable PNG conversion for extracted DDS textures
    #[must_use]
    pub fn with_png_conversion(mut self, convert: bool) -> Self {
        self.convert_to_png = convert;
        self
    }

    /// Alias for [`Self::with_png_conversion`]
    #[must_use]
    pub fn with_convert_to_png(self, convert: bool) -> Self {
        self.with_png_conversion(convert)
    }

    /// Set whether to convert GR2 to GLB.
    #[must_use]
    pub fn with_convert_to_glb(mut self, convert: bool) -> Self {
        self.convert_to_glb = convert;
        self
    }

    /// Set whether to extract DDS textures.
    #[must_use]
    pub fn with_extract_textures(mut self, extract: bool) -> Self {
        self.extract_textures = extract;
        self
    }

    /// Set whether to extract virtual textures.
    #[must_use]
    pub fn with_extract_virtual_textures(mut self, extract: bool) -> Self {
        self.extract_virtual_textures = extract;
        self
    }

    /// Set whether to keep the original GR2 after conversion.
    #[must_use]
    pub fn with_keep_original(mut self, keep: bool) -> Self {
        self.keep_original_gr2 = keep;
        self
    }

    /// Set whether to keep original DDS files after PNG conversion.
    #[must_use]
    pub fn with_keep_original_dds(mut self, keep: bool) -> Self {
        self.keep_original_dds = keep;
        self
    }
}
