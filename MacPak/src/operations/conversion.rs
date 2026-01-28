//! Format conversion operations

use crate::error::Result;
use std::path::Path;

// From LSF
pub fn lsf_to_lsx(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsf_to_lsx(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

pub fn lsf_to_lsj(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsf_to_lsj(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

// From LSX
pub fn lsx_to_lsf(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsx_to_lsf(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

pub fn lsx_to_lsj(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsx_to_lsj(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

// From LSJ
pub fn lsj_to_lsf(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsj_to_lsf(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

pub fn lsj_to_lsx(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsj_to_lsx(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

// LOCA (localization)
pub fn loca_to_xml(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::convert_loca_to_xml(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}

pub fn xml_to_loca(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::convert_xml_to_loca(source.as_ref(), dest.as_ref()).map_err(|e| e.into())
}
