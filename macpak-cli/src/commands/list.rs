use std::path::Path;

pub fn execute(source: &Path) -> anyhow::Result<()> {
    println!("Listing contents of {:?}", source);
    let files = MacLarian::pak::PakOperations::list(source)?;
    for file in files {
        println!("  {}", file);
    }
    Ok(())
}