//! LSF file reading and parsing
//!
//! Based on `LSLib`'s `LSFReader.cs` implementation.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0

// Binary format parsing requires many intentional casts between integer types
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)]

use super::document::{LsfDocument, LsfNode, LsfAttribute, LsfMetadataFormat};
use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

// LSF Version constants
// V1: Initial format
// V2: Added chunked/frame compression (auto-detect on read)
// V3: Extended node format (16-byte vs 12-byte)
// V4: BG3 extended header (handled implicitly)
// V6: BG3 node keys section
const LSF_VER_INITIAL: u32 = 1;
const LSF_VER_EXTENDED_NODES: u32 = 3;
const LSF_VER_BG3_NODE_KEYS: u32 = 6;

/// Read an LSF file from disk
///
/// # Errors
/// Returns an error if the file cannot be read or has an invalid format.
pub fn read_lsf<P: AsRef<Path>>(path: P) -> Result<LsfDocument> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    parse_lsf_bytes(&buffer)
}

/// Parse LSF data from bytes
///
/// # Errors
/// Returns an error if the data has an invalid LSF format.
pub fn parse_lsf_bytes(data: &[u8]) -> Result<LsfDocument> {
    let mut cursor = Cursor::new(data);

    // Read magic
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    if &magic != b"LSOF" {
        return Err(Error::InvalidLsfMagic(magic));
    }

    let version = cursor.read_u32::<LittleEndian>()?;
    if !(LSF_VER_INITIAL..=7).contains(&version) {
        return Err(Error::UnsupportedLsfVersion(version));
    }

    let engine_version = cursor.read_u64::<LittleEndian>()?;

    // Read section sizes - ORDER IS: (uncompressed_size, compressed_size) per LSLib
    let strings_uncompressed = cursor.read_u32::<LittleEndian>()? as usize;
    let strings_compressed = cursor.read_u32::<LittleEndian>()? as usize;

    // Keys section only exists in v6+
    let (keys_uncompressed, keys_compressed) = if version >= LSF_VER_BG3_NODE_KEYS {
        let u = cursor.read_u32::<LittleEndian>()? as usize;
        let c = cursor.read_u32::<LittleEndian>()? as usize;
        (u, c)
    } else {
        (0, 0)
    };

    let nodes_uncompressed = cursor.read_u32::<LittleEndian>()? as usize;
    let nodes_compressed = cursor.read_u32::<LittleEndian>()? as usize;

    let attributes_uncompressed = cursor.read_u32::<LittleEndian>()? as usize;
    let attributes_compressed = cursor.read_u32::<LittleEndian>()? as usize;

    let values_uncompressed = cursor.read_u32::<LittleEndian>()? as usize;
    let values_compressed = cursor.read_u32::<LittleEndian>()? as usize;

    let compression_flags = cursor.read_u32::<LittleEndian>()?;
    let metadata_format_raw = cursor.read_u32::<LittleEndian>()?;
    let metadata_format = LsfMetadataFormat::from(metadata_format_raw);

    // Compression method is in lower 4 bits
    let compression_method = compression_flags & 0x0F;
    let is_compressed = compression_method != 0;

    // Determine if using extended node/attribute format
    // V3+ format is only used when MetadataFormat is KeysAndAdjacency
    let has_extended_nodes = version >= LSF_VER_EXTENDED_NODES
        && metadata_format == LsfMetadataFormat::KeysAndAdjacency;

    // Read sections in FILE ORDER: Strings, Nodes, Attributes, Values, [Keys]
    let names = read_names(&mut cursor, strings_uncompressed, strings_compressed, is_compressed)?;

    // Detect node format - this also determines attribute format since they must match
    let node_extended_format = detect_extended_format(nodes_uncompressed, has_extended_nodes);

    let nodes = read_nodes(
        &mut cursor,
        nodes_uncompressed,
        nodes_compressed,
        is_compressed,
        node_extended_format,
    )?;

    // Use the same format detected for nodes - they must be consistent
    let attributes = read_attributes(
        &mut cursor,
        attributes_uncompressed,
        attributes_compressed,
        is_compressed,
        node_extended_format,
        &nodes,
    )?;

    let values = read_section(&mut cursor, values_uncompressed, values_compressed, is_compressed)?;

    // Keys section comes AFTER values (only in v6+)
    let node_keys = if version >= LSF_VER_BG3_NODE_KEYS && keys_uncompressed > 0 {
        let keys_data = read_section(&mut cursor, keys_uncompressed, keys_compressed, is_compressed)?;
        parse_keys(&keys_data, &names, nodes.len())?
    } else {
        vec![None; nodes.len()]
    };

    let has_keys_section = version >= LSF_VER_BG3_NODE_KEYS && keys_uncompressed > 0;

    Ok(LsfDocument {
        engine_version,
        names,
        nodes,
        attributes,
        values,
        node_keys,
        has_keys_section,
        metadata_format,
    })
}

/// Detect if extended format (16-byte) or V2 format (12-byte) based on data size
fn detect_extended_format(data_size: usize, version_hint: bool) -> bool {
    if data_size % 16 == 0 && data_size % 12 != 0 {
        true  // Only divisible by 16
    } else if data_size % 12 == 0 && data_size % 16 != 0 {
        false // Only divisible by 12
    } else {
        // Divisible by both (or neither), fall back to hint
        version_hint
    }
}

fn read_section<R: Read>(
    reader: &mut R,
    uncompressed_size: usize,
    compressed_size: usize,
    is_compressed: bool,
) -> Result<Vec<u8>> {
    if uncompressed_size == 0 {
        return Ok(Vec::new());
    }

    let read_size = if is_compressed && compressed_size > 0 {
        compressed_size
    } else {
        uncompressed_size
    };

    let mut buffer = vec![0u8; read_size];
    reader.read_exact(&mut buffer)?;

    if is_compressed && compressed_size > 0 {
        // Try LZ4 frame format first
        let mut decoder = lz4_flex::frame::FrameDecoder::new(Cursor::new(&buffer));
        let mut decompressed = Vec::new();
        if decoder.read_to_end(&mut decompressed).is_ok() {
            return Ok(decompressed);
        }

        // Fall back to LZ4 block decompression
        lz4_flex::block::decompress(&buffer, uncompressed_size)
            .map_err(|e| Error::DecompressionError(format!("LZ4: {e}")))
    } else {
        Ok(buffer)
    }
}

fn read_names<R: Read>(
    reader: &mut R,
    uncompressed_size: usize,
    compressed_size: usize,
    is_compressed: bool,
) -> Result<Vec<Vec<String>>> {
    let data = read_section(reader, uncompressed_size, compressed_size, is_compressed)?;
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut cursor = Cursor::new(data);
    let num_hash_entries = cursor.read_u32::<LittleEndian>()? as usize;

    let mut names = Vec::with_capacity(num_hash_entries);
    for _ in 0..num_hash_entries {
        let num_strings = cursor.read_u16::<LittleEndian>()? as usize;
        let mut string_list = Vec::with_capacity(num_strings);

        for _ in 0..num_strings {
            let string_len = cursor.read_u16::<LittleEndian>()? as usize;
            let mut string_bytes = vec![0u8; string_len];
            cursor.read_exact(&mut string_bytes)?;
            string_list.push(String::from_utf8_lossy(&string_bytes).into_owned());
        }
        names.push(string_list);
    }
    Ok(names)
}

fn read_nodes<R: Read>(
    reader: &mut R,
    uncompressed_size: usize,
    compressed_size: usize,
    is_compressed: bool,
    extended_format: bool,
) -> Result<Vec<LsfNode>> {
    let data = read_section(reader, uncompressed_size, compressed_size, is_compressed)?;
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut cursor = Cursor::new(&data);

    // Node size: 16 bytes for extended (v3+), 12 bytes for v2
    let node_size = if extended_format { 16 } else { 12 };
    let node_count = data.len() / node_size;
    let mut nodes = Vec::with_capacity(node_count);

    for _ in 0..node_count {
        // NameHashTableIndex is a u32 packed as: upper 16 bits = hash index, lower 16 bits = offset
        let name_hash_table_index = cursor.read_u32::<LittleEndian>()?;
        let name_index_outer = (name_hash_table_index >> 16) as usize;
        let name_index_inner = (name_hash_table_index & 0xFFFF) as usize;

        if extended_format {
            // V3 format: NameIndex, ParentIndex, NextSiblingIndex, FirstAttributeIndex
            let parent_index = cursor.read_i32::<LittleEndian>()?;
            let _next_sibling_index = cursor.read_i32::<LittleEndian>()?;
            let first_attribute_index = cursor.read_i32::<LittleEndian>()?;

            nodes.push(LsfNode {
                name_index_outer,
                name_index_inner,
                parent_index,
                first_attribute_index,
            });
        } else {
            // V2 format: NameIndex, FirstAttributeIndex, ParentIndex
            let first_attribute_index = cursor.read_i32::<LittleEndian>()?;
            let parent_index = cursor.read_i32::<LittleEndian>()?;

            nodes.push(LsfNode {
                name_index_outer,
                name_index_inner,
                parent_index,
                first_attribute_index,
            });
        }
    }

    Ok(nodes)
}

fn read_attributes<R: Read>(
    reader: &mut R,
    uncompressed_size: usize,
    compressed_size: usize,
    is_compressed: bool,
    extended_format: bool,
    _nodes: &[LsfNode],
) -> Result<Vec<LsfAttribute>> {
    let data = read_section(reader, uncompressed_size, compressed_size, is_compressed)?;
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut cursor = Cursor::new(&data);

    // Attribute size: 16 bytes for extended (v3+), 12 bytes for v2
    let attr_size = if extended_format { 16 } else { 12 };
    let attr_count = data.len() / attr_size;
    let mut attributes = Vec::with_capacity(attr_count);

    // Need to calculate offsets progressively for V2
    let mut current_data_offset: usize = 0;

    for _ in 0..attr_count {
        // NameHashTableIndex packed same as nodes
        let name_hash_table_index = cursor.read_u32::<LittleEndian>()?;
        let name_index_outer = (name_hash_table_index >> 16) as usize;
        let name_index_inner = (name_hash_table_index & 0xFFFF) as usize;

        // TypeAndLength: lower 6 bits = type, upper 26 bits = length
        let type_and_length = cursor.read_u32::<LittleEndian>()?;
        // Type ID is in lower 6 bits: type_and_length & 0x3F (used for validation)
        let length = (type_and_length >> 6) as usize;

        if extended_format {
            // V3 format: has NextAttributeIndex and explicit Offset
            let next_index = cursor.read_i32::<LittleEndian>()?;
            let offset = cursor.read_u32::<LittleEndian>()? as usize;

            attributes.push(LsfAttribute {
                name_index_outer,
                name_index_inner,
                type_info: type_and_length,
                next_index,
                offset,
            });
        } else {
            // V2 format: has NodeIndex instead of NextAttributeIndex, no explicit Offset
            let node_index = cursor.read_i32::<LittleEndian>()?;

            // Offset is calculated from cumulative lengths
            let offset = current_data_offset;
            current_data_offset += length;

            // Store node_index temporarily in next_index field (will be fixed below)
            // We use negative offset to distinguish: -(node_index + 1) so node 0 becomes -1
            attributes.push(LsfAttribute {
                name_index_outer,
                name_index_inner,
                type_info: type_and_length,
                next_index: -(node_index + 1), // Temporary: stores negated node_index
                offset,
            });
        }
    }

    // For V2, build attribute chains based on node ownership
    if !extended_format {
        // Group attributes by their owning node and chain them
        // The next_index field currently holds -(node_index + 1)

        // First, collect attribute indices for each node
        let mut node_attrs: std::collections::HashMap<i32, Vec<usize>> = std::collections::HashMap::new();
        for (attr_idx, attr) in attributes.iter().enumerate() {
            // Decode node_index from temporary encoding
            let node_index = -(attr.next_index + 1);
            node_attrs.entry(node_index).or_default().push(attr_idx);
        }

        // Now chain attributes within each node
        for attr_indices in node_attrs.values() {
            for i in 0..attr_indices.len() {
                let attr_idx = attr_indices[i];
                if i + 1 < attr_indices.len() {
                    // Point to next attribute in chain
                    attributes[attr_idx].next_index = attr_indices[i + 1] as i32;
                } else {
                    // Last attribute in chain
                    attributes[attr_idx].next_index = -1;
                }
            }
        }
    }

    Ok(attributes)
}

fn parse_keys(
    data: &[u8],
    names: &[Vec<String>],
    node_count: usize,
) -> Result<Vec<Option<String>>> {
    if data.is_empty() {
        return Ok(vec![None; node_count]);
    }

    let mut cursor = Cursor::new(data);
    let mut keys = vec![None; node_count];

    // Each key entry is 8 bytes: u32 node_index, u32 name_hash_table_index
    while cursor.position() < cursor.get_ref().len() as u64 {
        let node_index = cursor.read_u32::<LittleEndian>()? as usize;
        let name_hash_table_index = cursor.read_u32::<LittleEndian>()?;
        let name_index_outer = (name_hash_table_index >> 16) as usize;
        let name_index_inner = (name_hash_table_index & 0xFFFF) as usize;

        // Handle sentinel values
        if name_index_outer == 0xFFFF || name_index_inner == 0xFFFF {
            continue;
        }

        if let Some(name_list) = names.get(name_index_outer)
            && let Some(key_name) = name_list.get(name_index_inner)
                && node_index < keys.len() {
                    keys[node_index] = Some(key_name.clone());
                }
    }

    Ok(keys)
}
