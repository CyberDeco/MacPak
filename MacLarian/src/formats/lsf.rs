//! LSF (Larian Story Format) binary format handler

use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

/// Main conversion function
pub fn convert_lsf_to_lsx<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    tracing::info!("Converting LSF→LSX: {:?} → {:?}", source.as_ref(), dest.as_ref());
    let lsf_doc = LsfDocument::from_file(&source)?;
    let lsx_xml = lsf_doc.to_lsx()?;
    std::fs::write(dest, lsx_xml)?;
    tracing::info!("Conversion complete");
    Ok(())
}

#[derive(Debug)]
pub struct LsfDocument {
    engine_version: u64,
    names: Vec<Vec<String>>,
    nodes: Vec<LsfNode>,
    attributes: Vec<LsfAttribute>,
    values: Vec<u8>,
    node_keys: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct LsfNode {
    name_index_outer: usize,
    name_index_inner: usize,
    parent_index: i32,
    first_attribute_index: i32,
}

#[derive(Debug, Clone)]
pub struct LsfAttribute {
    name_index_outer: usize,
    name_index_inner: usize,
    type_info: u32,
    next_index: i32,
    offset: usize,
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
            let next_sibling_index = cursor.read_i32::<LittleEndian>()?;
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
            // No keys section, return empty vec
            return Ok(vec![None; node_count]);
        }
        
        let mut cursor = Cursor::new(data);
        let mut keys = vec![None; node_count];
        
        // Each key entry is 8 bytes: u32 node_index, u16 name_outer, u16 name_inner
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
    
    fn get_name(&self, outer: usize, inner: usize) -> Result<&str> {
        self.names
            .get(outer)
            .and_then(|list| list.get(inner))
            .map(|s| s.as_str())
            .ok_or_else(|| Error::InvalidStringIndex(inner as i32))
    }
    
    pub fn to_lsx(&self) -> Result<String> {
        let mut output = Vec::new();
        
        // Write UTF-8 BOM
        output.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        
        let mut writer = Writer::new_with_indent(&mut output, b'\t', 1);
        
        // XML declaration
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;
        
        // <save>
        writer.write_event(Event::Start(BytesStart::new("save")))?;
        
        // <version>
        self.write_version(&mut writer)?;
        
        // <region>
        let region_id = self.nodes.first()
            .filter(|n| n.parent_index == -1)
            .map(|n| self.get_name(n.name_index_outer, n.name_index_inner))
            .transpose()?
            .unwrap_or("root");
        
        let mut region = BytesStart::new("region");
        region.push_attribute(("id", region_id));
        writer.write_event(Event::Start(region.borrow()))?;
        
        // Write root nodes
        for (i, node) in self.nodes.iter().enumerate() {
            if node.parent_index == -1 {
                self.write_node(&mut writer, i)?;
            }
        }
        
        writer.write_event(Event::End(BytesEnd::new("region")))?;
        writer.write_event(Event::End(BytesEnd::new("save")))?;
        
        let xml = String::from_utf8(output)?;
        // Convert to Windows line endings (CRLF) to match LSLib output
        let xml = xml.replace("\n", "\r\n");
        // Fix spacing before self-closing tags
        let xml = xml.replace("/>", " />");
        Ok(xml)
    }
    
    fn write_version<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let major = ((self.engine_version >> 55) & 0x7F) as u32;
        let minor = ((self.engine_version >> 47) & 0xFF) as u32;
        let revision = ((self.engine_version >> 31) & 0xFFFF) as u32;
        let build = (self.engine_version & 0x7FFFFFFF) as u32;
        
        let mut version = BytesStart::new("version");
        version.push_attribute(("major", major.to_string().as_str()));
        version.push_attribute(("minor", minor.to_string().as_str()));
        version.push_attribute(("revision", revision.to_string().as_str()));
        version.push_attribute(("build", build.to_string().as_str()));
        version.push_attribute(("lslib_meta", "v1,bswap_guids,lsf_keys_adjacency"));
        writer.write_event(Event::Empty(version))?;
        Ok(())
    }
    
    fn write_node<W: std::io::Write>(&self, writer: &mut Writer<W>, node_idx: usize) -> Result<()> {
        let node = &self.nodes[node_idx];
        let node_name = self.get_name(node.name_index_outer, node.name_index_inner)?;
        
        let has_attributes = node.first_attribute_index >= 0;
        let children: Vec<_> = self.nodes
            .iter()
            .enumerate()
            .filter(|(_, child)| child.parent_index == node_idx as i32)
            .collect();
        let has_children = !children.is_empty();
        
        // Get key attribute from the keys section
        let key_attr = self.node_keys.get(node_idx).and_then(|k| k.as_deref());
        
        let mut node_start = BytesStart::new("node");
        node_start.push_attribute(("id", node_name));
        
        if let Some(key) = key_attr {
            node_start.push_attribute(("key", key));
        }
        
        if !has_attributes && !has_children {
            writer.write_event(Event::Empty(node_start))?;
            return Ok(());
        }
        
        writer.write_event(Event::Start(node_start.borrow()))?;
        
        if has_attributes {
            let mut attr_idx = node.first_attribute_index as usize;
            loop {
                self.write_attribute(writer, attr_idx)?;
                let attr = &self.attributes[attr_idx];
                if attr.next_index < 0 {
                    break;
                }
                attr_idx = attr.next_index as usize;
            }
        }
        
        if has_children {
            writer.write_event(Event::Start(BytesStart::new("children")))?;
            for (child_idx, _) in children {
                self.write_node(writer, child_idx)?;
            }
            writer.write_event(Event::End(BytesEnd::new("children")))?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("node")))?;
        Ok(())
    }
    
    fn write_attribute<W: std::io::Write>(&self, writer: &mut Writer<W>, attr_idx: usize) -> Result<()> {
        let attr = &self.attributes[attr_idx];
        let attr_name = self.get_name(attr.name_index_outer, attr.name_index_inner)?;
        let type_id = attr.type_info & 0x3F;
        let value_length = (attr.type_info >> 6) as usize;
        
        let type_name = match type_id {
            1 => "uint8",
            2 => "int16",
            3 => "uint16",
            4 => "int32",
            5 => "uint32",
            6 => "float",
            7 => "double",
            19 => "bool",
            20 => "string",
            21 => "LSString",
            22 => "FixedString",
            23 => "LSString",
            24 => "uint64",
            26 => "int64",
            28 => "TranslatedString",
            31 => "guid",
            _ => "Unknown",
        };
        
        let value_str = self.extract_value(attr.offset, value_length, type_id)?;
        
        let mut attr_start = BytesStart::new("attribute");
        attr_start.push_attribute(("id", attr_name));
        attr_start.push_attribute(("type", type_name));
        
        // TranslatedString has special format: handle and version instead of value
        if type_id == 28 {
            // Parse TranslatedString: version (u16) + handle_length (i32) + handle (string)
            if let Ok((handle, version, value)) = self.extract_translated_string(attr.offset, value_length) {
                attr_start.push_attribute(("handle", handle.as_str()));
                if let Some(val) = value {
                    // If there's a value, write it
                    attr_start.push_attribute(("value", val.as_str()));
                } else {
                    // Otherwise write the version
                    attr_start.push_attribute(("version", version.to_string().as_str()));
                }
            }
        } else {
            attr_start.push_attribute(("value", value_str.as_str()));
        }
        
        writer.write_event(Event::Empty(attr_start))?;
        Ok(())
    }
    
    fn extract_translated_string(&self, offset: usize, length: usize) -> Result<(String, u16, Option<String>)> {
        if offset + length > self.values.len() {
            return Ok((String::new(), 0, None));
        }
        
        let bytes = &self.values[offset..offset + length];
        let mut cursor = Cursor::new(bytes);
        
        // Read version (u16)
        let version = cursor.read_u16::<LittleEndian>()?;
        
        // Read handle length (i32)
        let handle_length = cursor.read_i32::<LittleEndian>()? as usize;
        
        if handle_length == 0 {
            return Ok((String::new(), version, None));
        }
        
        // Read handle string (null-terminated)
        let mut handle_bytes = vec![0u8; handle_length.saturating_sub(1)];
        cursor.read_exact(&mut handle_bytes)?;
        let _ = cursor.read_u8()?; // null terminator
        
        let handle = String::from_utf8_lossy(&handle_bytes).into_owned();
        
        // Check if there's a value after the handle
        // If there are more bytes, try to read the value
        let value = if cursor.position() < bytes.len() as u64 {
            // There might be a value string, but for BG3 it's typically empty
            // Just return None for now since the version is what matters
            None
        } else {
            None
        };
        
        Ok((handle, version, value))
    }
    
    fn extract_value(&self, offset: usize, length: usize, type_id: u32) -> Result<String> {
        if offset + length > self.values.len() {
            return Ok(String::new());
        }
        
        let bytes = &self.values[offset..offset + length];
        
        Ok(match type_id {
            20 | 21 | 22 | 23 | 29 | 30 => {
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                String::from_utf8_lossy(&bytes[..end]).into_owned()
            }
            19 => if bytes.first() == Some(&1) { "True" } else { "False" }.to_string(),
            1 => bytes.first().map(|v| v.to_string()).unwrap_or_default(),
            2 => i16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            3 => u16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            4 => i32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            5 => u32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            26 | 32 => i64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            24 => u64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            6 => f32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            7 => f64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            31 => format_uuid(bytes),
            _ => {
                let byte_list: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
                format!("[{}]", byte_list.join(", "))
            }
        })
    }
}

fn format_uuid(bytes: &[u8]) -> String {
    if bytes.len() >= 16 {
        // BG3 uses byte-swapped GUIDs per Windows GUID format
        // Swap: first 4 bytes (reverse), next 2 bytes (reverse), next 2 bytes (reverse),
        // then last 8 bytes swap in pairs
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[3], bytes[2], bytes[1], bytes[0],  // First 4 bytes reversed
            bytes[5], bytes[4],                       // Next 2 bytes reversed
            bytes[7], bytes[6],                       // Next 2 bytes reversed
            bytes[9], bytes[8],                       // Swap pair
            bytes[11], bytes[10],                     // Swap pair
            bytes[13], bytes[12],                     // Swap pair
            bytes[15], bytes[14]                      // Swap pair
        )
    } else {
        String::new()
    }
}