//! Public accessor methods for GtpFile.

use std::io::{Read, Seek};
use super::GtpFile;

impl<R: Read + Seek> GtpFile<R> {
    /// Get the number of pages in this GTP file.
    pub fn num_pages(&self) -> usize {
        self.chunk_offsets.len()
    }

    /// Get the number of chunks in a specific page.
    pub fn num_chunks(&self, page_index: usize) -> usize {
        self.chunk_offsets.get(page_index).map_or(0, std::vec::Vec::len)
    }
}
