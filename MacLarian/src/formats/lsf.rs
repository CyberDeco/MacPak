//! LSF (Larian Story Format) binary format parser

use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

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
        
        // Read header
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != b"LSOF" {
            return Err(Error::InvalidLsfMagic(magic));
        }
        
        let version = cursor.read_u32::<LittleEndian>()?;
        if version < 2 || version > 7 {
            return Err(Error::UnsupportedLsfVersion(version));
        }
        
        let engine_version = cursor.read_u64::<LittleEndian>()?;
        
        // Read section sizes
        let sections = [
            (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // strings
            (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // keys
            (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // nodes
            (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // attributes
            (cursor.read_u32::<LittleEndian>()? as usize, cursor.read_u32::<LittleEndian>()? as usize), // values
        ];
        
        let compression_flags = cursor.read_u32::<LittleEndian>()?;
        let _extended_format = cursor.read_u32::<LittleEndian>()?;
        let is_compressed = compression_flags & 0x0F != 0;
        
        // Read and decompress sections
        let names = Self::read_names(&mut cursor, sections[0], is_compressed)?;
        let nodes = Self::read_nodes(&mut cursor, sections[2], is_compressed)?;
        let attributes = Self::read_attributes(&mut cursor, sections[3], is_compressed)?;
        let values = Self::read_section(&mut cursor, sections[4], is_compressed)?;
        
        // Read keys section (maps node indices to their key attribute names)
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
            lz4_flex::decompress(&buffer, uncompressed_size)
                .map_err(|e| Error::DecompressionError(format!("LZ4: {}", e)))
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
}