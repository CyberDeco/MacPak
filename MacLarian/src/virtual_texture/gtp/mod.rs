//! GTP (Game Texture Page) file reader
//!
//! GTP files contain the actual tile data for virtual textures.
//! Each file consists of pages, and each page contains multiple chunks (tiles).
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

mod accessors;
mod extract;
mod read_header;

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use super::gts::GtsFile;
use super::types::GtpHeader;
use crate::error::{Error, Result};

/// GTP file reader.
pub struct GtpFile<R: Read + Seek> {
    reader: BufReader<R>,
    pub header: GtpHeader,
    /// Chunk offsets for each page, indexed by page then chunk.
    pub chunk_offsets: Vec<Vec<u32>>,
    page_size: u32,
    tile_width: i32,
    tile_height: i32,
}

impl GtpFile<File> {
    /// Open a GTP file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or has an invalid format.
    pub fn open<P: AsRef<Path>>(path: P, gts: &GtsFile) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        Self::new(file, gts)
    }
}

impl<R: Read + Seek> GtpFile<R> {
    /// Create a new GTP reader from any Read + Seek source.
    ///
    /// # Errors
    /// Returns an error if reading fails or the data has an invalid format.
    pub fn new(reader: R, gts: &GtsFile) -> Result<Self> {
        let mut reader = BufReader::new(reader);

        // Get file size
        let file_size = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        // Read header
        let header = read_header::read_header(&mut reader)?;

        if header.magic != GtpHeader::MAGIC {
            let magic = header.magic;
            let expected = GtpHeader::MAGIC;
            return Err(Error::ConversionError(format!(
                "Invalid GTP magic: 0x{magic:08X}, expected 0x{expected:08X}"
            )));
        }

        let page_size = gts.header.page_size;
        let num_pages = (file_size / u64::from(page_size)) as usize;

        // Read chunk offsets for each page
        let chunk_offsets = read_header::read_chunk_offsets(&mut reader, page_size, num_pages)?;

        Ok(Self {
            reader,
            header,
            chunk_offsets,
            page_size,
            tile_width: gts.header.tile_width,
            tile_height: gts.header.tile_height,
        })
    }
}
