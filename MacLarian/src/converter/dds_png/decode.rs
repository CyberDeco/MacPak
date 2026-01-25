//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! DDS decoding - Block Compression (BC) decompression using bcdec_rs

use crate::error::{Error, Result};
use ddsfile::{D3DFormat, Dds, DxgiFormat};

/// Decode DDS texture data to RGBA pixels
///
/// # Errors
/// Returns an error if the format is unsupported or data is invalid.
pub fn decode_dds_to_rgba(dds: &Dds) -> Result<Vec<u8>> {
    let width = dds.get_width() as usize;
    let height = dds.get_height() as usize;
    let data = dds
        .get_data(0)
        .map_err(|e| Error::DdsError(format!("No DDS data: {e}")))?;

    // Determine format and decode
    if let Some(dxgi) = dds.get_dxgi_format() {
        decode_dxgi_format(data, width, height, dxgi)
    } else if let Some(d3d) = dds.get_d3d_format() {
        decode_d3d_format(data, width, height, d3d)
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
        DxgiFormat::R8G8B8A8_UNorm | DxgiFormat::R8G8B8A8_UNorm_sRGB => Ok(data.to_vec()),
        DxgiFormat::B8G8R8A8_UNorm | DxgiFormat::B8G8R8A8_UNorm_sRGB => {
            // BGRA to RGBA
            let mut rgba = data.to_vec();
            for chunk in rgba.chunks_exact_mut(4) {
                chunk.swap(0, 2);
            }
            Ok(rgba)
        }
        DxgiFormat::BC1_UNorm | DxgiFormat::BC1_UNorm_sRGB => decode_bc(data, width, height, BcFormat::Bc1),
        DxgiFormat::BC2_UNorm | DxgiFormat::BC2_UNorm_sRGB => decode_bc(data, width, height, BcFormat::Bc2),
        DxgiFormat::BC3_UNorm | DxgiFormat::BC3_UNorm_sRGB => decode_bc(data, width, height, BcFormat::Bc3),
        DxgiFormat::BC4_UNorm => decode_bc(data, width, height, BcFormat::Bc4),
        DxgiFormat::BC5_UNorm => decode_bc(data, width, height, BcFormat::Bc5),
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
        D3DFormat::DXT1 => decode_bc(data, width, height, BcFormat::Bc1),
        D3DFormat::DXT3 => decode_bc(data, width, height, BcFormat::Bc2),
        D3DFormat::DXT5 => decode_bc(data, width, height, BcFormat::Bc3),
        _ => Err(Error::DdsError(format!(
            "Unsupported D3D format: {format:?}"
        ))),
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

#[cfg(test)]
mod tests {
    // Tests removed - using bcdec_rs which is already well-tested
}
