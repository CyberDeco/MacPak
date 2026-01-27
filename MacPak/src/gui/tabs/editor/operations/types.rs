//! Types and constants for file loading operations

/// Threshold for large file warning (50,000 lines for text, 5,000 nodes for LSF)
pub const LARGE_FILE_LINE_THRESHOLD: usize = 50_000;
pub const LARGE_LSF_NODE_THRESHOLD: usize = 5_000;

/// Result from background file loading - first phase (size check)
pub enum FileLoadPhase1 {
    /// Text file ready to display (small enough, no confirmation needed)
    Ready(FileLoadResult),
    /// LSF file needs conversion (always show progress)
    LsfNeedsConversion {
        path_str: String,
        format: String,
        node_count: usize,
        lsf_data: Vec<u8>,
        needs_warning: bool, // True if large file warning should be shown
    },
    /// LOCA file needs conversion (always show progress)
    LocaNeedsConversion {
        path_str: String,
        loca_data: Vec<u8>,
        needs_warning: bool, // True if result will be large
    },
    /// Large text file needs confirmation then formatting with progress
    TextNeedsConfirmation {
        result: FileLoadResult,
        filename: String,
    },
    /// Error occurred
    Error { path_str: String, error: String },
}

/// Result from background file loading
pub struct FileLoadResult {
    pub content: String,
    pub format: String,
    pub path_str: String,
    pub converted_from_binary: bool,
    pub line_count: usize,
    pub needs_formatting: bool,
    pub error: Option<String>,
}
