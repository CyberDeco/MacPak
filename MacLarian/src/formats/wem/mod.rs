//! WEM (Wwise Encoded Media) format decoder
//!
//! WEM files are RIFF containers used by Wwise (Audiokinetic's game audio middleware).
//! BG3 uses Wwise for audio, storing voice lines as WEM files with Vorbis encoding.
//!
//! ## Format Overview
//!
//! WEM files can contain various audio codecs:
//! - PCM (0x0001) - Uncompressed audio
//! - Wwise Vorbis (0xFFFF) - Modified Ogg Vorbis
//! - ADPCM (0x0002) - Compressed audio
//!
//! ## Decoding
//!
//! Wwise Vorbis uses a proprietary format that requires external tools to decode.
//! Use `load_wem_file_vgmstream()` which shells out to vgmstream-cli.
//!
//! Install vgmstream: `brew install vgmstream` (macOS)
//!
//! ## Caching
//!
//! Use `AudioCache` to cache decoded audio for efficient repeated playback.

mod decoder;
mod cache;

pub use decoder::{
    WemError,
    WemHeader,
    DecodedAudio,
    WwiseVorbisHeader,
    parse_wem_header,
    parse_wwise_vorbis_header,
    decode_wwise_vorbis_fallback,
};

pub use cache::{AudioCache, AudioCacheError, CachedAudio, CacheStats};

#[cfg(feature = "audio")]
pub use decoder::load_wem_file_vgmstream;
