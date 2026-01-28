//! File format handlers for GUI-specific formats
//!
//! These formats are specific to the GUI application and are not part of
//! the core MacLarian library.

pub mod voice_meta;

#[cfg(feature = "gui")]
pub mod wem;

// Re-export voice meta types
pub use voice_meta::{
    VoiceMetaCache, VoiceMetaEntry, find_voice_files_path, find_voice_meta_path,
    load_voice_meta_from_folder, load_voice_meta_from_pak,
};

// Re-export WEM/audio types
#[cfg(feature = "gui")]
pub use wem::{
    AudioCache, AudioCacheError, CacheStats, CachedAudio, DecodedAudio, WemError, WemHeader,
    WwiseVorbisHeader, decode_wwise_vorbis_fallback, load_wem_file_vgmstream, parse_wem_header,
    parse_wwise_vorbis_header,
};
