//! CLI interface for format conversion
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
        // LSF conversions
        ("lsf", "lsx") => {
            println!("Converting LSF → LSX");
            MacLarian::converter::lsf_to_lsx(source, destination)?;
        }
        ("lsf", "lsj") => {
            println!("Converting LSF → LSJ");
            MacLarian::converter::lsf_to_lsj(source, destination)?;
        }
        
        // LSX conversions
        ("lsx", "lsf") => {
            println!("Converting LSX → LSF");
            MacLarian::converter::lsx_to_lsf(source, destination)?;
        }
        ("lsx", "lsj") => {
            println!("Converting LSX → LSJ");
            MacLarian::converter::lsx_to_lsj(source, destination)?;
        }
        
        // LSJ conversions
        ("lsj", "lsx") => {
            println!("Converting LSJ → LSX");
            MacLarian::converter::lsj_to_lsx(source, destination)?;
        }
        ("lsj", "lsf") => {
            println!("Converting LSJ → LSF");
            MacLarian::converter::lsj_to_lsf(source, destination)?;
        }
        
        // Same format (copy)
        (fmt1, fmt2) if fmt1 == fmt2 => {
            println!("Source and destination formats are the same, copying file...");
            std::fs::copy(source, destination)?;
        }
        
        // Unsupported
        _ => {
            anyhow::bail!(
                "Unsupported conversion: {} → {}\n\
                 Supported conversions:\n\
                 • LSF ↔ LSX\n\
                 • LSF ↔ LSJ\n\
                 • LSX ↔ LSJ",
                input, output
            );
        }
    }
    
    println!("✓ Conversion complete");
    Ok(())
}