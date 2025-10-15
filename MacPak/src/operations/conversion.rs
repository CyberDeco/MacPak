//! Format conversion operations

use crate::error::Result;
use std::path::Path;

// From LSF
pub fn lsf_to_lsx(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    MacLarian::converter::lsf_to_lsx(source.as_ref(), dest.as_ref())
        .map_err(|e| e.into())
}

pub fn lsf_to_lsj(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    MacLarian::converter::lsf_to_lsj(source.as_ref(), dest.as_ref())
        .map_err(|e| e.into())
}

// From LSX
pub fn lsx_to_lsf(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    MacLarian::converter::lsx_to_lsf(source.as_ref(), dest.as_ref())
        .map_err(|e| e.into())
}

pub fn lsx_to_lsj(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    MacLarian::converter::lsx_to_lsj(source.as_ref(), dest.as_ref())
        .map_err(|e| e.into())
}

// From LSJ
pub fn lsj_to_lsf(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    MacLarian::converter::lsj_to_lsf(source.as_ref(), dest.as_ref())
        .map_err(|e| e.into())
}

pub fn lsj_to_lsx(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    MacLarian::converter::lsj_to_lsx(source.as_ref(), dest.as_ref())
        .map_err(|e| e.into())
}