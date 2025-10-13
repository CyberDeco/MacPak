//! Format conversion operations

use crate::error::Result;
use std::path::Path;

pub fn lsf_to_lsx(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsf_to_lsx(source.as_ref(), dest.as_ref())
        .map_err(|e| e.into())
}