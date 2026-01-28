//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! LSF to LSJ conversion (via LSX intermediate)

use crate::error::Result;
use crate::formats::{lsf, lsj, lsx};
use std::path::Path;

/// Convert LSF file to LSJ format
/// This goes through LSX as an intermediate step: LSF → LSX → LSJ
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_lsf_to_lsj<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    convert_lsf_to_lsj_with_progress(source, dest, &|_| {})
}

/// Convert LSF file to LSJ format with progress callback
/// This goes through LSX as an intermediate step: LSF → LSX → LSJ
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_lsf_to_lsj_with_progress<P: AsRef<Path>>(
    source: P,
    dest: P,
    progress: crate::converter::ConvertProgressCallback,
) -> Result<()> {
    use crate::converter::{ConvertPhase, ConvertProgress};
    tracing::info!(
        "Converting LSF→LSJ: {:?} → {:?}",
        source.as_ref(),
        dest.as_ref()
    );

    // Step 1: Read LSF
    progress(&ConvertProgress::with_file(
        ConvertPhase::ReadingSource,
        1,
        5,
        "Reading LSF binary...",
    ));
    let lsf_doc = lsf::read_lsf(&source)?;

    // Step 2: Convert LSF to LSX XML string
    let node_count = lsf_doc.nodes.len();
    progress(&ConvertProgress::with_file(
        ConvertPhase::Converting,
        2,
        5,
        format!("Converting {node_count} nodes to XML..."),
    ));
    let lsx_xml = super::lsf_to_lsx::to_lsx(&lsf_doc)?;

    // Step 3: Parse LSX XML
    progress(&ConvertProgress::with_file(
        ConvertPhase::Parsing,
        3,
        5,
        "Parsing XML structure...",
    ));
    let lsx_doc = lsx::parse_lsx(&lsx_xml)?;

    // Step 4: Convert LSX to LSJ
    let region_count = lsx_doc.regions.len();
    progress(&ConvertProgress::with_file(
        ConvertPhase::Converting,
        4,
        5,
        format!("Converting {region_count} regions to JSON..."),
    ));
    let lsj_doc = super::lsx_to_lsj::to_lsj(&lsx_doc)?;

    // Step 5: Write LSJ
    progress(&ConvertProgress::with_file(
        ConvertPhase::WritingOutput,
        5,
        5,
        "Writing LSJ file...",
    ));
    lsj::write_lsj(&lsj_doc, dest)?;

    tracing::info!("Conversion complete");
    Ok(())
}
