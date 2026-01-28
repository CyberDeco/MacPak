pub mod database;
pub mod indexer;
pub mod search;

use crate::error::Result;

/// File index for tracking and searching files.
#[derive(Debug)]
pub struct FileIndex {
    // Will implement later
}

impl FileIndex {
    /// Creates a new file index.
    ///
    /// # Errors
    ///
    /// Returns an error if index initialization fails.
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}
