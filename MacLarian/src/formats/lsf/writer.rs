//! LSF file writing and serialization
//!
//! # Compression Format Convention
//!
//! LSF files use LZ4 compression with two different formats per section:
//! - **Strings section**: LZ4 Block format (raw compressed data, no frame header)
//! - **All other sections** (nodes, attributes, values, keys): LZ4 Frame format
//!   (with magic bytes 0x04 0x22 0x4D 0x18)
//!
//! This matches `LSLib`'s behavior where `allowChunked=false` uses Block format
//! and `allowChunked=true` uses Frame format. The compression flags in the header
//! (0x22 = LZ4 + `DefaultCompress`) indicate the method but not per-section format.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0

// Binary format writing requires many intentional casts between integer types
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use super::document::LsfDocument;
use crate::error::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;
use std::path::Path;

/// Node/attribute format for LSF files
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LsfFormat {
    /// V2 format: 12-byte nodes/attributes (more compact, default)
    /// Used when `MetadataFormat` = None (0)
    #[default]
    V2,
    /// V3 format: 16-byte nodes/attributes (extended with sibling/offset info)
    /// Used when `MetadataFormat` = `KeysAndAdjacency` (1)
    V3,
}

/// Compress data using LZ4 block format (used for strings section)
fn compress_lz4_block(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    lz4_flex::block::compress(data)
}

/// Compress data using LZ4 frame format (used for nodes, attributes, values, keys)
fn compress_lz4_frame(data: &[u8]) -> std::io::Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let mut encoder = lz4_flex::frame::FrameEncoder::new(Vec::new());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

/// Write an LSF document to disk (LZ4 compressed, V2 format)
///
/// # Errors
/// Returns an error if serialization or file writing fails.
pub fn write_lsf<P: AsRef<Path>>(doc: &LsfDocument, path: P) -> Result<()> {
    write_lsf_with_format(doc, path, LsfFormat::V2)
}

/// Write an LSF document to disk with specified format
///
/// # Errors
/// Returns an error if serialization or file writing fails.
pub fn write_lsf_with_format<P: AsRef<Path>>(
    doc: &LsfDocument,
    path: P,
    format: LsfFormat,
) -> Result<()> {
    let bytes = serialize_lsf_with_format(doc, format)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Serialize LSF document to bytes (LZ4 compressed, V2 format)
///
/// # Errors
/// Returns an error if serialization fails.
pub fn serialize_lsf(doc: &LsfDocument) -> Result<Vec<u8>> {
    serialize_lsf_with_format(doc, LsfFormat::V2)
}

/// Serialize LSF document to bytes with specified format
///
/// # Errors
/// Returns an error if serialization fails.
pub fn serialize_lsf_with_format(doc: &LsfDocument, format: LsfFormat) -> Result<Vec<u8>> {
    let mut output = Vec::new();

    // Write header
    output.extend_from_slice(b"LSOF");
    output.write_u32::<LittleEndian>(6)?;
    output.write_u64::<LittleEndian>(doc.engine_version)?;

    // Prepare sections
    let names_data = write_names(doc)?;
    let keys_data = write_keys(doc)?;
    let nodes_data = write_nodes(doc, format)?;
    let attributes_data = write_attributes(doc, format)?;
    let values_data = &doc.values;

    // Compress sections per LSLib convention:
    // - Strings: LZ4 Block (allowChunked=false) - raw compressed data
    // - All others: LZ4 Frame (allowChunked=true) - with frame header magic
    let names_compressed = compress_lz4_block(&names_data);
    let keys_compressed = compress_lz4_frame(&keys_data)?;
    let nodes_compressed = compress_lz4_frame(&nodes_data)?;
    let attributes_compressed = compress_lz4_frame(&attributes_data)?;
    let values_compressed = compress_lz4_frame(values_data)?;

    // Write section sizes - uncompressed size first, then compressed size
    // Header order for v6+: Strings, Keys, Nodes, Attributes, Values

    // Strings section
    output.write_u32::<LittleEndian>(names_data.len() as u32)?;
    output.write_u32::<LittleEndian>(names_compressed.len() as u32)?;

    // Keys section
    output.write_u32::<LittleEndian>(keys_data.len() as u32)?;
    output.write_u32::<LittleEndian>(keys_compressed.len() as u32)?;

    // Nodes section
    output.write_u32::<LittleEndian>(nodes_data.len() as u32)?;
    output.write_u32::<LittleEndian>(nodes_compressed.len() as u32)?;

    // Attributes section
    output.write_u32::<LittleEndian>(attributes_data.len() as u32)?;
    output.write_u32::<LittleEndian>(attributes_compressed.len() as u32)?;

    // Values section
    output.write_u32::<LittleEndian>(values_data.len() as u32)?;
    output.write_u32::<LittleEndian>(values_compressed.len() as u32)?;

    // Compression flags: 0x22 = MethodLZ4 (0x02) | DefaultCompress (0x20)
    // Note: This indicates compression method, not per-section format (Block vs Frame)
    output.write_u32::<LittleEndian>(0x22)?;

    // Metadata format determines node/attribute format:
    // - 0 (None) = V2 format (12-byte nodes/attrs)
    // - 1 (KeysAndAdjacency) = V3 format (16-byte nodes/attrs with sibling/offset)
    // - 2 (None2) = V2 format (same as 0, different lslib_meta string)
    let metadata_format = match format {
        LsfFormat::V3 => 1u32, // KeysAndAdjacency
        LsfFormat::V2 => 0u32, // None
    };
    output.write_u32::<LittleEndian>(metadata_format)?;

    // Write section data
    // File order: Strings, Nodes, Attributes, Values, Keys
    output.extend_from_slice(&names_compressed);
    output.extend_from_slice(&nodes_compressed);
    output.extend_from_slice(&attributes_compressed);
    output.extend_from_slice(&values_compressed);
    output.extend_from_slice(&keys_compressed);

    Ok(output)
}

/// Serialize names section
fn write_names(doc: &LsfDocument) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Write number of name lists
    buffer.write_u32::<LittleEndian>(doc.names.len() as u32)?;

    for name_list in &doc.names {
        // Write number of names in this list
        buffer.write_u16::<LittleEndian>(name_list.len() as u16)?;

        for name in name_list {
            // Write name length and bytes
            buffer.write_u16::<LittleEndian>(name.len() as u16)?;
            buffer.extend_from_slice(name.as_bytes());
        }
    }

    Ok(buffer)
}

/// Serialize keys section
fn write_keys(doc: &LsfDocument) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    for (node_idx, key_opt) in doc.node_keys.iter().enumerate() {
        if let Some(key) = key_opt
            && let Some((outer, inner)) = doc.find_name_indices(key)
        {
            buffer.write_u32::<LittleEndian>(node_idx as u32)?;
            // Pack as single u32: outer in high 16 bits, inner in low 16 bits
            let packed_name = ((outer as u32) << 16) | (inner as u32);
            buffer.write_u32::<LittleEndian>(packed_name)?;
        }
    }

    Ok(buffer)
}

/// Serialize nodes section
fn write_nodes(doc: &LsfDocument, format: LsfFormat) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    for node in &doc.nodes {
        // NameHashTableIndex: packed as (outer << 16) | inner
        buffer.write_u16::<LittleEndian>(node.name_index_inner as u16)?;
        buffer.write_u16::<LittleEndian>(node.name_index_outer as u16)?;

        match format {
            LsfFormat::V2 => {
                // V2: NameIndex(4), FirstAttributeIndex(4), ParentIndex(4) = 12 bytes
                buffer.write_i32::<LittleEndian>(node.first_attribute_index)?;
                buffer.write_i32::<LittleEndian>(node.parent_index)?;
            }
            LsfFormat::V3 => {
                // V3: NameIndex(4), ParentIndex(4), NextSiblingIndex(4), FirstAttributeIndex(4) = 16 bytes
                buffer.write_i32::<LittleEndian>(node.parent_index)?;
                buffer.write_i32::<LittleEndian>(-1)?; // next_sibling_index (not tracked)
                buffer.write_i32::<LittleEndian>(node.first_attribute_index)?;
            }
        }
    }

    Ok(buffer)
}

/// Serialize attributes section
fn write_attributes(doc: &LsfDocument, format: LsfFormat) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    match format {
        LsfFormat::V2 => {
            // V2: NameIndex(4), TypeAndLength(4), NodeIndex(4) = 12 bytes
            // Need to figure out which node owns each attribute
            let attr_to_node = build_attr_to_node_map(doc);

            for (attr_idx, attr) in doc.attributes.iter().enumerate() {
                buffer.write_u16::<LittleEndian>(attr.name_index_inner as u16)?;
                buffer.write_u16::<LittleEndian>(attr.name_index_outer as u16)?;
                buffer.write_u32::<LittleEndian>(attr.type_info)?;
                let node_index = attr_to_node.get(&attr_idx).copied().unwrap_or(-1);
                buffer.write_i32::<LittleEndian>(node_index)?;
            }
        }
        LsfFormat::V3 => {
            // V3: NameIndex(4), TypeAndLength(4), NextAttributeIndex(4), Offset(4) = 16 bytes
            for attr in &doc.attributes {
                buffer.write_u16::<LittleEndian>(attr.name_index_inner as u16)?;
                buffer.write_u16::<LittleEndian>(attr.name_index_outer as u16)?;
                buffer.write_u32::<LittleEndian>(attr.type_info)?;
                buffer.write_i32::<LittleEndian>(attr.next_index)?;
                buffer.write_u32::<LittleEndian>(attr.offset as u32)?;
            }
        }
    }

    Ok(buffer)
}

/// Build a map from attribute index to owning node index (for V2 format)
fn build_attr_to_node_map(doc: &LsfDocument) -> std::collections::HashMap<usize, i32> {
    let mut map = std::collections::HashMap::new();

    for (node_idx, node) in doc.nodes.iter().enumerate() {
        let mut attr_idx = node.first_attribute_index;
        while attr_idx >= 0 {
            map.insert(attr_idx as usize, node_idx as i32);
            if let Some(attr) = doc.attributes.get(attr_idx as usize) {
                attr_idx = attr.next_index;
            } else {
                break;
            }
        }
    }

    map
}
