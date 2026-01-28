//! DDS ↔ PNG texture conversion
//!
//! Converts between DDS (`DirectDraw` Surface) texture files and PNG images.
//! Supports common DDS formats used in BG3: BC1, BC2, BC3, BC4, BC5, BC7, and uncompressed.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`
//!
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(clippy::cast_possible_truncation)]

mod decode;
mod encode;
pub mod types;

use crate::error::{Error, Result};
use ddsfile::Dds;
use image::{DynamicImage, ImageBuffer, RgbaImage};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub use encode::DdsFormat;
pub use types::{ImagePhase, ImageProgress, ImageProgressCallback};

/// Convert a DDS file to PNG
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_dds_to_png<P: AsRef<Path>, Q: AsRef<Path>>(dds_path: P, png_path: Q) -> Result<()> {
    convert_dds_to_png_with_progress(dds_path, png_path, &|_| {})
}

/// Convert a DDS file to PNG with progress callback
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_dds_to_png_with_progress<P: AsRef<Path>, Q: AsRef<Path>>(
    dds_path: P,
    png_path: Q,
    progress: ImageProgressCallback,
) -> Result<()> {
    progress(&ImageProgress::with_file(
        ImagePhase::ReadingFile,
        1,
        4,
        dds_path.as_ref().display().to_string(),
    ));
    let file = File::open(dds_path.as_ref())?;
    let mut reader = BufReader::new(file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

    progress(&ImageProgress::new(ImagePhase::Decoding, 2, 4));
    let dds = Dds::read(&mut std::io::Cursor::new(&data))
        .map_err(|e| Error::DdsError(format!("Failed to parse DDS: {e}")))?;
    let rgba = decode::decode_dds_to_rgba(&dds)?;

    progress(&ImageProgress::new(ImagePhase::Encoding, 3, 4));
    let img: RgbaImage = ImageBuffer::from_raw(dds.get_width(), dds.get_height(), rgba)
        .ok_or_else(|| Error::DdsError("Failed to create image buffer".to_string()))?;
    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    img.write_with_encoder(encoder)
        .map_err(|e| Error::DdsError(format!("Failed to encode PNG: {e}")))?;

    progress(&ImageProgress::with_file(
        ImagePhase::WritingFile,
        4,
        4,
        png_path.as_ref().display().to_string(),
    ));
    let mut output = BufWriter::new(File::create(png_path.as_ref())?);
    output.write_all(&png_data)?;

    progress(&ImageProgress::new(ImagePhase::Complete, 4, 4));
    Ok(())
}

/// Convert DDS bytes to PNG bytes
///
/// # Errors
/// Returns an error if the DDS data cannot be parsed or decoded.
pub fn dds_bytes_to_png_bytes(dds_data: &[u8]) -> Result<Vec<u8>> {
    let dds = Dds::read(&mut std::io::Cursor::new(dds_data))
        .map_err(|e| Error::DdsError(format!("Failed to parse DDS: {e}")))?;

    let rgba = decode::decode_dds_to_rgba(&dds)?;

    let img: RgbaImage = ImageBuffer::from_raw(dds.get_width(), dds.get_height(), rgba)
        .ok_or_else(|| Error::DdsError("Failed to create image buffer".to_string()))?;

    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    img.write_with_encoder(encoder)
        .map_err(|e| Error::DdsError(format!("Failed to encode PNG: {e}")))?;

    Ok(png_data)
}

/// Convert a PNG file to DDS with default BC3 compression
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_png_to_dds<P: AsRef<Path>, Q: AsRef<Path>>(png_path: P, dds_path: Q) -> Result<()> {
    convert_png_to_dds_with_format(png_path, dds_path, DdsFormat::BC3)
}

/// Convert a PNG file to DDS with progress callback
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_png_to_dds_with_progress<P: AsRef<Path>, Q: AsRef<Path>>(
    png_path: P,
    dds_path: Q,
    progress: ImageProgressCallback,
) -> Result<()> {
    convert_png_to_dds_with_format_and_progress(png_path, dds_path, DdsFormat::BC3, progress)
}

/// Convert a PNG file to DDS with specified compression format
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_png_to_dds_with_format<P: AsRef<Path>, Q: AsRef<Path>>(
    png_path: P,
    dds_path: Q,
    format: DdsFormat,
) -> Result<()> {
    convert_png_to_dds_with_format_and_progress(png_path, dds_path, format, &|_| {})
}

/// Convert a PNG file to DDS with specified compression format and progress callback
///
/// # Errors
/// Returns an error if the file cannot be read or conversion fails.
pub fn convert_png_to_dds_with_format_and_progress<P: AsRef<Path>, Q: AsRef<Path>>(
    png_path: P,
    dds_path: Q,
    format: DdsFormat,
    progress: ImageProgressCallback,
) -> Result<()> {
    progress(&ImageProgress::with_file(
        ImagePhase::ReadingFile,
        1,
        4,
        png_path.as_ref().display().to_string(),
    ));
    let img = image::open(png_path.as_ref())
        .map_err(|e| Error::DdsError(format!("Failed to open PNG: {e}")))?;

    progress(&ImageProgress::new(ImagePhase::Decoding, 2, 4));
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let pixels = rgba.as_raw();

    progress(&ImageProgress::with_file(
        ImagePhase::Encoding,
        3,
        4,
        format!("{format:?}"),
    ));
    let dds_data = encode::encode_to_dds(pixels, width, height, format)?;

    progress(&ImageProgress::with_file(
        ImagePhase::WritingFile,
        4,
        4,
        dds_path.as_ref().display().to_string(),
    ));
    let mut output = BufWriter::new(File::create(dds_path.as_ref())?);
    output.write_all(&dds_data)?;

    progress(&ImageProgress::new(ImagePhase::Complete, 4, 4));
    Ok(())
}

/// Convert a PNG image to DDS bytes with specified format
///
/// # Errors
/// Returns an error if encoding fails.
pub fn png_image_to_dds_bytes(img: &DynamicImage, format: DdsFormat) -> Result<Vec<u8>> {
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let pixels = rgba.as_raw();

    encode::encode_to_dds(pixels, width, height, format)
}
