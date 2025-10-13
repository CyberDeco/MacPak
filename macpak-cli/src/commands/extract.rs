use std::path::Path;

pub fn execute(source: &Path, destination: &Path) -> anyhow::Result<()> {
    println!("Extracting {:?} to {:?}", source, destination);
    macpak::operations::extraction::extract_pak(source, destination)?;
    println!("✓ Extraction complete");
    Ok(())
}