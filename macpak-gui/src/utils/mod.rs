//! Shared utilities for MacPak GUI

pub mod handle;
pub mod meta_dialog;
pub mod meta_generator;
pub mod uuid;

pub use handle::generate_handle;
pub use uuid::{generate_uuid, UuidFormat};
