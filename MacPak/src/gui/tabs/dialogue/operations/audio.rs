//! Audio playback operations for dialogue voice lines
//!
//! This module provides the rodio-based playback infrastructure.
//! WEM decoding is handled by maclarian's wem module via vgmstream.
//! Decoded audio is cached via AudioCache for efficient replay.

use std::path::Path;
use std::sync::{Arc, Mutex};
use floem::reactive::SignalUpdate;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use maclarian::formats::wem::{AudioCacheError, DecodedAudio, WemError};

use crate::gui::state::DialogueState;

/// Audio player for dialogue voice playback
pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    current_sink: Arc<Mutex<Option<Sink>>>,
}

impl AudioPlayer {
    /// Create a new audio player
    ///
    /// # Errors
    /// Returns an error if audio output cannot be initialized
    pub fn new() -> Result<Self, AudioError> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::OutputInit(e.to_string()))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            current_sink: Arc::new(Mutex::new(None)),
        })
    }

    /// Play decoded audio
    pub fn play(&self, audio: DecodedAudio) -> Result<(), AudioError> {
        // Stop any currently playing audio
        self.stop();

        // Create a new sink
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::PlaybackError(e.to_string()))?;

        // Create source from decoded audio
        let source = DecodedAudioSource::new(audio);
        sink.append(source);

        // Store sink so we can stop it later
        if let Ok(mut current) = self.current_sink.lock() {
            *current = Some(sink);
        }

        Ok(())
    }

    /// Play a WEM file from disk using vgmstream
    pub fn play_file(&self, path: &Path) -> Result<(), AudioError> {
        let audio = maclarian::formats::wem::load_wem_file_vgmstream(path)?;
        self.play(audio)
    }

    /// Stop current playback
    pub fn stop(&self) {
        if let Ok(mut current) = self.current_sink.lock() {
            if let Some(sink) = current.take() {
                sink.stop();
            }
        }
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        if let Ok(current) = self.current_sink.lock() {
            if let Some(ref sink) = *current {
                return !sink.empty();
            }
        }
        false
    }
}

/// Errors that can occur during audio playback
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Failed to initialize audio output: {0}")]
    OutputInit(String),
    #[error("Playback error: {0}")]
    PlaybackError(String),
    #[error("WEM decode error: {0}")]
    WemDecode(#[from] WemError),
    #[error("Cache error: {0}")]
    CacheError(#[from] AudioCacheError),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Voice path not set")]
    VoicePathNotSet,
    #[error("Voice meta not found for handle: {0}")]
    VoiceMetaNotFound(String),
    #[error("Cache lock failed")]
    CacheLockFailed,
}

/// Rodio source wrapper for DecodedAudio
struct DecodedAudioSource {
    samples: Vec<i16>,
    position: usize,
    channels: u16,
    sample_rate: u32,
}

impl DecodedAudioSource {
    fn new(audio: DecodedAudio) -> Self {
        Self {
            samples: audio.samples,
            channels: audio.channels,
            sample_rate: audio.sample_rate,
            position: 0,
        }
    }
}

impl Iterator for DecodedAudioSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.samples.len() {
            let sample = self.samples[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for DecodedAudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len() - self.position)
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        let total_samples = self.samples.len() / self.channels as usize;
        let seconds = total_samples as f64 / self.sample_rate as f64;
        Some(std::time::Duration::from_secs_f64(seconds))
    }
}

/// Play audio for a dialogue node using the audio cache
///
/// This uses the AudioCache for efficient playback:
/// - First playback: decodes WEM via vgmstream and caches the result
/// - Subsequent playback: uses cached decoded audio (O(1) lookup)
pub fn play_node_audio(
    player: &AudioPlayer,
    state: &DialogueState,
    text_handle: &str,
    node_uuid: &str,
) -> Result<(), AudioError> {

    // Get voice meta for this handle (to get the .wem filename)
    let voice_meta = state.get_voice_meta(text_handle)
        .ok_or_else(|| AudioError::VoiceMetaNotFound(text_handle.to_string()))?;

    let wem_filename = voice_meta.source_file.clone();

    // Get or load audio from cache
    let audio = {
        let mut cache = state.audio_cache.write()
            .map_err(|_| AudioError::CacheLockFailed)?;

        // Use cache's get_or_load which handles decoding and caching
        let cached = cache.get_or_load(text_handle, &wem_filename)?;

        // Clone the audio data for playback (cache retains its copy)
        cached.audio.clone()
    };

    // Play the audio
    player.play(audio)?;

    // Update state to show which node is playing
    state.playing_audio_node.set(Some(node_uuid.to_string()));

    // Log cache stats for debugging
    if let Ok(cache) = state.audio_cache.read() {
        let stats = cache.stats();
        tracing::debug!(
            "Audio cache: {} entries, {:.1}MB, {} hits, {} misses",
            cache.len(),
            cache.total_size_mb(),
            stats.hits,
            stats.misses
        );
    }

    Ok(())
}
