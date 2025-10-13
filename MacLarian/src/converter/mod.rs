//! Format conversion utilities

use crate::error::Result;
use crate::formats::lsf;
use crate::formats::lsx::LsxDocument;
use std::path::Path;

/// Convert LSF to LSX
pub fn lsf_to_lsx<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    lsf::convert_lsf_to_lsx(source, dest)
}

/// Convert LSX to LSF
pub fn lsx_to_lsf<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    let content = std::fs::read_to_string(source)?;
    let doc = LsxDocument::from_xml(&content)?;
    
    // TODO: Implement LSX -> LSF conversion
    // For now, return an error
    Err(crate::error::Error::ConversionError(
        "LSX to LSF conversion not yet implemented".to_string()
    ))
}

/// Batch convert all files in a directory
pub fn batch_convert_directory<P: AsRef<Path>>(
    dir: P,
    from_format: &str,
    to_format: &str,
    recursive: bool,
) -> Result<Vec<std::path::PathBuf>> {
    use walkdir::WalkDir;
    
    let mut converted = Vec::new();
    let walker = if recursive {
        WalkDir::new(&dir)
    } else {
        WalkDir::new(&dir).max_depth(1)
    };
    
    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(ext) = path.extension() {
            if ext == from_format {
                let mut dest_path = path.to_path_buf();
                dest_path.set_extension(to_format);
                
                match (from_format, to_format) {
                    ("lsf", "lsx") => lsf_to_lsx(path, &dest_path)?,
                    ("lsx", "lsf") => lsx_to_lsf(path, &dest_path)?,
                    _ => continue,
                }
                
                converted.push(dest_path);
            }
        }
    }
    
    Ok(converted)
}
