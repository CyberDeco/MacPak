//! WEM decoder implementation
//!
//! Decodes WEM (Wwise Encoded Media) files to PCM audio.

use std::io::{Read, Seek, SeekFrom, Cursor};
use byteorder::{LittleEndian, ReadBytesExt};
use thiserror::Error;

/// Errors that can occur during WEM decoding
#[derive(Error, Debug)]
pub enum WemError {
    #[error("Invalid RIFF header")]
    InvalidRiffHeader,
    #[error("Invalid WAVE format")]
    InvalidWaveFormat,
    #[error("Unsupported audio format: {0:#06x}")]
    UnsupportedFormat(u16),
    #[error("Missing required chunk: {0}")]
    MissingChunk(&'static str),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Vorbis decode error: {0}")]
    VorbisDecode(String),
    #[error("Invalid Wwise header: {0}")]
    InvalidWwiseHeader(String),
    #[error("Ogg error: {0}")]
    OggError(String),
}

/// RIFF chunk identifiers
const RIFF_MAGIC: &[u8; 4] = b"RIFF";
const WAVE_MAGIC: &[u8; 4] = b"WAVE";

/// Parsed WEM file header information
#[derive(Debug, Clone)]
pub struct WemHeader {
    /// Total file size (excluding RIFF header)
    pub file_size: u32,
    /// Audio format code (0xFFFF = Wwise Vorbis, 0x0001 = PCM)
    pub format_code: u16,
    /// Number of audio channels
    pub channels: u16,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Average bytes per second
    pub avg_bytes_per_sec: u32,
    /// Block alignment
    pub block_align: u16,
    /// Bits per sample (often 0 for compressed formats)
    pub bits_per_sample: u16,
    /// Extra format data from fmt chunk (Wwise-specific)
    pub extra_data: Vec<u8>,
    /// Offset to audio data in the file
    pub data_offset: u64,
    /// Size of audio data
    pub data_size: u32,
}

/// Decoded audio data ready for playback
#[derive(Debug, Clone)]
pub struct DecodedAudio {
    /// PCM samples as i16 (interleaved if stereo)
    pub samples: Vec<i16>,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Sample rate in Hz
    pub sample_rate: u32,
}

impl DecodedAudio {
    /// Get duration in seconds
    #[must_use]
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 || self.channels == 0 {
            return 0.0;
        }
        self.samples.len() as f32 / (self.sample_rate as f32 * self.channels as f32)
    }

    /// Get duration in milliseconds
    #[must_use]
    pub fn duration_ms(&self) -> u32 {
        (self.duration_secs() * 1000.0) as u32
    }
}

/// Parse WEM file header from a reader
///
/// # Errors
/// Returns an error if the file is not a valid WEM/RIFF file
pub fn parse_wem_header<R: Read + Seek>(reader: &mut R) -> Result<WemHeader, WemError> {
    // Read RIFF header
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != RIFF_MAGIC {
        return Err(WemError::InvalidRiffHeader);
    }

    let file_size = reader.read_u32::<LittleEndian>()?;

    // Check WAVE format
    reader.read_exact(&mut magic)?;
    if &magic != WAVE_MAGIC {
        return Err(WemError::InvalidWaveFormat);
    }

    let mut header = WemHeader {
        file_size,
        format_code: 0,
        channels: 0,
        sample_rate: 0,
        avg_bytes_per_sec: 0,
        block_align: 0,
        bits_per_sample: 0,
        extra_data: Vec::new(),
        data_offset: 0,
        data_size: 0,
    };

    // Parse chunks
    let mut found_fmt = false;
    let mut found_data = false;

    while !found_data {
        let mut chunk_id = [0u8; 4];
        if reader.read_exact(&mut chunk_id).is_err() {
            break;
        }

        let chunk_size = reader.read_u32::<LittleEndian>()?;
        let chunk_start = reader.stream_position()?;

        match &chunk_id {
            b"fmt " => {
                header.format_code = reader.read_u16::<LittleEndian>()?;
                header.channels = reader.read_u16::<LittleEndian>()?;
                header.sample_rate = reader.read_u32::<LittleEndian>()?;
                header.avg_bytes_per_sec = reader.read_u32::<LittleEndian>()?;
                header.block_align = reader.read_u16::<LittleEndian>()?;
                header.bits_per_sample = reader.read_u16::<LittleEndian>()?;

                // Read extra format data if present (Wwise-specific info)
                let basic_size = 16u32;
                if chunk_size > basic_size {
                    let extra_size = (chunk_size - basic_size) as usize;
                    header.extra_data = vec![0u8; extra_size];
                    reader.read_exact(&mut header.extra_data)?;
                }
                found_fmt = true;
            }
            b"data" => {
                header.data_offset = reader.stream_position()?;
                header.data_size = chunk_size;
                found_data = true;
            }
            _ => {
                // Skip unknown chunks (vorb, smpl, etc.)
            }
        }

        // Seek to next chunk (align to word boundary)
        let next_pos = chunk_start + u64::from(chunk_size);
        let aligned_pos = (next_pos + 1) & !1; // Round up to even
        reader.seek(SeekFrom::Start(aligned_pos))?;
    }

    if !found_fmt {
        return Err(WemError::MissingChunk("fmt "));
    }
    if !found_data {
        return Err(WemError::MissingChunk("data"));
    }

    Ok(header)
}

/// Parsed Wwise Vorbis header from fmt extra_data
#[derive(Debug, Clone)]
pub struct WwiseVorbisHeader {
    /// Total sample count
    pub sample_count: u32,
    /// Size of the Vorbis setup packet
    pub setup_packet_size: u32,
    /// Offset to first audio packet within data chunk
    pub first_audio_offset: u32,
    /// Mod signal (indicates packet size encoding)
    pub mod_signal: u16,
    /// UID for external codebook lookup
    pub uid: u32,
    /// Blocksize 0 exponent (2^n)
    pub blocksize_0_exp: u8,
    /// Blocksize 1 exponent (2^n)
    pub blocksize_1_exp: u8,
}

/// Parse Wwise Vorbis header from fmt extra_data
///
/// Based on analysis of BG3 WEM files and vgmstream documentation.
/// The Wwise format varies by version, this handles the BG3 variant.
pub fn parse_wwise_vorbis_header(extra_data: &[u8]) -> Result<WwiseVorbisHeader, WemError> {
    if extra_data.len() < 32 {
        return Err(WemError::InvalidWwiseHeader(format!(
            "Extra data too short: {} bytes, need at least 32",
            extra_data.len()
        )));
    }

    let mut cursor = Cursor::new(extra_data);

    // Wwise Vorbis header structure for BG3:
    // Offset 0: cb_size (2 bytes) - extra data size indicator
    let _cb_size = cursor.read_u16::<LittleEndian>()?;

    // Offset 2: unknown (2 bytes)
    let _unknown1 = cursor.read_u16::<LittleEndian>()?;

    // Offset 4: unknown flags (4 bytes) - e.g., 0x00004101
    let _flags = cursor.read_u32::<LittleEndian>()?;

    // Offset 8: Sample count (4 bytes) - actual total samples
    let sample_count = cursor.read_u32::<LittleEndian>()?;

    // Offset 12: Setup packet size (4 bytes) - Vorbis setup header size
    let setup_packet_size = cursor.read_u32::<LittleEndian>()?;

    // Offset 16: Audio data size (4 bytes) - should match data chunk size
    let _audio_data_size = cursor.read_u32::<LittleEndian>()?;

    // Offset 20: Mod signal / packet mode (2 bytes)
    let mod_signal = cursor.read_u16::<LittleEndian>()?;

    // Offset 22: unknown (2 bytes)
    let _unknown2 = cursor.read_u16::<LittleEndian>()?;

    // Offset 24: UID / codebook identifier (4 bytes)
    let uid = cursor.read_u32::<LittleEndian>()?;

    // Offset 28: First audio packet offset within data chunk (4 bytes)
    let first_audio_offset = cursor.read_u32::<LittleEndian>()?;

    // Offset 32+ may contain more data, including blocksizes
    // Try to read blocksizes if we have enough data
    let (blocksize_0_exp, blocksize_1_exp) = if extra_data.len() >= 40 {
        // Offset 32: may contain blocksize info
        let bs_low = cursor.read_u16::<LittleEndian>().unwrap_or(0);
        let bs_high = cursor.read_u16::<LittleEndian>().unwrap_or(0);
        // Extract blocksize exponents (typically 8-13)
        let bs0 = if bs_low > 0 { (bs_low as f32).log2() as u8 } else { 8 };
        let bs1 = if bs_high > 0 { (bs_high as f32).log2() as u8 } else { 11 };
        (bs0.max(6).min(13), bs1.max(6).min(13))
    } else {
        // Default blocksizes for voice (mono, lower bitrate)
        (8, 11) // 256 and 2048 samples
    };

    Ok(WwiseVorbisHeader {
        sample_count,
        setup_packet_size,
        first_audio_offset,
        mod_signal,
        uid,
        blocksize_0_exp,
        blocksize_1_exp,
    })
}

/// Find vgmstream-cli, checking app bundle first, then PATH
#[cfg(feature = "gui")]
fn find_vgmstream_cli() -> Option<std::path::PathBuf> {
    // 1. Check inside app bundle (for packaged .app)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            // Check MacOS/ directory (same as executable)
            let bundled = parent.join("vgmstream-cli");
            if bundled.exists() {
                return Some(bundled);
            }

            // Check Resources/ directory
            if let Some(contents) = parent.parent() {
                let resources = contents.join("Resources").join("vgmstream-cli");
                if resources.exists() {
                    return Some(resources);
                }
            }
        }
    }

    // 2. Check common Homebrew locations
    let homebrew_paths = [
        "/opt/homebrew/bin/vgmstream-cli",  // Apple Silicon
        "/usr/local/bin/vgmstream-cli",      // Intel Mac
    ];

    for path in homebrew_paths {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // 3. Fall back to PATH
    if let Ok(output) = std::process::Command::new("which")
        .arg("vgmstream-cli")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(std::path::PathBuf::from(path));
            }
        }
    }

    None
}

/// Decode Wwise Vorbis using vgmstream-cli
///
/// vgmstream is a mature library that handles all Wwise format variations.
/// We shell out to vgmstream-cli to convert WEM to WAV, then read the result.
#[cfg(feature = "gui")]
fn decode_wwise_vorbis_with_vgmstream(wem_path: &std::path::Path) -> Result<DecodedAudio, WemError> {
    use std::process::Command;

    // Find vgmstream-cli
    let vgmstream_path = find_vgmstream_cli().ok_or_else(|| {
        WemError::VorbisDecode(
            "vgmstream-cli not found. Install it for audio playback:\n  \
             macOS: brew install vgmstream\n  \
             Linux: See https://github.com/vgmstream/vgmstream\n  \
             Windows: Download from https://vgmstream.org".to_string()
        )
    })?;

    // Create temp file for WAV output
    let temp_dir = std::env::temp_dir();
    let wav_path = temp_dir.join(format!("macpak_audio_{}.wav", std::process::id()));

    // Run vgmstream-cli to convert WEM to WAV
    let output = Command::new(&vgmstream_path)
        .arg("-o")
        .arg(&wav_path)
        .arg(wem_path)
        .output()
        .map_err(|e| WemError::VorbisDecode(format!("Failed to run vgmstream-cli: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WemError::VorbisDecode(format!(
            "vgmstream-cli failed: {}", stderr.trim()
        )));
    }

    // Read the WAV file
    let wav_data = std::fs::read(&wav_path)
        .map_err(|e| WemError::VorbisDecode(format!("Failed to read WAV output: {}", e)))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&wav_path);

    // Parse WAV header and extract samples
    parse_wav_to_audio(&wav_data)
}

/// Parse a WAV file into DecodedAudio
#[cfg(feature = "gui")]
fn parse_wav_to_audio(wav_data: &[u8]) -> Result<DecodedAudio, WemError> {
    use std::io::{Cursor, Read};
    use byteorder::{LittleEndian, ReadBytesExt};

    let mut cursor = Cursor::new(wav_data);

    // RIFF header
    let mut riff = [0u8; 4];
    cursor.read_exact(&mut riff)?;
    if &riff != b"RIFF" {
        return Err(WemError::VorbisDecode("Invalid WAV: missing RIFF".to_string()));
    }

    let _file_size = cursor.read_u32::<LittleEndian>()?;

    let mut wave = [0u8; 4];
    cursor.read_exact(&mut wave)?;
    if &wave != b"WAVE" {
        return Err(WemError::VorbisDecode("Invalid WAV: missing WAVE".to_string()));
    }

    // Find fmt and data chunks
    let mut channels = 0u16;
    let mut sample_rate = 0u32;
    let mut bits_per_sample = 0u16;
    let mut samples = Vec::new();

    loop {
        let mut chunk_id = [0u8; 4];
        if cursor.read_exact(&mut chunk_id).is_err() {
            break;
        }
        let chunk_size = cursor.read_u32::<LittleEndian>()?;

        match &chunk_id {
            b"fmt " => {
                let _format = cursor.read_u16::<LittleEndian>()?; // PCM = 1
                channels = cursor.read_u16::<LittleEndian>()?;
                sample_rate = cursor.read_u32::<LittleEndian>()?;
                let _byte_rate = cursor.read_u32::<LittleEndian>()?;
                let _block_align = cursor.read_u16::<LittleEndian>()?;
                bits_per_sample = cursor.read_u16::<LittleEndian>()?;

                // Skip any extra fmt data
                let read_so_far = 16;
                if chunk_size > read_so_far {
                    let skip = chunk_size - read_so_far;
                    cursor.set_position(cursor.position() + skip as u64);
                }
            }
            b"data" => {
                // Read sample data
                let num_samples = chunk_size as usize / (bits_per_sample as usize / 8);
                samples.reserve(num_samples);

                match bits_per_sample {
                    16 => {
                        for _ in 0..num_samples {
                            samples.push(cursor.read_i16::<LittleEndian>()?);
                        }
                    }
                    24 => {
                        // Convert 24-bit to 16-bit
                        for _ in 0..(chunk_size / 3) {
                            let mut bytes = [0u8; 3];
                            cursor.read_exact(&mut bytes)?;
                            // Take upper 16 bits of 24-bit sample
                            let sample = i16::from_le_bytes([bytes[1], bytes[2]]);
                            samples.push(sample);
                        }
                    }
                    32 => {
                        // Assume 32-bit float, convert to 16-bit
                        for _ in 0..(chunk_size / 4) {
                            let float_sample = cursor.read_f32::<LittleEndian>()?;
                            let sample = (float_sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            samples.push(sample);
                        }
                    }
                    _ => {
                        return Err(WemError::VorbisDecode(format!(
                            "Unsupported bits per sample: {}", bits_per_sample
                        )));
                    }
                }
                break;
            }
            _ => {
                // Skip unknown chunk
                cursor.set_position(cursor.position() + chunk_size as u64);
            }
        }
    }

    if samples.is_empty() {
        return Err(WemError::VorbisDecode("WAV file has no audio data".to_string()));
    }

    Ok(DecodedAudio {
        samples,
        channels,
        sample_rate,
    })
}

/// Load and decode a WEM file using vgmstream
///
/// This is the recommended way to decode BG3 voice files.
/// Requires vgmstream-cli to be installed.
#[cfg(feature = "gui")]
pub fn load_wem_file_vgmstream(path: &std::path::Path) -> Result<DecodedAudio, WemError> {
    decode_wwise_vorbis_with_vgmstream(path)
}

/// Decode Wwise Vorbis with fallback to silence
///
/// This generates silent audio with correct duration when vgmstream
/// isn't available. Useful for testing the playback pipeline.
#[cfg(feature = "gui")]
pub fn decode_wwise_vorbis_fallback(header: &WemHeader) -> Result<DecodedAudio, WemError> {
    let wwise_header = parse_wwise_vorbis_header(&header.extra_data)?;

    // Generate silent audio with correct duration
    let duration_secs = wwise_header.sample_count as f32 / header.sample_rate as f32;
    let num_samples = (duration_secs * header.sample_rate as f32 * header.channels as f32) as usize;

    tracing::warn!(
        "Using silent fallback for Wwise Vorbis ({:.2}s, {} channels, {} Hz)",
        duration_secs,
        header.channels,
        header.sample_rate
    );

    Ok(DecodedAudio {
        samples: vec![0i16; num_samples],
        channels: header.channels,
        sample_rate: header.sample_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pcm_wem_header() {
        // Create a minimal valid WEM file header for testing
        let mut data = Vec::new();

        // RIFF header
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&100u32.to_le_bytes()); // file size
        data.extend_from_slice(b"WAVE");

        // fmt chunk
        data.extend_from_slice(b"fmt ");
        data.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        data.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        data.extend_from_slice(&2u16.to_le_bytes()); // 2 channels
        data.extend_from_slice(&48000u32.to_le_bytes()); // sample rate
        data.extend_from_slice(&192000u32.to_le_bytes()); // byte rate
        data.extend_from_slice(&4u16.to_le_bytes()); // block align
        data.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

        // data chunk
        data.extend_from_slice(b"data");
        data.extend_from_slice(&0u32.to_le_bytes()); // data size

        let mut cursor = Cursor::new(data);
        let header = parse_wem_header(&mut cursor).unwrap();

        assert_eq!(header.format_code, 1);
        assert_eq!(header.channels, 2);
        assert_eq!(header.sample_rate, 48000);
    }

    #[test]
    fn test_duration_calculation() {
        let audio = DecodedAudio {
            samples: vec![0i16; 96000], // 1 second of stereo 48kHz
            channels: 2,
            sample_rate: 48000,
        };

        assert!((audio.duration_secs() - 1.0).abs() < 0.001);
        assert_eq!(audio.duration_ms(), 1000);
    }
}
