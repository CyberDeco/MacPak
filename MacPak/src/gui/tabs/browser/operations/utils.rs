//! Utility functions for the browser

/// Format file size for display
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return format!("{:.1} {}", size, unit);
        }
        size /= 1024.0;
    }
    format!("{:.1} PB", size)
}

/// Check if a file extension is a text/editable file type
pub fn is_text_file(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "lsf"
            | "lsx"
            | "lsj"
            | "loca"
            | "khn"
            | "txt"
            | "xml"
            | "json"
            | "lua"
            | "md"
            | "cfg"
            | "ini"
            | "yaml"
            | "yml"
            | "toml"
    )
}

/// Clean up temporary files created by the browser
pub fn cleanup_temp_files() {
    // Clean up loca preview temp file
    let _ = std::fs::remove_file("/tmp/temp_loca.xml");
}
