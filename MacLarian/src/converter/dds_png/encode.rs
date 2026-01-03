//! DDS encoding - Block Compression (BC) compression

use crate::error::{Error, Result};
use ddsfile::{AlphaMode, D3DFormat, Dds, DxgiFormat, NewDxgiParams};

use super::decode::bc1_colors;

/// DDS compression format for PNG to DDS conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdsFormat {
    /// BC1/DXT1 - Good for opaque textures or 1-bit alpha
    BC1,
    /// BC2/DXT3 - Explicit 4-bit alpha, good for sharp alpha transitions
    BC2,
    /// BC3/DXT5 - Interpolated alpha, good for smooth alpha gradients
    BC3,
    /// Uncompressed RGBA
    Rgba,
}

/// Encode RGBA pixels to DDS with specified format
pub fn encode_to_dds(pixels: &[u8], width: u32, height: u32, format: DdsFormat) -> Result<Vec<u8>> {
    match format {
        DdsFormat::BC1 => encode_bc1_dds(pixels, width, height),
        DdsFormat::BC2 => encode_bc2_dds(pixels, width, height),
        DdsFormat::BC3 => encode_bc3_dds(pixels, width, height),
        DdsFormat::Rgba => encode_rgba_dds(pixels, width, height),
    }
}

/// Encode as uncompressed RGBA DDS
fn encode_rgba_dds(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut dds = Dds::new_dxgi(NewDxgiParams {
        height,
        width,
        depth: None,
        format: DxgiFormat::R8G8B8A8_UNorm,
        mipmap_levels: None,
        array_layers: None,
        caps2: None,
        is_cubemap: false,
        resource_dimension: ddsfile::D3D10ResourceDimension::Texture2D,
        alpha_mode: AlphaMode::Straight,
    })
    .map_err(|e| Error::DdsError(format!("Failed to create DDS: {e}")))?;

    let data = dds
        .get_mut_data(0)
        .map_err(|e| Error::DdsError(format!("No DDS data layer: {e}")))?;
    data.copy_from_slice(pixels);

    let mut output = Vec::new();
    dds.write(&mut output)
        .map_err(|e| Error::DdsError(format!("Failed to write DDS: {e}")))?;

    Ok(output)
}

/// Encode as BC1/DXT1 DDS
fn encode_bc1_dds(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let compressed = encode_bc1(pixels, width as usize, height as usize);
    build_dds_with_d3d_format(width, height, D3DFormat::DXT1, &compressed)
}

/// Encode as BC2/DXT3 DDS
fn encode_bc2_dds(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let compressed = encode_bc2(pixels, width as usize, height as usize);
    build_dds_with_d3d_format(width, height, D3DFormat::DXT3, &compressed)
}

/// Encode as BC3/DXT5 DDS
fn encode_bc3_dds(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let compressed = encode_bc3(pixels, width as usize, height as usize);
    build_dds_with_d3d_format(width, height, D3DFormat::DXT5, &compressed)
}

/// Build a DDS file with D3D format (DXT1/3/5)
fn build_dds_with_d3d_format(
    width: u32,
    height: u32,
    format: D3DFormat,
    data: &[u8],
) -> Result<Vec<u8>> {
    let mut dds = Dds::new_d3d(ddsfile::NewD3dParams {
        height,
        width,
        depth: None,
        format,
        mipmap_levels: None,
        caps2: None,
    })
    .map_err(|e| Error::DdsError(format!("Failed to create DDS: {e}")))?;

    let dds_data = dds
        .get_mut_data(0)
        .map_err(|e| Error::DdsError(format!("No DDS data layer: {e}")))?;
    dds_data.copy_from_slice(data);

    let mut output = Vec::new();
    dds.write(&mut output)
        .map_err(|e| Error::DdsError(format!("Failed to write DDS: {e}")))?;

    Ok(output)
}

// ============================================================================
// BC1 (DXT1) Encoding
// ============================================================================

/// Encode RGBA pixels to BC1 (DXT1) format
fn encode_bc1(pixels: &[u8], width: usize, height: usize) -> Vec<u8> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut output = vec![0u8; blocks_x * blocks_y * 8];

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block = extract_block(pixels, width, height, bx * 4, by * 4);
            let encoded = encode_bc1_block(&block);
            let offset = (by * blocks_x + bx) * 8;
            output[offset..offset + 8].copy_from_slice(&encoded);
        }
    }

    output
}

/// Encode a 4x4 block to BC1 (8 bytes)
fn encode_bc1_block(block: &[[u8; 4]; 16]) -> [u8; 8] {
    // Find min/max colors using principal axis
    let (c0, c1) = find_endpoint_colors(block);

    // Ensure c0 > c1 for 4-color mode (no transparency)
    let (c0_565, c1_565) = if c0 >= c1 { (c0, c1) } else { (c1, c0) };

    // Generate palette
    let colors = bc1_encode_colors(c0_565, c1_565);

    // Find best index for each pixel
    let mut indices: u32 = 0;
    for (i, pixel) in block.iter().enumerate() {
        let best_idx = find_closest_color(pixel, &colors);
        indices |= (best_idx as u32) << (i * 2);
    }

    // Pack output
    let mut output = [0u8; 8];
    output[0..2].copy_from_slice(&c0_565.to_le_bytes());
    output[2..4].copy_from_slice(&c1_565.to_le_bytes());
    output[4..8].copy_from_slice(&indices.to_le_bytes());

    output
}

// ============================================================================
// BC2 (DXT3) Encoding
// ============================================================================

/// Encode RGBA pixels to BC2 (DXT3) format
fn encode_bc2(pixels: &[u8], width: usize, height: usize) -> Vec<u8> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut output = vec![0u8; blocks_x * blocks_y * 16];

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block = extract_block(pixels, width, height, bx * 4, by * 4);
            let encoded = encode_bc2_block(&block);
            let offset = (by * blocks_x + bx) * 16;
            output[offset..offset + 16].copy_from_slice(&encoded);
        }
    }

    output
}

/// Encode a 4x4 block to BC2 (16 bytes: 8 alpha + 8 color)
fn encode_bc2_block(block: &[[u8; 4]; 16]) -> [u8; 16] {
    let mut output = [0u8; 16];

    // First 8 bytes: explicit 4-bit alpha for each pixel
    for i in 0..16 {
        let alpha_4bit = block[i][3] >> 4; // Convert 8-bit to 4-bit
        let byte_idx = i / 2;
        let shift = (i % 2) * 4;
        output[byte_idx] |= alpha_4bit << shift;
    }

    // Last 8 bytes: BC1 color block
    let color_block = encode_bc1_block(block);
    output[8..16].copy_from_slice(&color_block);

    output
}

// ============================================================================
// BC3 (DXT5) Encoding
// ============================================================================

/// Encode RGBA pixels to BC3 (DXT5) format
fn encode_bc3(pixels: &[u8], width: usize, height: usize) -> Vec<u8> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut output = vec![0u8; blocks_x * blocks_y * 16];

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let block = extract_block(pixels, width, height, bx * 4, by * 4);
            let encoded = encode_bc3_block(&block);
            let offset = (by * blocks_x + bx) * 16;
            output[offset..offset + 16].copy_from_slice(&encoded);
        }
    }

    output
}

/// Encode a 4x4 block to BC3 (16 bytes: 8 alpha + 8 color)
fn encode_bc3_block(block: &[[u8; 4]; 16]) -> [u8; 16] {
    let mut output = [0u8; 16];

    // First 8 bytes: interpolated alpha block
    let alpha_block = encode_bc3_alpha_block(block);
    output[0..8].copy_from_slice(&alpha_block);

    // Last 8 bytes: BC1 color block
    let color_block = encode_bc1_block(block);
    output[8..16].copy_from_slice(&color_block);

    output
}

/// Encode alpha channel for BC3 (8 bytes)
fn encode_bc3_alpha_block(block: &[[u8; 4]; 16]) -> [u8; 8] {
    // Find min/max alpha
    let mut min_alpha = 255u8;
    let mut max_alpha = 0u8;
    for pixel in block {
        min_alpha = min_alpha.min(pixel[3]);
        max_alpha = max_alpha.max(pixel[3]);
    }

    // Use 8-value interpolation (a0 > a1)
    let a0 = max_alpha;
    let a1 = min_alpha;

    // Generate alpha palette
    let alphas = if a0 > a1 {
        [
            a0,
            a1,
            ((6 * a0 as u16 + 1 * a1 as u16) / 7) as u8,
            ((5 * a0 as u16 + 2 * a1 as u16) / 7) as u8,
            ((4 * a0 as u16 + 3 * a1 as u16) / 7) as u8,
            ((3 * a0 as u16 + 4 * a1 as u16) / 7) as u8,
            ((2 * a0 as u16 + 5 * a1 as u16) / 7) as u8,
            ((1 * a0 as u16 + 6 * a1 as u16) / 7) as u8,
        ]
    } else {
        [
            a0,
            a1,
            ((4 * a0 as u16 + 1 * a1 as u16) / 5) as u8,
            ((3 * a0 as u16 + 2 * a1 as u16) / 5) as u8,
            ((2 * a0 as u16 + 3 * a1 as u16) / 5) as u8,
            ((1 * a0 as u16 + 4 * a1 as u16) / 5) as u8,
            0,
            255,
        ]
    };

    // Find best index for each pixel
    let mut indices: u64 = 0;
    for (i, pixel) in block.iter().enumerate() {
        let alpha = pixel[3];
        let mut best_idx = 0u64;
        let mut best_dist = 256i32;
        for (j, &palette_alpha) in alphas.iter().enumerate() {
            let dist = (alpha as i32 - palette_alpha as i32).abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = j as u64;
            }
        }
        indices |= best_idx << (i * 3);
    }

    // Pack output
    let mut output = [0u8; 8];
    output[0] = a0;
    output[1] = a1;
    output[2] = (indices & 0xFF) as u8;
    output[3] = ((indices >> 8) & 0xFF) as u8;
    output[4] = ((indices >> 16) & 0xFF) as u8;
    output[5] = ((indices >> 24) & 0xFF) as u8;
    output[6] = ((indices >> 32) & 0xFF) as u8;
    output[7] = ((indices >> 40) & 0xFF) as u8;

    output
}

// ============================================================================
// Shared Helpers
// ============================================================================

/// Extract a 4x4 block of RGBA pixels, padding with edge pixels if needed
fn extract_block(pixels: &[u8], width: usize, height: usize, x: usize, y: usize) -> [[u8; 4]; 16] {
    let mut block = [[0u8; 4]; 16];

    for py in 0..4 {
        for px in 0..4 {
            let sx = (x + px).min(width - 1);
            let sy = (y + py).min(height - 1);
            let src_idx = (sy * width + sx) * 4;
            let dst_idx = py * 4 + px;

            block[dst_idx][0] = pixels[src_idx];
            block[dst_idx][1] = pixels[src_idx + 1];
            block[dst_idx][2] = pixels[src_idx + 2];
            block[dst_idx][3] = pixels[src_idx + 3];
        }
    }

    block
}

/// Find endpoint colors for BC1 encoding
fn find_endpoint_colors(block: &[[u8; 4]; 16]) -> (u16, u16) {
    // Find min/max luminance pixels as initial endpoints
    let mut min_lum = 255 * 3;
    let mut max_lum = 0;
    let mut min_pixel = [0u8; 3];
    let mut max_pixel = [0u8; 3];

    for pixel in block {
        let lum = pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32;
        if lum < min_lum {
            min_lum = lum;
            min_pixel = [pixel[0], pixel[1], pixel[2]];
        }
        if lum > max_lum {
            max_lum = lum;
            max_pixel = [pixel[0], pixel[1], pixel[2]];
        }
    }

    // Convert to RGB565
    let c0 = rgb_to_565(max_pixel[0], max_pixel[1], max_pixel[2]);
    let c1 = rgb_to_565(min_pixel[0], min_pixel[1], min_pixel[2]);

    (c0, c1)
}

/// Convert RGB888 to RGB565
pub fn rgb_to_565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = (r >> 3) as u16;
    let g6 = (g >> 2) as u16;
    let b5 = (b >> 3) as u16;
    (r5 << 11) | (g6 << 5) | b5
}

/// Generate BC1 color palette from endpoints (for encoding)
fn bc1_encode_colors(c0: u16, c1: u16) -> [[u8; 4]; 4] {
    // Use the shared decoder function
    bc1_colors(c0, c1)
}

/// Find the closest color in the palette
fn find_closest_color(pixel: &[u8; 4], palette: &[[u8; 4]; 4]) -> u8 {
    let mut best_idx = 0u8;
    let mut best_dist = u32::MAX;

    for (i, color) in palette.iter().enumerate() {
        let dr = pixel[0] as i32 - color[0] as i32;
        let dg = pixel[1] as i32 - color[1] as i32;
        let db = pixel[2] as i32 - color[2] as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;

        if dist < best_dist {
            best_dist = dist;
            best_idx = i as u8;
        }
    }

    best_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_565() {
        assert_eq!(rgb_to_565(255, 255, 255), 0xFFFF);
        assert_eq!(rgb_to_565(0, 0, 0), 0x0000);
        assert_eq!(rgb_to_565(255, 0, 0), 0xF800); // Red
        assert_eq!(rgb_to_565(0, 255, 0), 0x07E0); // Green
        assert_eq!(rgb_to_565(0, 0, 255), 0x001F); // Blue
    }
}
