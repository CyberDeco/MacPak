//! PAK creation functionality

pub use super::pak_tools::PakOperations;

/// # Errors
/// Returns an error if PAK creation fails.
pub fn create_pak<P: AsRef<std::path::Path>>(source: P, pak: P) -> crate::error::Result<()> {
    PakOperations::create(source, pak)
}
