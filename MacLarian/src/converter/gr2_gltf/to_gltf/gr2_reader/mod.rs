//! GR2 format structures and parser.
//!
//!

mod reader;
mod types;
mod vertex_types;

pub use reader::Gr2Reader;
pub use types::{Bone, MeshData, MeshExtendedData, Model, Skeleton};
