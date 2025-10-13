//! LSF (Larian Story Format) binary format parser

use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use lz4_flex::frame::{FrameDecoder, FrameEncoder};

#[derive(Debug)]
pub struct LsfDocument {
    pub engine_version: u64,
    pub names: Vec<Vec<String>>,
    pub nodes: Vec<LsfNode>,
    pub attributes: Vec<LsfAttribute>,
    pub values: Vec<u8>,
    pub node_keys: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct LsfNode {
    pub name_index_outer: usize,
    pub name_index_inner: usize,
    pub parent_index: i32,
    pub first_attribute_index: i32,
}

#[derive(Debug, Clone)]
pub struct LsfAttribute {
    pub name_index_outer: usize,
    pub name_index_inner: usize,
    pub type_info: u32,
    pub next_index: i32,
    pub offset: usize,
}

impl LsfDocument {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Self::from_bytes(&buffer)
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
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
        let names = Self::read_names(&mut cursor, sections[0], is_compressed)?;
        
        println!("\n=== Reading nodes ===");
        let nodes = Self::read_nodes(&mut cursor, sections[2], is_compressed)?;
        
        println!("\n=== Reading attributes ===");
        let attributes = Self::read_attributes(&mut cursor, sections[3], is_compressed)?;
        
        println!("\n=== Reading values ===");
        let values = Self::read_section(&mut cursor, sections[4], is_compressed)?;
        
        // Read keys section
        println!("\n=== Reading node keys ===");
        let node_keys = Self::read_keys(&mut cursor, sections[1], is_compressed, &names, nodes.len())?;
        
        Ok(LsfDocument {
            engine_version,
            names,
            nodes,
            attributes,
            values,
            node_keys,
        })
    }
    
    pub fn get_name(&self, outer: usize, inner: usize) -> Result<&str> {
        self.names
            .get(outer)
            .and_then(|list| list.get(inner))
            .map(|s| s.as_str())
            .ok_or_else(|| Error::InvalidStringIndex(inner as i32))
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
        let data = Self::read_section(reader, sizes, is_compressed)?;
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
        let data = Self::read_section(reader, sizes, is_compressed)?;
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
        let data = Self::read_section(reader, sizes, is_compressed)?;
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
        let data = Self::read_section(reader, sizes, is_compressed)?;
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
    
    fn parse_keys(data: &[u8], names: &[Vec<String>], node_count: usize) -> Result<Vec<Option<String>>> {
        if data.is_empty() {
            return Ok(vec![None; node_count]);
        }
        
        let mut cursor = Cursor::new(data);
        let mut keys = vec![None; node_count];
        
        while cursor.position() < cursor.get_ref().len() as u64 {
            let node_index = cursor.read_u32::<LittleEndian>()? as usize;
            // Read as packed u32
            let packed_name = cursor.read_u32::<LittleEndian>()?;
            let name_index_outer = ((packed_name >> 16) & 0xFFFF) as usize;
            let name_index_inner = (packed_name & 0xFFFF) as usize;
            
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

    /// Serialize names section
    fn write_names(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        
        // Write number of name lists
        buffer.write_u32::<LittleEndian>(self.names.len() as u32)?;
        
        for name_list in &self.names {
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
    fn write_keys(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        
        for (node_idx, key_opt) in self.node_keys.iter().enumerate() {
            if let Some(key) = key_opt {
                if let Some((outer, inner)) = self.find_name_indices(key) {
                    buffer.write_u32::<LittleEndian>(node_idx as u32)?;
                    // Pack as single u32: outer in high 16 bits, inner in low 16 bits
                    let packed_name = ((outer as u32) << 16) | (inner as u32);
                    buffer.write_u32::<LittleEndian>(packed_name)?;
                }
            }
        }
        
        Ok(buffer)
    }
    
    /// Find name indices in the names table
    fn find_name_indices(&self, name: &str) -> Option<(usize, usize)> {
        for (outer, name_list) in self.names.iter().enumerate() {
            for (inner, list_name) in name_list.iter().enumerate() {
                if list_name == name {
                    return Some((outer, inner));
                }
            }
        }
        None
    }
    
    /// Serialize nodes section
    fn write_nodes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        
        for node in &self.nodes {
            buffer.write_u16::<LittleEndian>(node.name_index_inner as u16)?;
            buffer.write_u16::<LittleEndian>(node.name_index_outer as u16)?;
            buffer.write_i32::<LittleEndian>(node.parent_index)?;
            buffer.write_i32::<LittleEndian>(-1)?; // next_sibling_index
            buffer.write_i32::<LittleEndian>(node.first_attribute_index)?;
        }
        
        Ok(buffer)
    }
    
    /// Serialize attributes section
    fn write_attributes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        
        for attr in &self.attributes {
            buffer.write_u16::<LittleEndian>(attr.name_index_inner as u16)?;
            buffer.write_u16::<LittleEndian>(attr.name_index_outer as u16)?;
            buffer.write_u32::<LittleEndian>(attr.type_info)?;
            buffer.write_i32::<LittleEndian>(attr.next_index)?;
            buffer.write_u32::<LittleEndian>(attr.offset as u32)?;
        }
        
        Ok(buffer)
    }
    
    /// Write LSF document to binary format
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        
        // Write header
        output.extend_from_slice(b"LSOF");
        output.write_u32::<LittleEndian>(6)?;
        output.write_u64::<LittleEndian>(self.engine_version)?;
        
        // Prepare sections
        let names_data = self.write_names()?;
        let keys_data = self.write_keys()?;
        let nodes_data = self.write_nodes()?;
        let attributes_data = self.write_attributes()?;
        let values_data = &self.values;
        
        // Compress sections using LZ4 Frame format
        let mut encoder = FrameEncoder::new(Vec::new());
        encoder.write_all(&names_data)?;
        let names_compressed = encoder.finish()?;
        
        let keys_compressed = if keys_data.is_empty() {
            Vec::new()
        } else {
            let mut encoder = FrameEncoder::new(Vec::new());
            encoder.write_all(&keys_data)?;
            encoder.finish()?
        };
        
        let mut encoder = FrameEncoder::new(Vec::new());
        encoder.write_all(&nodes_data)?;
        let nodes_compressed = encoder.finish()?;
        
        let mut encoder = FrameEncoder::new(Vec::new());
        encoder.write_all(&attributes_data)?;
        let attributes_compressed = encoder.finish()?;
        
        let mut encoder = FrameEncoder::new(Vec::new());
        encoder.write_all(values_data)?;
        let values_compressed = encoder.finish()?;
       
        // Write section sizes - compressed size, then uncompressed size
        // Strings section
        output.write_u32::<LittleEndian>(names_compressed.len() as u32)?;
        output.write_u32::<LittleEndian>(names_data.len() as u32)?;
        
        // Keys section
        output.write_u32::<LittleEndian>(keys_compressed.len() as u32)?;
        output.write_u32::<LittleEndian>(keys_data.len() as u32)?;
        
        // Nodes section
        output.write_u32::<LittleEndian>(nodes_compressed.len() as u32)?;
        output.write_u32::<LittleEndian>(nodes_data.len() as u32)?;
        
        // Attributes section
        output.write_u32::<LittleEndian>(attributes_compressed.len() as u32)?;
        output.write_u32::<LittleEndian>(attributes_data.len() as u32)?;
        
        // Values section
        output.write_u32::<LittleEndian>(values_compressed.len() as u32)?;
        output.write_u32::<LittleEndian>(values_data.len() as u32)?;
        
        // Compression flags (0x01 = LZ4)
        output.write_u32::<LittleEndian>(0x01)?;
        
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
}