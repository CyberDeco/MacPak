//! Shared utilities for MacPak GUI

pub mod clipboard;
pub mod config_dialog;
pub mod errors;
pub mod meta_dialog;
pub mod meta_generator;
pub mod uuid;

pub use clipboard::copy_to_clipboard;
pub use config_dialog::config_dialog;
pub use errors::show_file_error;
pub use meta_generator::generate_meta_lsx;
pub use uuid::{UuidFormat, generate_uuid};
