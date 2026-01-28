pub mod project;
pub mod settings;

use crate::error::Result;
use std::path::Path;

/// Represents a modding workspace.
#[derive(Debug)]
pub struct Workspace {
    // Will implement later
}

impl Workspace {
    /// Creates a new workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if workspace creation fails.
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Opens an existing workspace or creates a new one at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace cannot be opened.
    pub fn open(&mut self, _path: impl AsRef<Path>) -> Result<()> {
        // TODO: Implement workspace opening
        Ok(())
    }
}
