//! LSX (XML) format module
//!
//!

mod document;
mod reader;
mod writer;

pub use document::{LsxAttribute, LsxDocument, LsxNode, LsxRegion};
pub use reader::{parse_lsx, read_lsx};
pub use writer::{serialize_lsx, write_lsx};
