//! File format handlers for GUI-specific formats
//!
//! These formats are specific to the GUI application and are not part of
//! the core MacLarian library.

pub mod voice_meta;

#[cfg(feature = "gui")]
pub mod wem;

// Re-export voice meta types
pub use voice_meta::{
    VoiceMetaEntry, VoiceMetaCache,
    load_voice_meta_from_pak, load_voice_meta_from_folder,
    find_voice_files_path, find_voice_meta_path,
};

// Re-export WEM/audio types
#[cfg(feature = "gui")]
pub use wem::{
    WemError, WemHeader, DecodedAudio, WwiseVorbisHeader,
    parse_wem_header, parse_wwise_vorbis_header, decode_wwise_vorbis_fallback,
    AudioCache, AudioCacheError, CachedAudio, CacheStats,
    load_wem_file_vgmstream,
};
