// lister.rs
pub use super::pak_tools::PakOperations;

pub fn list_pak_contents<P: AsRef<std::path::Path>>(pak: P) -> crate::error::Result<Vec<String>> {
    PakOperations::list(pak)
}