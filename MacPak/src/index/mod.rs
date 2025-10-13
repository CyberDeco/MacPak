pub mod database;
pub mod indexer;
pub mod search;

use crate::error::Result;

#[derive(Debug)]
pub struct FileIndex {
    // Will implement later
}

impl FileIndex {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}
