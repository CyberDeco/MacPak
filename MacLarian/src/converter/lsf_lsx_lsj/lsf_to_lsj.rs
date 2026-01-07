//! LSF to LSJ conversion (via LSX intermediate)

use crate::error::Result;
use crate::formats::{lsf, lsj, lsx};
use std::path::Path;

/// Convert LSF file to LSJ format
/// This goes through LSX as an intermediate step: LSF → LSX → LSJ
pub fn convert_lsf_to_lsj<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    tracing::info!("Converting LSF→LSJ: {:?} → {:?}", source.as_ref(), dest.as_ref());
    
    // Step 1: Read LSF
    let lsf_doc = lsf::read_lsf(&source)?;
    
    // Step 2: Convert LSF to LSX XML string
    let lsx_xml = super::lsf_to_lsx::to_lsx(&lsf_doc)?;
    
    // Step 3: Parse LSX XML
    let lsx_doc = lsx::parse_lsx(&lsx_xml)?;
    
    // Step 4: Convert LSX to LSJ
    let lsj_doc = super::lsx_to_lsj::to_lsj(&lsx_doc)?;
    
    // Step 5: Write LSJ
    lsj::write_lsj(&lsj_doc, dest)?;
    
    tracing::info!("Conversion complete");
    Ok(())
}