use std::path::Path;

pub fn execute(source: &Path, destination: &Path, _input: &str, _output: &str) -> anyhow::Result<()> {
    println!("Converting {:?} to {:?}", source, destination);
    macpak::operations::conversion::lsf_to_lsx(source, destination)?;
    println!("✓ Conversion complete");
    Ok(())
}