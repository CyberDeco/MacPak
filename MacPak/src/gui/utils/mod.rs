//! Shared utilities for MacPak GUI

pub mod handle;
// pub use handle::generate_handle;
pub mod meta_dialog;
pub mod meta_generator;
pub mod uuid;

pub use uuid::{generate_uuid, UuidFormat};
pub use meta_generator::generate_meta_lsx;
