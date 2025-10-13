pub mod project;
pub mod settings;

use crate::error::Result;
use std::path::Path;

#[derive(Debug)]
pub struct Workspace {
    // Will implement later
}

impl Workspace {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
    
    pub fn open(&mut self, _path: impl AsRef<Path>) -> Result<()> {
        // TODO: Implement workspace opening
        Ok(())
    }
}
