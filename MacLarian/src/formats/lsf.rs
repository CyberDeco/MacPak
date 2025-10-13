//! LSF (Larian Story Format) binary format handler

use crate::error::{Error, Result};
use crate::formats::common::AttributeValue;
use byteorder::{LittleEndian, ReadBytesExt};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::Path;

// Re-use larian-formats for validation
use larian_formats::lsf::Reader as LsfValidator;

/// Main conversion function
pub fn convert_lsf_to_lsx<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    tracing::info!("Converting LSF→LSX: {:?} → {:?}", source.as_ref(), dest.as_ref());
    
    // Step 1: Validate with larian-formats
    validate_lsf_file(&source)?;
    
    // Step 2: Parse with our own parser
    let lsf_doc = parse_lsf_file(&source)?;
    
    // Step 3: Convert to LSX XML
    let lsx_xml = lsf_doc.to_lsx()?;
    
    // Step 4: Write output
    std::fs::write(dest, lsx_xml)?;
    
    tracing::info!("Conversion complete");
    Ok(())
}

/// Validate LSF file using larian-formats
fn validate_lsf_file<P: AsRef<Path>>(source: P) -> Result<()> {
    let file = File::open(source.as_ref())?;
    let reader = BufReader::new(file);
    
    let mut validator = LsfValidator::new(reader)
        .map_err(|e| Error::ConversionError(format!("Invalid LSF: {:?}", e)))?;
    
    validator.read()
        .map_err(|e| Error::ConversionError(format!("LSF parse failed: {:?}", e)))?;
    
    Ok(())
}

/// Our own LSF document structure with public fields
#[derive(Debug)]
pub struct LsfDocument {
    version: u32,
    engine_version: u64,
    names: Vec<Vec<String>>,
    nodes: Vec<LsfNode>,
    attributes: Vec<LsfAttribute>,
    values: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct LsfNode {
    name_index_outer: usize,
    name_index_inner: usize,
    parent_index: i32,
    next_sibling_index: i32,
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
    /// Parse LSF file with full access to internals
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        let mut cursor = Cursor::new(buffer);
        Self::from_reader(&mut cursor)
    }
    
    fn from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        println!("Magic: {:?}", magic);
        if &magic != b"LSOF" {
            return Err(Error::InvalidLsfMagic(magic));
        }
        
        // Read version
        let version = reader.read_u32::<LittleEndian>()?;
        println!("Version: {}", version);
        if version < 2 || version > 7 {
            return Err(Error::UnsupportedLsfVersion(version));
        }
        
        let engine_version = reader.read_u64::<LittleEndian>()?;
        println!("Engine version: {}", engine_version);
        
        // Read section metadata
        let strings_uncompressed = reader.read_u32::<LittleEndian>()? as usize;
        let strings_compressed = reader.read_u32::<LittleEndian>()? as usize;
        println!("Strings: {} compressed → {} uncompressed", strings_compressed, strings_uncompressed);
        
        let keys_uncompressed = reader.read_u32::<LittleEndian>()? as usize;
        let keys_compressed = reader.read_u32::<LittleEndian>()? as usize;
        println!("Keys: {} compressed → {} uncompressed", keys_compressed, keys_uncompressed);
        
        let nodes_uncompressed = reader.read_u32::<LittleEndian>()? as usize;
        let nodes_compressed = reader.read_u32::<LittleEndian>()? as usize;
        println!("Nodes: {} compressed → {} uncompressed", nodes_compressed, nodes_uncompressed);
        
        let attrs_uncompressed = reader.read_u32::<LittleEndian>()? as usize;
        let attrs_compressed = reader.read_u32::<LittleEndian>()? as usize;
        println!("Attrs: {} compressed → {} uncompressed", attrs_compressed, attrs_uncompressed);
        
        let values_uncompressed = reader.read_u32::<LittleEndian>()? as usize;
        let values_compressed = reader.read_u32::<LittleEndian>()? as usize;
        println!("Values: {} compressed → {} uncompressed", values_compressed, values_uncompressed);
        
        let compression_flags = reader.read_u32::<LittleEndian>()?;
        let extended_format = reader.read_u32::<LittleEndian>()?;
        println!("Compression flags: {:#x}, Extended: {:#x}", compression_flags, extended_format);
        
        let is_compressed = compression_flags & 0x0F != 0;
        println!("Is compressed: {}", is_compressed);
        
        // Read and decompress strings
        println!("\nReading strings section...");
        let names = Self::read_names(reader, strings_compressed, strings_uncompressed, is_compressed)?;
        println!("Read {} name lists", names.len());
        
        // Read and decompress nodes
        println!("\nReading nodes section...");
        let nodes = Self::read_nodes(reader, nodes_compressed, nodes_uncompressed, is_compressed)?;
        println!("Read {} nodes", nodes.len());
        
        // Read and decompress attributes
        println!("\nReading attributes section...");
        let attributes = Self::read_attributes(reader, attrs_compressed, attrs_uncompressed, is_compressed)?;
        println!("Read {} attributes", attributes.len());
        
        // Read and decompress values
        println!("\nReading values section...");
        let values = Self::read_values(reader, values_compressed, values_uncompressed, is_compressed)?;
        println!("Read {} value bytes", values.len());
        
        Ok(LsfDocument {
            version,
            engine_version,
            names,
            nodes,
            attributes,
            values,
        })
    }
    
    fn read_names<R: Read>(
        reader: &mut R,
        compressed_size: usize,
        uncompressed_size: usize,
        is_compressed: bool,
    ) -> Result<Vec<Vec<String>>> {
        // Read the data (compressed or not)
        let data = Self::read_and_decompress(reader, compressed_size, uncompressed_size, is_compressed)?;
        
        // Parse names structure
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
        compressed_size: usize,
        uncompressed_size: usize,
        is_compressed: bool,
    ) -> Result<Vec<LsfNode>> {
        let data = Self::read_and_decompress(reader, compressed_size, uncompressed_size, is_compressed)?;
        let mut cursor = Cursor::new(data);
        let mut nodes = Vec::new();
        
        // Each node is 16 bytes in v7
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
                next_sibling_index,
                first_attribute_index,
            });
        }
        
        Ok(nodes)
    }
    
    fn read_attributes<R: Read>(
        reader: &mut R,
        compressed_size: usize,
        uncompressed_size: usize,
        is_compressed: bool,
    ) -> Result<Vec<LsfAttribute>> {
        let data = Self::read_and_decompress(reader, compressed_size, uncompressed_size, is_compressed)?;
        let mut cursor = Cursor::new(data);
        let mut attributes = Vec::new();
        
        // Each attribute is 16 bytes
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
    
    fn read_values<R: Read>(
        reader: &mut R,
        compressed_size: usize,
        uncompressed_size: usize,
        is_compressed: bool,
    ) -> Result<Vec<u8>> {
        Self::read_and_decompress(reader, compressed_size, uncompressed_size, is_compressed)
    }
    
    fn read_and_decompress<R: Read>(
        reader: &mut R,
        compressed_size: usize,
        uncompressed_size: usize,
        is_compressed: bool,
    ) -> Result<Vec<u8>> {
        // If not compressed and compressed_size is 0, use uncompressed_size
        let actual_size = if !is_compressed && compressed_size == 0 {
            uncompressed_size
        } else {
            compressed_size
        };
        
        let mut buffer = vec![0u8; actual_size];
        reader.read_exact(&mut buffer)?;
        
        if is_compressed {
            lz4_flex::decompress(&buffer, uncompressed_size)
                .map_err(|e| Error::DecompressionError(format!("LZ4: {}", e)))
        } else {
            Ok(buffer)
        }
    }
    
    /// Convert to LSX XML
    pub fn to_lsx(&self) -> Result<String> {
        let mut output = Vec::new();
        let mut writer = Writer::new_with_indent(&mut output, b' ', 2);
        
        // XML declaration
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;
        
        // <save>
        writer.write_event(Event::Start(BytesStart::new("save")))?;
        
        // <version>
        let mut version = BytesStart::new("version");
        version.push_attribute(("major", self.version.to_string().as_str()));
        version.push_attribute(("minor", "0"));
        version.push_attribute(("revision", "0"));
        version.push_attribute(("build", "0"));
        writer.write_event(Event::Empty(version))?;
        
        // <region>
        let mut region = BytesStart::new("region");
        region.push_attribute(("id", "root"));
        writer.write_event(Event::Start(region.borrow()))?;
        
        // Write nodes
        for (i, node) in self.nodes.iter().enumerate() {
            if node.parent_index == -1 {
                self.write_node(&mut writer, i)?;
            }
        }
        
        writer.write_event(Event::End(BytesEnd::new("region")))?;
        writer.write_event(Event::End(BytesEnd::new("save")))?;
        
        Ok(String::from_utf8(output)?)
    }
    
    fn write_node<W: std::io::Write>(&self, writer: &mut Writer<W>, node_idx: usize) -> Result<()> {
        let node = &self.nodes[node_idx];
        
        // Get node name
        let node_name = self.get_name(node.name_index_outer, node.name_index_inner)?;
        
        // <node>
        let mut node_start = BytesStart::new("node");
        node_start.push_attribute(("id", node_name));
        writer.write_event(Event::Start(node_start.borrow()))?;
        
        // Write attributes
        if node.first_attribute_index >= 0 {
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
        
        // Write child nodes
        let mut next_idx = node.next_sibling_index;
        while next_idx >= 0 {
            let next_node = &self.nodes[next_idx as usize];
            if next_node.parent_index == node_idx as i32 {
                self.write_node(writer, next_idx as usize)?;
            }
            next_idx = next_node.next_sibling_index;
        }
        
        writer.write_event(Event::End(BytesEnd::new("node")))?;
        Ok(())
    }
    
    fn write_attribute<W: std::io::Write>(&self, writer: &mut Writer<W>, attr_idx: usize) -> Result<()> {
        let attr = &self.attributes[attr_idx];
        
        // Get attribute name
        let attr_name = self.get_name(attr.name_index_outer, attr.name_index_inner)?;
        
        // Get type
        let type_id = attr.type_info & 0x3F;
        let type_name = AttributeValue::from_type_id(type_id);
        
        // Get value length
        let value_length = (attr.type_info >> 6) as usize;
        
        // Extract value
        let value_str = self.extract_value(attr.offset, value_length, type_id)?;
        
        // <attribute>
        let mut attr_start = BytesStart::new("attribute");
        attr_start.push_attribute(("id", attr_name));
        attr_start.push_attribute(("type", type_name));
        attr_start.push_attribute(("value", value_str.as_str()));
        writer.write_event(Event::Empty(attr_start))?;
        
        Ok(())
    }
    
    fn get_name(&self, outer: usize, inner: usize) -> Result<&str> {
        self.names.get(outer)
            .and_then(|list| list.get(inner))
            .map(|s| s.as_str())
            .ok_or_else(|| Error::InvalidStringIndex(inner as i32))
    }
    
    fn extract_value(&self, offset: usize, length: usize, type_id: u32) -> Result<String> {
        if offset + length > self.values.len() {
            return Ok(String::new());
        }
        
        let bytes = &self.values[offset..offset + length];
        
        Ok(match type_id {
            // String types
            20 | 21 | 22 | 23 | 29 | 30 => {
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                String::from_utf8_lossy(&bytes[..end]).into_owned()
            }
            // Bool
            19 => match bytes.get(0) {
                Some(0) => "False".to_string(),
                Some(1) => "True".to_string(),
                _ => "False".to_string(),
            }
            // Integer types
            1 => bytes.get(0).map(|v| v.to_string()).unwrap_or_default(),
            2 => i16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            3 => u16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            4 => i32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            5 => u32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            26 | 32 => i64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            24 => u64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            // Float types
            6 => f32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            7 => f64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
            // UUID
            31 => format_uuid(bytes),
            // Default: hex representation
            _ => format!("{:?}", bytes),
        })
    }
}

fn format_uuid(bytes: &[u8]) -> String {
    if bytes.len() >= 16 {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    } else {
        String::new()
    }
}

/// Parse LSF file
fn parse_lsf_file<P: AsRef<Path>>(path: P) -> Result<LsfDocument> {
    LsfDocument::from_file(path)
}