//! DDS decoding - Block Compression (BC) decompression using `bcdec_rs`
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`
//!
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(clippy::cast_possible_truncation, clippy::doc_markdown)]

use crate::error::{Error, Result};
use ddsfile::{D3DFormat, Dds, DxgiFormat, FourCC};

/// Decode DDS texture data to RGBA pixels
///
/// # Errors
/// Returns an error if the format is unsupported, the texture is 3D, or data is invalid.
pub fn decode_dds_to_rgba(dds: &Dds) -> Result<Vec<u8>> {
    // Check for unsupported texture types
    if let Some(depth) = dds.header.depth {
        if depth > 1 {
            return Err(Error::DdsError(format!(
                "3D/volume textures are not supported (depth={depth})"
            )));
        }
    }

    let width = dds.get_width() as usize;
    let height = dds.get_height() as usize;
    let data = dds
        .get_data(0)
        .map_err(|e| Error::DdsError(format!("Failed to read DDS data: {e}")))?;

    // Determine format and decode
    if let Some(dxgi) = dds.get_dxgi_format() {
        decode_dxgi_format(data, width, height, dxgi)
    } else if let Some(d3d) = dds.get_d3d_format() {
        decode_d3d_format(data, width, height, d3d)
    } else if let Some(fourcc) = dds.header.spf.fourcc.as_ref() {
        // Handle FourCC codes not recognized by ddsfile crate
        decode_fourcc(data, width, height, fourcc.0)
    } else {
        Err(Error::DdsError("Unknown DDS format".to_string()))
    }
}

/// Decode DXGI format textures
fn decode_dxgi_format(
    data: &[u8],
    width: usize,
    height: usize,
    format: DxgiFormat,
) -> Result<Vec<u8>> {
    match format {
        // Uncompressed RGBA formats
        DxgiFormat::R8G8B8A8_UNorm | DxgiFormat::R8G8B8A8_UNorm_sRGB => Ok(data.to_vec()),
        DxgiFormat::B8G8R8A8_UNorm | DxgiFormat::B8G8R8A8_UNorm_sRGB => {
            // BGRA to RGBA
            let mut rgba = data.to_vec();
            for chunk in rgba.chunks_exact_mut(4) {
                chunk.swap(0, 2);
            }
            Ok(rgba)
        }
        // BC compressed formats
        DxgiFormat::BC1_UNorm | DxgiFormat::BC1_UNorm_sRGB => decode_bc(data, width, height, BcFormat::Bc1),
        DxgiFormat::BC2_UNorm | DxgiFormat::BC2_UNorm_sRGB => decode_bc(data, width, height, BcFormat::Bc2),
        DxgiFormat::BC3_UNorm | DxgiFormat::BC3_UNorm_sRGB => decode_bc(data, width, height, BcFormat::Bc3),
        DxgiFormat::BC4_UNorm => decode_bc(data, width, height, BcFormat::Bc4),
        DxgiFormat::BC5_UNorm => decode_bc(data, width, height, BcFormat::Bc5),
        DxgiFormat::BC6H_UF16 => decode_bc6h(data, width, height, false), // unsigned
        DxgiFormat::BC6H_SF16 => decode_bc6h(data, width, height, true),  // signed
        DxgiFormat::BC7_UNorm | DxgiFormat::BC7_UNorm_sRGB => decode_bc(data, width, height, BcFormat::Bc7),
        _ => Err(Error::DdsError(format!(
            "Unsupported DXGI format: {format:?}"
        ))),
    }
}

/// Decode D3D format textures
fn decode_d3d_format(
    data: &[u8],
    width: usize,
    height: usize,
    format: D3DFormat,
) -> Result<Vec<u8>> {
    match format {
        // Uncompressed formats
        D3DFormat::A8R8G8B8 => {
            // ARGB to RGBA
            let mut rgba = Vec::with_capacity(data.len());
            for chunk in data.chunks_exact(4) {
                rgba.push(chunk[1]); // R
                rgba.push(chunk[2]); // G
                rgba.push(chunk[3]); // B
                rgba.push(chunk[0]); // A
            }
            Ok(rgba)
        }
        D3DFormat::X8R8G8B8 => {
            // XRGB to RGBA (X is padding, treat as opaque)
            let mut rgba = Vec::with_capacity(data.len());
            for chunk in data.chunks_exact(4) {
                rgba.push(chunk[1]); // R
                rgba.push(chunk[2]); // G
                rgba.push(chunk[3]); // B
                rgba.push(255);      // A (opaque)
            }
            Ok(rgba)
        }
        D3DFormat::R8G8B8 => {
            // RGB to RGBA (add alpha channel)
            let pixel_count = width * height;
            let mut rgba = Vec::with_capacity(pixel_count * 4);
            for chunk in data.chunks_exact(3) {
                rgba.push(chunk[0]); // R
                rgba.push(chunk[1]); // G
                rgba.push(chunk[2]); // B
                rgba.push(255);      // A (opaque)
            }
            Ok(rgba)
        }
        // DXT compressed formats
        D3DFormat::DXT1 => decode_bc(data, width, height, BcFormat::Bc1),
        D3DFormat::DXT2 | D3DFormat::DXT3 => decode_bc(data, width, height, BcFormat::Bc2),
        D3DFormat::DXT4 | D3DFormat::DXT5 => decode_bc(data, width, height, BcFormat::Bc3),
        _ => Err(Error::DdsError(format!(
            "Unsupported D3D format: {format:?}"
        ))),
    }
}

/// Decode textures by raw FourCC code (for formats not recognized by ddsfile crate)
fn decode_fourcc(data: &[u8], width: usize, height: usize, fourcc: u32) -> Result<Vec<u8>> {
    match fourcc {
        // BC4 unsigned (single channel) - "BC4U" or "ATI1"
        FourCC::BC4_UNORM | FourCC::ATI1 => decode_bc(data, width, height, BcFormat::Bc4),
        // BC4 signed
        FourCC::BC4_SNORM => decode_bc(data, width, height, BcFormat::Bc4),
        // BC5 unsigned (two channels) - "ATI2" (BC5_UNORM and ATI2 are the same value)
        FourCC::BC5_UNORM => decode_bc(data, width, height, BcFormat::Bc5),
        // BC5 signed
        FourCC::BC5_SNORM => decode_bc(data, width, height, BcFormat::Bc5),
        _ => {
            // Convert FourCC to readable string for error message
            let bytes = fourcc.to_le_bytes();
            let fourcc_str: String = bytes.iter().map(|&b| b as char).collect();
            Err(Error::DdsError(format!(
                "Unsupported FourCC: {fourcc_str} (0x{fourcc:08X})"
            )))
        }
    }
}

// ============================================================================
// Block Compression (BC) formats - unified decoder using bcdec_rs
// ============================================================================

/// Supported BC compression formats
#[derive(Clone, Copy)]
enum BcFormat {
    Bc1,  // DXT1 - 8 bytes per 4x4 block
    Bc2,  // DXT3 - 16 bytes per 4x4 block (explicit alpha)
    Bc3,  // DXT5 - 16 bytes per 4x4 block (interpolated alpha)
    Bc4,  // Single channel - 8 bytes per 4x4 block
    Bc5,  // Two channels - 16 bytes per 4x4 block
    Bc7,  // High quality - 16 bytes per 4x4 block
}

impl BcFormat {
    /// Block size in bytes for this format
    const fn block_size(self) -> usize {
        match self {
            Self::Bc1 | Self::Bc4 => 8,
            Self::Bc2 | Self::Bc3 | Self::Bc5 | Self::Bc7 => 16,
        }
    }
}

/// Decode BC-compressed texture data to RGBA using bcdec_rs
fn decode_bc(data: &[u8], width: usize, height: usize, format: BcFormat) -> Result<Vec<u8>> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);
    let block_size = format.block_size();

    // Temporary buffer for a single 4x4 block (16 pixels * 4 bytes = 64 bytes)
    // Pitch is 4 pixels * 4 bytes per pixel = 16 bytes per row
    let mut block_rgba = [0u8; 64];
    let block_pitch = 16;

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) * block_size;
            if block_idx + block_size > data.len() {
                break;
            }
            let block = &data[block_idx..block_idx + block_size];

            // Decode the 4x4 block using bcdec_rs
            match format {
                BcFormat::Bc1 => bcdec_rs::bc1(block, &mut block_rgba, block_pitch),
                BcFormat::Bc2 => bcdec_rs::bc2(block, &mut block_rgba, block_pitch),
                BcFormat::Bc3 => bcdec_rs::bc3(block, &mut block_rgba, block_pitch),
                BcFormat::Bc4 => bcdec_rs::bc4(block, &mut block_rgba, block_pitch, false), // unsigned
                BcFormat::Bc5 => bcdec_rs::bc5(block, &mut block_rgba, block_pitch, false), // unsigned
                BcFormat::Bc7 => bcdec_rs::bc7(block, &mut block_rgba, block_pitch),
            }

            // Copy decoded pixels to output
            for py in 0..4 {
                for px in 0..4 {
                    let fx = bx * 4 + px;
                    let fy = by * 4 + py;
                    if fx >= width || fy >= height {
                        continue;
                    }
                    let src_idx = (py * 4 + px) * 4;
                    let dst_idx = (fy * width + fx) * 4;
                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&block_rgba[src_idx..src_idx + 4]);
                }
            }
        }
    }

    Ok(rgba)
}

// ============================================================================
// BC6H (HDR) format decoder
// ============================================================================

/// Decode BC6H HDR textures to RGBA8 (tone-mapped from float)
fn decode_bc6h(data: &[u8], width: usize, height: usize, signed: bool) -> Result<Vec<u8>> {
    const BLOCK_SIZE: usize = 16;
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);

    // Temporary buffer for a single 4x4 block of RGB floats (16 pixels * 3 floats * 4 bytes = 192 bytes)
    let mut block_rgb = [0f32; 48];
    let block_pitch = 4 * 3; // 4 pixels * 3 floats per row

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) * BLOCK_SIZE;
            if block_idx + BLOCK_SIZE > data.len() {
                break;
            }
            let block = &data[block_idx..block_idx + BLOCK_SIZE];

            // Decode BC6H to RGB float
            bcdec_rs::bc6h_float(block, &mut block_rgb, block_pitch, signed);

            // Copy and tone-map to RGBA8
            for py in 0..4 {
                for px in 0..4 {
                    let fx = bx * 4 + px;
                    let fy = by * 4 + py;
                    if fx >= width || fy >= height {
                        continue;
                    }
                    let src_idx = (py * 4 + px) * 3;
                    let dst_idx = (fy * width + fx) * 4;

                    // Simple Reinhard tone mapping and gamma correction
                    let r = tone_map_hdr(block_rgb[src_idx]);
                    let g = tone_map_hdr(block_rgb[src_idx + 1]);
                    let b = tone_map_hdr(block_rgb[src_idx + 2]);

                    rgba[dst_idx] = r;
                    rgba[dst_idx + 1] = g;
                    rgba[dst_idx + 2] = b;
                    rgba[dst_idx + 3] = 255; // Opaque alpha
                }
            }
        }
    }

    Ok(rgba)
}

/// Tone map HDR float value to 8-bit using Reinhard operator
#[allow(clippy::cast_sign_loss)] // Value is clamped to 0.0-255.0, always non-negative
fn tone_map_hdr(value: f32) -> u8 {
    // Clamp negative values
    let v = value.max(0.0);
    // Reinhard tone mapping: v / (1 + v)
    let mapped = v / (1.0 + v);
    // Apply gamma correction (sRGB ~= 2.2)
    let gamma_corrected = mapped.powf(1.0 / 2.2);
    // Convert to 8-bit
    (gamma_corrected * 255.0).clamp(0.0, 255.0) as u8
}

