//! GR2 (Granny2) file support

mod decompress;
mod file;

// Re-export public API
pub use decompress::decompress_gr2;
pub use file::{decompress_file, get_file_info, Gr2Header, Gr2SectionInfo};
