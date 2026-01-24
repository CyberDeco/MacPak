use std::path::Path;

pub fn execute(source: &Path, destination: &Path) -> anyhow::Result<()> {
    println!("Extracting {:?} to {:?}", source, destination);
    crate::pak::PakOperations::extract(source, destination)?;
    println!("Extraction complete");
    Ok(())
}
