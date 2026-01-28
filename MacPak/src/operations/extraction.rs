//! File extraction operations

use crate::error::Result;
use std::path::Path;

/// Extracts a PAK archive to the specified destination directory.
///
/// # Errors
///
/// Returns an error if the PAK file cannot be read or extraction fails.
pub fn extract_pak(pak: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::pak::PakOperations::extract(pak.as_ref(), dest.as_ref()).map_err(Into::into)
}
