//! MacPak - High-level BG3 modding toolkit

use std::path::Path;

// Re-export MacLarian
pub use maclarian;

pub mod error;
pub mod index;
pub mod operations;
pub mod workspace;
pub mod utils;

pub use error::{Error, Result};

/// Main toolkit interface
pub struct Toolkit {
    workspace: workspace::Workspace,
    index: index::FileIndex,
}

impl Toolkit {
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace: workspace::Workspace::new()?,
            index: index::FileIndex::new()?,
        })
    }
    
    /// Open or create a workspace
    pub fn open_workspace(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.workspace.open(path)?;
        Ok(())
    }
    
    // High-level operations
    pub fn extract_pak(&self, pak: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
        operations::extraction::extract_pak(pak, dest)
    }
    
    pub fn convert_lsf_to_lsx(&self, source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
        operations::conversion::lsf_to_lsx(source, dest)
    }
}
