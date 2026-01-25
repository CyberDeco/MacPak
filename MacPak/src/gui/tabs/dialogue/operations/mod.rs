//! Dialog operations - split into focused modules

mod file_ops;
mod loading;
mod display;
mod export;
mod voice;
mod audio;

// Re-export public API
pub use file_ops::{load_pak_directly, open_dialog_folder};
pub use loading::{load_dialog, load_dialog_entry, load_dialog_from_pak};
pub use display::{
    build_display_nodes,
    resolve_speaker_names,
    resolve_localized_text,
    resolve_flag_names,
    resolve_difficulty_classes,
};
pub use export::{export_html, export_de2};
pub use voice::{load_voice_meta, find_voice_files_path};
pub use audio::{AudioPlayer, AudioError, play_node_audio};
