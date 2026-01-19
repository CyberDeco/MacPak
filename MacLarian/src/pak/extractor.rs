//! PAK extraction functionality
pub use super::pak_tools::PakOperations;

/// # Errors
/// Returns an error if PAK extraction fails.
pub fn extract_pak<P: AsRef<std::path::Path>>(pak: P, dest: P) -> crate::error::Result<()> {
    PakOperations::extract(pak, dest)
}
