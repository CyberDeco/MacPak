//! Path utilities

use std::path::{Path, PathBuf};

/// Normalize path separators to forward slashes (for PAK files)
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Get relative path and normalize separators
pub fn relative_path<P: AsRef<Path>>(path: P, base: P) -> Option<String> {
    path.as_ref()
        .strip_prefix(base.as_ref())
        .ok()
        .map(|p| normalize_path(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("foo\\bar\\baz"), "foo/bar/baz");
        assert_eq!(normalize_path("foo/bar/baz"), "foo/bar/baz");
    }
}
