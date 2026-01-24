//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! DDS decoding - Block Compression (BC) decompression

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
        DxgiFormat::BC1_UNorm | DxgiFormat::BC1_UNorm_sRGB => decode_bc1(data, width, height),
        DxgiFormat::BC2_UNorm | DxgiFormat::BC2_UNorm_sRGB => decode_bc2(data, width, height),
        DxgiFormat::BC3_UNorm | DxgiFormat::BC3_UNorm_sRGB => decode_bc3(data, width, height),
        DxgiFormat::BC4_UNorm => decode_bc4(data, width, height),
        DxgiFormat::BC5_UNorm => decode_bc5(data, width, height),
        DxgiFormat::BC7_UNorm | DxgiFormat::BC7_UNorm_sRGB => decode_bc7(data, width, height),
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
        D3DFormat::DXT1 => decode_bc1(data, width, height),
        D3DFormat::DXT3 => decode_bc2(data, width, height),
        D3DFormat::DXT5 => decode_bc3(data, width, height),
        _ => Err(Error::DdsError(format!(
            "Unsupported D3D format: {format:?}"
        ))),
    }
}

// ============================================================================
// BC1 (DXT1) - 4x4 blocks, 8 bytes each
// ============================================================================

fn decode_bc1(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) * 8;
            if block_idx + 8 > data.len() {
                break;
            }
            let block = &data[block_idx..block_idx + 8];
            decode_bc1_block(block, &mut rgba, bx * 4, by * 4, width, height);
        }
    }

    Ok(rgba)
}

fn decode_bc1_block(
    block: &[u8],
    rgba: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);

    let colors = bc1_colors(c0, c1);
    let indices = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);

    for py in 0..4 {
        for px in 0..4 {
            let fx = x + px;
            let fy = y + py;
            if fx >= width || fy >= height {
                continue;
            }
            let idx = ((py * 4 + px) * 2) as u32;
            let color_idx = ((indices >> idx) & 0x3) as usize;
            let pixel_idx = (fy * width + fx) * 4;
            rgba[pixel_idx..pixel_idx + 4].copy_from_slice(&colors[color_idx]);
        }
    }
}

pub fn bc1_colors(c0: u16, c1: u16) -> [[u8; 4]; 4] {
    let r0 = ((c0 >> 11) & 0x1F) as u8;
    let g0 = ((c0 >> 5) & 0x3F) as u8;
    let b0 = (c0 & 0x1F) as u8;

    let r1 = ((c1 >> 11) & 0x1F) as u8;
    let g1 = ((c1 >> 5) & 0x3F) as u8;
    let b1 = (c1 & 0x1F) as u8;

    let expand5 = |v: u8| (v << 3) | (v >> 2);
    let expand6 = |v: u8| (v << 2) | (v >> 4);

    let r0 = expand5(r0);
    let g0 = expand6(g0);
    let b0 = expand5(b0);
    let r1 = expand5(r1);
    let g1 = expand6(g1);
    let b1 = expand5(b1);

    if c0 > c1 {
        [
            [r0, g0, b0, 255],
            [r1, g1, b1, 255],
            [
                ((2 * u16::from(r0) + u16::from(r1)) / 3) as u8,
                ((2 * u16::from(g0) + u16::from(g1)) / 3) as u8,
                ((2 * u16::from(b0) + u16::from(b1)) / 3) as u8,
                255,
            ],
            [
                ((u16::from(r0) + 2 * u16::from(r1)) / 3) as u8,
                ((u16::from(g0) + 2 * u16::from(g1)) / 3) as u8,
                ((u16::from(b0) + 2 * u16::from(b1)) / 3) as u8,
                255,
            ],
        ]
    } else {
        [
            [r0, g0, b0, 255],
            [r1, g1, b1, 255],
            [
                u16::midpoint(u16::from(r0), u16::from(r1)) as u8,
                u16::midpoint(u16::from(g0), u16::from(g1)) as u8,
                u16::midpoint(u16::from(b0), u16::from(b1)) as u8,
                255,
            ],
            [0, 0, 0, 0], // Transparent
        ]
    }
}

// ============================================================================
// BC2 (DXT3) - 4x4 blocks, 16 bytes each (explicit alpha)
// ============================================================================

fn decode_bc2(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) * 16;
            if block_idx + 16 > data.len() {
                break;
            }
            let block = &data[block_idx..block_idx + 16];
            decode_bc2_block(block, &mut rgba, bx * 4, by * 4, width, height);
        }
    }

    Ok(rgba)
}

fn decode_bc2_block(
    block: &[u8],
    rgba: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    // First 8 bytes: explicit alpha (4 bits per pixel)
    let alpha_block = &block[0..8];
    // Last 8 bytes: BC1 color block
    let color_block = &block[8..16];

    let c0 = u16::from_le_bytes([color_block[0], color_block[1]]);
    let c1 = u16::from_le_bytes([color_block[2], color_block[3]]);
    let colors = bc1_colors(c0, c1);
    let indices = u32::from_le_bytes([color_block[4], color_block[5], color_block[6], color_block[7]]);

    for py in 0..4 {
        for px in 0..4 {
            let fx = x + px;
            let fy = y + py;
            if fx >= width || fy >= height {
                continue;
            }

            let idx = ((py * 4 + px) * 2) as u32;
            let color_idx = ((indices >> idx) & 0x3) as usize;
            let pixel_idx = (fy * width + fx) * 4;

            // Get alpha from explicit block
            let alpha_idx = py * 2 + px / 2;
            let alpha_shift = (px % 2) * 4;
            let alpha = ((alpha_block[alpha_idx] >> alpha_shift) & 0xF) * 17; // Expand 4-bit to 8-bit

            rgba[pixel_idx] = colors[color_idx][0];
            rgba[pixel_idx + 1] = colors[color_idx][1];
            rgba[pixel_idx + 2] = colors[color_idx][2];
            rgba[pixel_idx + 3] = alpha;
        }
    }
}

// ============================================================================
// BC3 (DXT5) - 4x4 blocks, 16 bytes each (interpolated alpha)
// ============================================================================

fn decode_bc3(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) * 16;
            if block_idx + 16 > data.len() {
                break;
            }
            let block = &data[block_idx..block_idx + 16];
            decode_bc3_block(block, &mut rgba, bx * 4, by * 4, width, height);
        }
    }

    Ok(rgba)
}

fn decode_bc3_block(
    block: &[u8],
    rgba: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    // First 8 bytes: interpolated alpha
    let alpha0 = block[0];
    let alpha1 = block[1];
    let alphas = bc3_alphas(alpha0, alpha1);

    let alpha_indices = u64::from_le_bytes([
        block[2], block[3], block[4], block[5], block[6], block[7], 0, 0,
    ]);

    // Last 8 bytes: BC1 color block
    let color_block = &block[8..16];
    let c0 = u16::from_le_bytes([color_block[0], color_block[1]]);
    let c1 = u16::from_le_bytes([color_block[2], color_block[3]]);
    let colors = bc1_colors(c0, c1);
    let indices = u32::from_le_bytes([color_block[4], color_block[5], color_block[6], color_block[7]]);

    for py in 0..4 {
        for px in 0..4 {
            let fx = x + px;
            let fy = y + py;
            if fx >= width || fy >= height {
                continue;
            }

            let idx = ((py * 4 + px) * 2) as u32;
            let color_idx = ((indices >> idx) & 0x3) as usize;
            let pixel_idx = (fy * width + fx) * 4;

            let alpha_bit_idx = (py * 4 + px) * 3;
            let alpha_idx = ((alpha_indices >> alpha_bit_idx) & 0x7) as usize;

            rgba[pixel_idx] = colors[color_idx][0];
            rgba[pixel_idx + 1] = colors[color_idx][1];
            rgba[pixel_idx + 2] = colors[color_idx][2];
            rgba[pixel_idx + 3] = alphas[alpha_idx];
        }
    }
}

pub fn bc3_alphas(a0: u8, a1: u8) -> [u8; 8] {
    if a0 > a1 {
        [
            a0,
            a1,
            ((6 * u16::from(a0) + u16::from(a1)) / 7) as u8,
            ((5 * u16::from(a0) + 2 * u16::from(a1)) / 7) as u8,
            ((4 * u16::from(a0) + 3 * u16::from(a1)) / 7) as u8,
            ((3 * u16::from(a0) + 4 * u16::from(a1)) / 7) as u8,
            ((2 * u16::from(a0) + 5 * u16::from(a1)) / 7) as u8,
            ((u16::from(a0) + 6 * u16::from(a1)) / 7) as u8,
        ]
    } else {
        [
            a0,
            a1,
            ((4 * u16::from(a0) + u16::from(a1)) / 5) as u8,
            ((3 * u16::from(a0) + 2 * u16::from(a1)) / 5) as u8,
            ((2 * u16::from(a0) + 3 * u16::from(a1)) / 5) as u8,
            ((u16::from(a0) + 4 * u16::from(a1)) / 5) as u8,
            0,
            255,
        ]
    }
}

// ============================================================================
// BC4 - Single channel, 4x4 blocks, 8 bytes each
// ============================================================================

fn decode_bc4(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) * 8;
            if block_idx + 8 > data.len() {
                break;
            }
            let block = &data[block_idx..block_idx + 8];
            decode_bc4_block(block, &mut rgba, bx * 4, by * 4, width, height);
        }
    }

    Ok(rgba)
}

fn decode_bc4_block(
    block: &[u8],
    rgba: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    let r0 = block[0];
    let r1 = block[1];
    let values = bc3_alphas(r0, r1); // Same interpolation as BC3 alpha

    let indices = u64::from_le_bytes([
        block[2], block[3], block[4], block[5], block[6], block[7], 0, 0,
    ]);

    for py in 0..4 {
        for px in 0..4 {
            let fx = x + px;
            let fy = y + py;
            if fx >= width || fy >= height {
                continue;
            }

            let bit_idx = (py * 4 + px) * 3;
            let val_idx = ((indices >> bit_idx) & 0x7) as usize;
            let pixel_idx = (fy * width + fx) * 4;

            let v = values[val_idx];
            rgba[pixel_idx] = v;
            rgba[pixel_idx + 1] = v;
            rgba[pixel_idx + 2] = v;
            rgba[pixel_idx + 3] = 255;
        }
    }
}

// ============================================================================
// BC5 - Two channels (normal maps), 4x4 blocks, 16 bytes each
// ============================================================================

fn decode_bc5(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block_idx = (by * blocks_x + bx) * 16;
            if block_idx + 16 > data.len() {
                break;
            }
            let block = &data[block_idx..block_idx + 16];
            decode_bc5_block(block, &mut rgba, bx * 4, by * 4, width, height);
        }
    }

    Ok(rgba)
}

fn decode_bc5_block(
    block: &[u8],
    rgba: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    // Red channel
    let r0 = block[0];
    let r1 = block[1];
    let r_values = bc3_alphas(r0, r1);
    let r_indices = u64::from_le_bytes([block[2], block[3], block[4], block[5], block[6], block[7], 0, 0]);

    // Green channel
    let g0 = block[8];
    let g1 = block[9];
    let g_values = bc3_alphas(g0, g1);
    let g_indices = u64::from_le_bytes([
        block[10], block[11], block[12], block[13], block[14], block[15], 0, 0,
    ]);

    for py in 0..4 {
        for px in 0..4 {
            let fx = x + px;
            let fy = y + py;
            if fx >= width || fy >= height {
                continue;
            }

            let bit_idx = (py * 4 + px) * 3;
            let r_idx = ((r_indices >> bit_idx) & 0x7) as usize;
            let g_idx = ((g_indices >> bit_idx) & 0x7) as usize;
            let pixel_idx = (fy * width + fx) * 4;

            rgba[pixel_idx] = r_values[r_idx];
            rgba[pixel_idx + 1] = g_values[g_idx];
            rgba[pixel_idx + 2] = 128; // Blue channel typically reconstructed from R,G
            rgba[pixel_idx + 3] = 255;
        }
    }
}

// ============================================================================
// BC7 - High quality, 4x4 blocks, 16 bytes each (8 modes)
// ============================================================================

fn decode_bc7(_data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    // BC7 is complex with 8 different modes. For now, return a placeholder.
    // TODO: Implement full BC7 decoding or use an external crate
    let mut rgba = vec![128u8; width * height * 4];

    // Set alpha to 255 for all pixels
    for i in (3..rgba.len()).step_by(4) {
        rgba[i] = 255;
    }

    // Note: Full BC7 implementation is complex. Consider using bcdec or similar.
    tracing::warn!("BC7 decoding not fully implemented - returning placeholder");

    Ok(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc1_color_expansion() {
        let colors = bc1_colors(0xFFFF, 0x0000);
        assert_eq!(colors[0], [255, 255, 255, 255]); // White
        assert_eq!(colors[1], [0, 0, 0, 255]); // Black
    }
}
