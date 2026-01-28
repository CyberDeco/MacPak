//! Format conversion operations

use crate::error::Result;
use std::path::Path;

/// Converts an LSF file to LSX format.
///
/// # Errors
///
/// Returns an error if the file cannot be read or conversion fails.
pub fn lsf_to_lsx(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsf_to_lsx(source.as_ref(), dest.as_ref()).map_err(Into::into)
}

/// Converts an LSF file to LSJ format.
///
/// # Errors
///
/// Returns an error if the file cannot be read or conversion fails.
pub fn lsf_to_lsj(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsf_to_lsj(source.as_ref(), dest.as_ref()).map_err(Into::into)
}

/// Converts an LSX file to LSF format.
///
/// # Errors
///
/// Returns an error if the file cannot be read or conversion fails.
pub fn lsx_to_lsf(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsx_to_lsf(source.as_ref(), dest.as_ref()).map_err(Into::into)
}

/// Converts an LSX file to LSJ format.
///
/// # Errors
///
/// Returns an error if the file cannot be read or conversion fails.
pub fn lsx_to_lsj(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsx_to_lsj(source.as_ref(), dest.as_ref()).map_err(Into::into)
}

/// Converts an LSJ file to LSF format.
///
/// # Errors
///
/// Returns an error if the file cannot be read or conversion fails.
pub fn lsj_to_lsf(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsj_to_lsf(source.as_ref(), dest.as_ref()).map_err(Into::into)
}

/// Converts an LSJ file to LSX format.
///
/// # Errors
///
/// Returns an error if the file cannot be read or conversion fails.
pub fn lsj_to_lsx(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::lsj_to_lsx(source.as_ref(), dest.as_ref()).map_err(Into::into)
}

/// Converts a LOCA localization file to XML format.
///
/// # Errors
///
/// Returns an error if the file cannot be read or conversion fails.
pub fn loca_to_xml(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::convert_loca_to_xml(source.as_ref(), dest.as_ref()).map_err(Into::into)
}

/// Converts an XML file to LOCA localization format.
///
/// # Errors
///
/// Returns an error if the file cannot be read or conversion fails.
pub fn xml_to_loca(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    maclarian::converter::convert_xml_to_loca(source.as_ref(), dest.as_ref()).map_err(Into::into)
}
