//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! LSJ to LSF conversion (via LSX intermediate)

use crate::error::Result;
use crate::formats::{lsf, lsj, lsx};
use std::path::Path;

/// Convert LSJ file to LSF format
/// This goes through LSX as an intermediate step: LSJ → LSX → LSF
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_lsj_to_lsf<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    convert_lsj_to_lsf_with_progress(source, dest, &|_| {})
}

/// Convert LSJ file to LSF format with progress callback
/// This goes through LSX as an intermediate step: LSJ → LSX → LSF
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_lsj_to_lsf_with_progress<P: AsRef<Path>>(
    source: P,
    dest: P,
    progress: crate::converter::ConvertProgressCallback,
) -> Result<()> {
    use crate::converter::{ConvertProgress, ConvertPhase};
    tracing::info!("Converting LSJ→LSF: {:?} → {:?}", source.as_ref(), dest.as_ref());

    // Step 1: Read LSJ
    progress(&ConvertProgress::with_file(ConvertPhase::ReadingSource, 1, 5, "Reading LSJ file..."));
    let lsj_doc = lsj::read_lsj(&source)?;

    // Step 2: Convert to LSX document structure
    let region_count = lsj_doc.save.regions.len();
    progress(&ConvertProgress::with_file(ConvertPhase::Converting, 2, 5, format!("Converting {region_count} regions to XML...")));
    let lsx_doc = super::lsj_to_lsx::to_lsx(&lsj_doc)?;

    // Step 3: Serialize LSX to XML string
    progress(&ConvertProgress::with_file(ConvertPhase::Converting, 3, 5, "Serializing XML..."));
    let lsx_xml = lsx::serialize_lsx(&lsx_doc)?;

    // Step 4: Parse XML and convert to LSF
    progress(&ConvertProgress::with_file(ConvertPhase::Converting, 4, 5, "Converting to LSF binary..."));
    let lsf_doc = super::lsx_to_lsf::from_lsx(&lsx_xml)?;

    // Step 5: Write LSF
    let node_count = lsf_doc.nodes.len();
    progress(&ConvertProgress::with_file(ConvertPhase::WritingOutput, 5, 5, format!("Writing LSF binary ({node_count} nodes)...")));
    lsf::write_lsf(&lsf_doc, dest)?;

    tracing::info!("Conversion complete");
    Ok(())
}