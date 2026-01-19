//! File extraction operations

use crate::error::Result;
use std::path::Path;

pub fn extract_pak(pak: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::pak::PakOperations::extract(pak.as_ref(), dest.as_ref())
        .map_err(|e| e.into())
}