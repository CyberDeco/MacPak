//! LSF file writing and serialization

use super::document::LsfDocument;
use crate::error::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;
use std::path::Path;

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

/// Write an LSF document to disk
pub fn write_lsf<P: AsRef<Path>>(doc: &LsfDocument, path: P) -> Result<()> {
    let bytes = serialize_lsf(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Serialize LSF document to bytes
pub fn serialize_lsf(doc: &LsfDocument) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    
    // Write header
    output.extend_from_slice(b"LSOF");
    output.write_u32::<LittleEndian>(6)?;
    output.write_u64::<LittleEndian>(doc.engine_version)?;
    
    // Prepare sections
    let names_data = write_names(doc)?;
    let keys_data = write_keys(doc)?;
    let nodes_data = write_nodes(doc)?;
    let attributes_data = write_attributes(doc)?;
    let values_data = &doc.values;

    // Compress sections - LSLib uses block for strings, frame for everything else
    let names_compressed = compress_lz4_block(&names_data);
    let keys_compressed = compress_lz4_frame(&keys_data)?;
    let nodes_compressed = compress_lz4_frame(&nodes_data)?;
    let attributes_compressed = compress_lz4_frame(&attributes_data)?;
    let values_compressed = compress_lz4_frame(values_data)?;

    // Write section sizes - uncompressed size first, then compressed size (per LSLib format)
    // Header order must match reader expectations for v6+:
    // Strings, Keys, Nodes, Attributes, Values

    // Strings section
    output.write_u32::<LittleEndian>(names_data.len() as u32)?;
    output.write_u32::<LittleEndian>(names_compressed.len() as u32)?;

    // Keys section (v6+ only, but we always write v6)
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

    // Compression flags: 0x02 = LZ4, 0x20 = Default level
    output.write_u32::<LittleEndian>(0x22)?;

    // Extended format flags (0 for BG3)
    output.write_u32::<LittleEndian>(0)?;

    // Write compressed section data
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
        if let Some(key) = key_opt {
            if let Some((outer, inner)) = doc.find_name_indices(key) {
                buffer.write_u32::<LittleEndian>(node_idx as u32)?;
                // Pack as single u32: outer in high 16 bits, inner in low 16 bits
                let packed_name = ((outer as u32) << 16) | (inner as u32);
                buffer.write_u32::<LittleEndian>(packed_name)?;
            }
        }
    }
    
    Ok(buffer)
}

/// Serialize nodes section
fn write_nodes(doc: &LsfDocument) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    
    for node in &doc.nodes {
        buffer.write_u16::<LittleEndian>(node.name_index_inner as u16)?;
        buffer.write_u16::<LittleEndian>(node.name_index_outer as u16)?;
        buffer.write_i32::<LittleEndian>(node.parent_index)?;
        buffer.write_i32::<LittleEndian>(-1)?; // next_sibling_index
        buffer.write_i32::<LittleEndian>(node.first_attribute_index)?;
    }
    
    Ok(buffer)
}

/// Serialize attributes section
fn write_attributes(doc: &LsfDocument) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    
    for attr in &doc.attributes {
        buffer.write_u16::<LittleEndian>(attr.name_index_inner as u16)?;
        buffer.write_u16::<LittleEndian>(attr.name_index_outer as u16)?;
        buffer.write_u32::<LittleEndian>(attr.type_info)?;
        buffer.write_i32::<LittleEndian>(attr.next_index)?;
        buffer.write_u32::<LittleEndian>(attr.offset as u32)?;
    }
    
    Ok(buffer)
}