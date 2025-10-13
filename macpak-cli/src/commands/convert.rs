//! CLI interface between LSF <> LSX conversion
use std::path::Path;

pub fn execute(
    source: &Path, 
    destination: &Path, 
    input_format: Option<&str>,
    output_format: Option<&str>
) -> anyhow::Result<()> {
    println!("Converting {:?} to {:?}", source, destination);
    
    // Auto-detect or use provided formats
    let input = if let Some(fmt) = input_format {
        fmt.to_lowercase()
    } else {
        source.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Cannot detect input format from source file extension"))?
    };
    
    let output = if let Some(fmt) = output_format {
        fmt.to_lowercase()
    } else {
        destination.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Cannot detect output format from destination file extension"))?
    };
    
    // Execute conversion based on input/output format
    match (input.as_str(), output.as_str()) {
        ("lsf", "lsx") => {
            println!("Converting LSF → LSX");
            MacPak::operations::conversion::lsf_to_lsx(source, destination)?;
        }
        ("lsx", "lsf") => {
            println!("Converting LSX → LSF");
            MacPak::operations::conversion::lsx_to_lsf(source, destination)?;
        }
        _ => {
            anyhow::bail!(
                "Unsupported conversion: {} → {}. Supported: lsf→lsx, lsx→lsf",
                input, output
            );
        }
    }
    
    println!("✓ Conversion complete");
    Ok(())
}