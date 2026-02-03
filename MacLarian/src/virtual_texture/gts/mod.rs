//! GTS (Game Texture Set) file reader
//!
//! GTS files contain metadata about virtual textures, including:
//! - Tile dimensions and layout
//! - Parameter blocks for each codec type
//! - Page file references
//! - Tile mapping information

#![allow(
    clippy::cast_possible_truncation,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::used_underscore_binding,
    clippy::doc_markdown,
    clippy::missing_panics_doc
)]

mod accessors;
mod read_header;
mod read_sections;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

use super::types::{
    GtsFlatTileInfo, GtsHeader, GtsLevelInfo, GtsPackedTileId, GtsPageFileInfo, GtsParameterBlock,
};
use crate::error::{Error, Result};

/// GTS file reader and parser.
#[derive(Debug)]
pub struct GtsFile {
    pub(crate) header: GtsHeader,
    pub(crate) parameter_blocks: HashMap<u32, GtsParameterBlock>,
    /// Level info parsed from file. Currently tile lookup uses packed tile IDs directly.
    #[allow(dead_code)]
    pub(crate) levels: Vec<GtsLevelInfo>,
    pub(crate) page_files: Vec<GtsPageFileInfo>,
    pub(crate) packed_tiles: Vec<GtsPackedTileId>,
    pub(crate) flat_tile_infos: Vec<GtsFlatTileInfo>,
}

impl GtsFile {
    /// Read and parse a GTS file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or has an invalid format.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read(&mut reader)
    }

    /// Read and parse GTS from a reader.
    ///
    /// # Errors
    /// Returns an error if reading fails or the data has an invalid format.
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let header = read_header::read_header(reader)?;

        if header.magic != GtsHeader::MAGIC {
            return Err(Error::InvalidGtsMagic);
        }

        // Read parameter blocks
        let parameter_blocks = read_sections::read_parameter_blocks(reader, &header)?;

        // Read levels
        let levels = read_sections::read_levels(reader, &header)?;

        // Read page file metadata
        let page_files = read_sections::read_page_files(reader, &header)?;

        // Read packed tile IDs
        let packed_tiles = read_sections::read_packed_tiles(reader, &header)?;

        // Read flat tile infos
        let flat_tile_infos = read_sections::read_flat_tile_infos(reader, &header)?;

        Ok(Self {
            header,
            parameter_blocks,
            levels,
            page_files,
            packed_tiles,
            flat_tile_infos,
        })
    }
}
