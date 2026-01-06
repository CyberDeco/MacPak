//! Dialog operations - split into focused modules

mod file_ops;
mod loading;
mod display;
mod export;
mod localization;
mod speakers;

// Re-export public API
pub use file_ops::{load_pak_directly, open_dialog_folder};
pub use loading::{load_dialog, load_dialog_entry, load_dialog_from_pak};
pub use export::{export_html, export_de2};
pub use speakers::SpeakerNameCache;
