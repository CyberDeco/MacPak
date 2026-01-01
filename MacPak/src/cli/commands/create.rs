use std::path::Path;

pub fn execute(source: &Path, destination: &Path) -> anyhow::Result<()> {
    println!("Creating PAK from {:?} to {:?}", source, destination);
    MacLarian::pak::create_pak(source, destination)?;
    println!("✓ PAK created");
    Ok(())
}