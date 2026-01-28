//! Dialog operations - split into focused modules

mod audio;
mod display;
mod export;
mod file_ops;
mod loading;
mod voice;

// Re-export public API
pub use audio::{AudioError, AudioPlayer, play_node_audio};
pub use display::{
    build_display_nodes, resolve_difficulty_classes, resolve_flag_names, resolve_localized_text,
    resolve_speaker_names,
};
pub use export::{export_de2, export_html};
pub use file_ops::{load_pak_directly, open_dialog_folder};
pub use loading::{load_dialog, load_dialog_entry, load_dialog_from_pak};
pub use voice::{find_voice_files_path, load_voice_meta};
