//! LSJ to LSF conversion (via LSX intermediate)

use crate::error::Result;
use crate::formats::{lsf, lsj, lsx};
use std::path::Path;

/// Convert LSJ file to LSF format
/// This goes through LSX as an intermediate step: LSJ → LSX → LSF
pub fn convert_lsj_to_lsf<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    tracing::info!("Converting LSJ→LSF: {:?} → {:?}", source.as_ref(), dest.as_ref());
    
    // Step 1: Read LSJ
    let lsj_doc = lsj::read_lsj(&source)?;
    
    // Step 2: Convert to LSX document structure
    let lsx_doc = super::lsj_to_lsx::to_lsx(&lsj_doc)?;
    
    // Step 3: Serialize LSX to XML string
    let lsx_xml = lsx::serialize_lsx(&lsx_doc)?;
    
    // Step 4: Parse XML and convert to LSF
    let lsf_doc = super::lsx_to_lsf::from_lsx(&lsx_xml)?;
    
    // Step 5: Write LSF
    lsf::write_lsf(&lsf_doc, dest)?;
    
    tracing::info!("Conversion complete");
    Ok(())
}