//! WEM audio file commands

use std::path::Path;
use anyhow::Result;
use crate::formats::wem::{
    parse_wem_header, parse_wwise_vorbis_header,
    decode_wwise_vorbis_fallback, load_wem_file_vgmstream,
};

/// Inspect a WEM file and display its header information
pub fn inspect(path: &Path) -> Result<()> {
    let data = std::fs::read(path)?;
    let mut cursor = std::io::Cursor::new(&data);

    let header = parse_wem_header(&mut cursor)?;

    println!("WEM File: {}", path.display());
    println!("---------------------------------");
    println!("Format code:      {:#06x}", header.format_code);
    println!("Format name:      {}", format_name(header.format_code));
    println!("Channels:         {}", header.channels);
    println!("Sample rate:      {} Hz", header.sample_rate);
    println!("Avg bytes/sec:    {}", header.avg_bytes_per_sec);
    println!("Block align:      {}", header.block_align);
    println!("Bits per sample:  {}", header.bits_per_sample);
    println!("Data offset:      {:#x}", header.data_offset);
    println!("Data size:        {} bytes", header.data_size);
    println!("Extra data size:  {} bytes", header.extra_data.len());

    // Calculate estimated duration
    if header.avg_bytes_per_sec > 0 {
        let duration = header.data_size as f32 / header.avg_bytes_per_sec as f32;
        println!("Est. duration:    {:.2} seconds", duration);
    }

    // Parse Wwise Vorbis header if applicable
    if header.format_code == 0xFFFF && header.extra_data.len() >= 32 {
        println!("\nWwise Vorbis Header:");
        match parse_wwise_vorbis_header(&header.extra_data) {
            Ok(wwise) => {
                let duration = wwise.sample_count as f32 / header.sample_rate as f32;
                println!("  Sample count:     {}", wwise.sample_count);
                println!("  Duration:         {:.2} seconds", duration);
                println!("  Setup pkt size:   {} bytes", wwise.setup_packet_size);
                println!("  First audio ofs:  {}", wwise.first_audio_offset);
                println!("  Mod signal:       {}", wwise.mod_signal);
                println!("  UID:              {:#010x}", wwise.uid);
                println!("  Blocksize 0:      2^{} = {}", wwise.blocksize_0_exp, 1u32 << wwise.blocksize_0_exp);
                println!("  Blocksize 1:      2^{} = {}", wwise.blocksize_1_exp, 1u32 << wwise.blocksize_1_exp);
            }
            Err(e) => {
                println!("  (Failed to parse: {})", e);
            }
        }
    }

    // Show extra data hex dump for debugging
    if !header.extra_data.is_empty() {
        println!("\nExtra format data (first 64 bytes):");
        let show_len = header.extra_data.len().min(64);
        for (i, chunk) in header.extra_data[..show_len].chunks(16).enumerate() {
            print!("  {:04x}: ", i * 16);
            for byte in chunk {
                print!("{:02x} ", byte);
            }
            println!();
        }
    }

    Ok(())
}

/// Decode a WEM file and show decode result or error
pub fn decode(path: &Path, output: Option<&Path>, silent_fallback: bool) -> Result<()> {
    println!("Decoding: {}", path.display());

    // Try vgmstream first (recommended for Wwise Vorbis)
    let audio_result = load_wem_file_vgmstream(path);

    let audio = match audio_result {
        Ok(audio) => audio,
        Err(e) if silent_fallback => {
            // Use silent fallback when vgmstream isn't available
            println!("Warning: vgmstream decode failed: {}", e);
            println!("  Using silent fallback (--silent mode)...");

            let data = std::fs::read(path)?;
            let mut cursor = std::io::Cursor::new(&data);
            let header = parse_wem_header(&mut cursor)?;
            decode_wwise_vorbis_fallback(&header)?
        }
        Err(e) => {
            println!("Decode failed: {}", e);
            return Err(e.into());
        }
    };

    println!("Decoded successfully!");
    println!("  Channels:    {}", audio.channels);
    println!("  Sample rate: {} Hz", audio.sample_rate);
    println!("  Samples:     {}", audio.samples.len());
    println!("  Duration:    {:.2} seconds", audio.duration_secs());

    if let Some(out_path) = output {
        // Write as WAV
        write_wav(out_path, &audio)?;
        println!("  Written to: {}", out_path.display());
    }

    Ok(())
}

/// Write decoded audio as WAV file
fn write_wav(path: &Path, audio: &crate::formats::wem::DecodedAudio) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;

    let data_size = (audio.samples.len() * 2) as u32;
    let file_size = 36 + data_size;

    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // chunk size
    file.write_all(&1u16.to_le_bytes())?; // PCM format
    file.write_all(&audio.channels.to_le_bytes())?;
    file.write_all(&audio.sample_rate.to_le_bytes())?;
    let byte_rate = audio.sample_rate * u32::from(audio.channels) * 2;
    file.write_all(&byte_rate.to_le_bytes())?;
    let block_align = audio.channels * 2;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?; // bits per sample

    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;

    // Write samples
    for sample in &audio.samples {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}

fn format_name(code: u16) -> &'static str {
    match code {
        0x0001 => "PCM",
        0x0002 => "ADPCM",
        0x0003 => "Vorbis (alt)",
        0xFFFF => "Wwise Vorbis",
        _ => "Unknown",
    }
}
