//! LSF file reading and parsing

use super::document::{LsfDocument, LsfNode, LsfAttribute};
use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use lz4_flex::frame::FrameDecoder;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

/// Read an LSF file from disk
pub fn read_lsf<P: AsRef<Path>>(path: P) -> Result<LsfDocument> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    parse_lsf_bytes(&buffer)
}

/// Parse LSF data from bytes
pub fn parse_lsf_bytes(data: &[u8]) -> Result<LsfDocument> {
    let mut cursor = Cursor::new(data);
    
    println!("Total file size: {} bytes", data.len());
    
    // Read header
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    println!("Magic: {:?}", std::str::from_utf8(&magic).unwrap_or("???"));
    if &magic != b"LSOF" {
        return Err(Error::InvalidLsfMagic(magic));
    }
    
    let version = cursor.read_u32::<LittleEndian>()?;
    println!("Version: {}", version);
    if version < 2 || version > 7 {
        return Err(Error::UnsupportedLsfVersion(version));
    }
    
    let engine_version = cursor.read_u64::<LittleEndian>()?;
    println!("Engine version: {:#x}", engine_version);
    
    // Read section sizes
    let sections = [
        (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // strings
        (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // keys
        (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // nodes
        (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // attributes
        (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // values
    ];
    
    println!("Strings: {} compressed, {} uncompressed", sections[0].0, sections[0].1);
    println!("Keys: {} compressed, {} uncompressed", sections[1].0, sections[1].1);
    println!("Nodes: {} compressed, {} uncompressed", sections[2].0, sections[2].1);
    println!("Attributes: {} compressed, {} uncompressed", sections[3].0, sections[3].1);
    println!("Values: {} compressed, {} uncompressed", sections[4].0, sections[4].1);
    
    let compression_flags = cursor.read_u32::<LittleEndian>()?;
    let _extended_format = cursor.read_u32::<LittleEndian>()?;
    let is_compressed = compression_flags & 0x0F != 0;
    
    println!("Compression flags: {:#x}, is_compressed: {}", compression_flags, is_compressed);
    println!("Header ends at position: {}", cursor.position());
    
    // Read and decompress sections
    println!("\n=== Reading strings ===");
    let names = read_names(&mut cursor, sections[0], is_compressed)?;
    
    println!("\n=== Reading nodes ===");
    let nodes = read_nodes(&mut cursor, sections[2], is_compressed)?;
    
    println!("\n=== Reading attributes ===");
    let attributes = read_attributes(&mut cursor, sections[3], is_compressed)?;
    
    println!("\n=== Reading values ===");
    let values = read_section(&mut cursor, sections[4], is_compressed)?;

    // Read keys section
    println!("\n=== Reading node keys ===");
    let node_keys = read_keys(&mut cursor, sections[1], is_compressed, &names, nodes.len())?;
    let has_keys_section = !node_keys.iter().all(|k| k.is_none());
    
    Ok(LsfDocument {
        engine_version,
        names,
        nodes,
        attributes,
        values,
        node_keys,
        has_keys_section,
    })
}

fn read_section<R: Read>(
    reader: &mut R,
    (compressed_size, uncompressed_size): (usize, usize),
    is_compressed: bool,
) -> Result<Vec<u8>> {
    let size = if !is_compressed && compressed_size == 0 {
        uncompressed_size
    } else {
        compressed_size
    };
    
    let mut buffer = vec![0u8; size];
    reader.read_exact(&mut buffer)?;
    
    if is_compressed {
        // Use frame format for compression
        let mut decoder = FrameDecoder::new(Cursor::new(buffer));
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| Error::DecompressionError(format!("LZ4: {}", e)))?;
        Ok(decompressed)
    } else {
        Ok(buffer)
    }
}

fn read_names<R: Read>(
    reader: &mut R,
    sizes: (usize, usize),
    is_compressed: bool,
) -> Result<Vec<Vec<String>>> {
    let data = read_section(reader, sizes, is_compressed)?;
    let mut cursor = Cursor::new(data);
    let num_entries = cursor.read_u32::<LittleEndian>()? as usize;
    
    let mut names = Vec::with_capacity(num_entries);
    for _ in 0..num_entries {
        let num_names = cursor.read_u16::<LittleEndian>()? as usize;
        let mut name_list = Vec::with_capacity(num_names);
        
        for _ in 0..num_names {
            let name_len = cursor.read_u16::<LittleEndian>()? as usize;
            let mut name_bytes = vec![0u8; name_len];
            cursor.read_exact(&mut name_bytes)?;
            name_list.push(String::from_utf8_lossy(&name_bytes).into_owned());
        }
        names.push(name_list);
    }
    Ok(names)
}

fn read_nodes<R: Read>(
    reader: &mut R,
    sizes: (usize, usize),
    is_compressed: bool,
) -> Result<Vec<LsfNode>> {
    let data = read_section(reader, sizes, is_compressed)?;
    let mut cursor = Cursor::new(data);
    let mut nodes = Vec::new();
    
    while cursor.position() < cursor.get_ref().len() as u64 {
        let name_index_inner = cursor.read_u16::<LittleEndian>()? as usize;
        let name_index_outer = cursor.read_u16::<LittleEndian>()? as usize;
        let parent_index = cursor.read_i32::<LittleEndian>()?;
        let _next_sibling_index = cursor.read_i32::<LittleEndian>()?;
        let first_attribute_index = cursor.read_i32::<LittleEndian>()?;
        
        nodes.push(LsfNode {
            name_index_outer,
            name_index_inner,
            parent_index,
            first_attribute_index,
        });
    }
    Ok(nodes)
}

fn read_attributes<R: Read>(
    reader: &mut R,
    sizes: (usize, usize),
    is_compressed: bool,
) -> Result<Vec<LsfAttribute>> {
    let data = read_section(reader, sizes, is_compressed)?;
    let mut cursor = Cursor::new(data);
    let mut attributes = Vec::new();
    
    while cursor.position() < cursor.get_ref().len() as u64 {
        let name_index_inner = cursor.read_u16::<LittleEndian>()? as usize;
        let name_index_outer = cursor.read_u16::<LittleEndian>()? as usize;
        let type_info = cursor.read_u32::<LittleEndian>()?;
        let next_index = cursor.read_i32::<LittleEndian>()?;
        let offset = cursor.read_u32::<LittleEndian>()? as usize;
        
        attributes.push(LsfAttribute {
            name_index_outer,
            name_index_inner,
            type_info,
            next_index,
            offset,
        });
    }
    Ok(attributes)
}

fn read_keys<R: Read>(
    reader: &mut R,
    sizes: (usize, usize),
    is_compressed: bool,
    names: &[Vec<String>],
    node_count: usize,
) -> Result<Vec<Option<String>>> {
    let data = read_section(reader, sizes, is_compressed)?;
    if data.is_empty() {
        return Ok(vec![None; node_count]);
    }
    
    let mut cursor = Cursor::new(data);
    let mut keys = vec![None; node_count];
    
    // Each key entry is 8 bytes: u32 node_index, u16 name_inner, u16 name_outer
    while cursor.position() < cursor.get_ref().len() as u64 {
        let node_index = cursor.read_u32::<LittleEndian>()? as usize;
        let name_index_inner = cursor.read_u16::<LittleEndian>()? as usize;
        let name_index_outer = cursor.read_u16::<LittleEndian>()? as usize;
        
        if let Some(name_list) = names.get(name_index_outer) {
            if let Some(key_name) = name_list.get(name_index_inner) {
                if node_index < keys.len() {
                    keys[node_index] = Some(key_name.clone());
                }
            }
        }
    }
    
    Ok(keys)
}