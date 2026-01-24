//! Extraction options for GR2 file processing
//!
//! This module provides configuration options for smart extraction operations
//! that can automatically convert GR2 files and extract associated textures.

use std::path::PathBuf;

/// Options for GR2 file processing during extraction.
///
/// When extracting GR2 files from PAK archives, these options control
/// automatic post-processing steps like conversion to GLB and texture extraction.
///
/// # Example
///
/// ```no_run
/// use maclarian::pak::Gr2ExtractionOptions;
///
/// // Enable all GR2 processing options (bundle mode)
/// let options = Gr2ExtractionOptions::bundle();
///
/// // Or configure individually
/// let options = Gr2ExtractionOptions::new()
///     .with_convert_to_glb(true)
///     .with_extract_textures(true)
///     .with_keep_original(false);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Gr2ExtractionOptions {
    /// Convert GR2 files to GLB format after extraction
    pub convert_to_glb: bool,

    /// Extract DDS textures associated with each GR2 file
    pub extract_textures: bool,

    /// Extract virtual textures (GTex files) associated with each GR2 file
    pub extract_virtual_textures: bool,

    /// Path to the game's Data folder (containing Shared.pak, Textures.pak, etc.)
    /// If None, auto-detection will be attempted.
    pub game_data_path: Option<PathBuf>,

    /// Path to the Virtual Textures folder (containing .gts/.gtp files)
    /// If None, auto-detection will be attempted based on game_data_path.
    pub virtual_textures_path: Option<PathBuf>,

    /// Keep the original GR2 file after conversion to GLB
    /// Default: true (keep original)
    pub keep_original_gr2: bool,

    /// Convert extracted DDS textures to PNG format
    /// Default: false (keep as DDS)
    pub convert_to_png: bool,
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
        }
    }

    /// Check if any GR2 processing options are enabled.
    #[must_use]
    pub fn has_gr2_processing(&self) -> bool {
        self.convert_to_glb || self.extract_textures || self.extract_virtual_textures
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

    /// Set the game data path.
    #[must_use]
    pub fn with_game_data_path(mut self, path: Option<PathBuf>) -> Self {
        self.game_data_path = path;
        self
    }

    /// Set the virtual textures path.
    #[must_use]
    pub fn with_virtual_textures_path(mut self, path: Option<PathBuf>) -> Self {
        self.virtual_textures_path = path;
        self
    }

    /// Set whether to keep the original GR2 after conversion.
    #[must_use]
    pub fn with_keep_original(mut self, keep: bool) -> Self {
        self.keep_original_gr2 = keep;
        self
    }

    /// Set whether to convert DDS textures to PNG.
    #[must_use]
    pub fn with_convert_to_png(mut self, convert: bool) -> Self {
        self.convert_to_png = convert;
        self
    }
}
