// extractor.rs
pub use super::pak_tools::PakOperations;

pub fn extract_pak<P: AsRef<std::path::Path>>(pak: P, dest: P) -> crate::error::Result<()> {
    PakOperations::extract(pak, dest)
}
