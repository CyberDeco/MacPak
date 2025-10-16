//! PAK creation functionality

pub use super::pak_tools::PakOperations;

pub fn create_pak<P: AsRef<std::path::Path>>(source: P, pak: P) -> crate::error::Result<()> {
    PakOperations::create(source, pak)
}
