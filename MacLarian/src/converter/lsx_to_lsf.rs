//! LSX to LSF conversion

use crate::error::{Error, Result};
use crate::formats::lsf::{self, LsfDocument, LsfNode, LsfAttribute};
use crate::formats::common::{
    type_name_to_id, serialize_value, serialize_translated_string, hash_string_lslib
};

use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::path::Path;

/// Convert LSX file to LSF format
pub fn convert_lsx_to_lsf<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    tracing::info!("Converting LSX→LSF: {:?} → {:?}", source.as_ref(), dest.as_ref());
    
    let content = std::fs::read_to_string(&source)?;
    let lsf_doc = from_lsx(&content)?;
    
    // Write LSF binary
    lsf::write_lsf(&lsf_doc, dest)?;
    
    tracing::info!("Conversion complete");
    Ok(())
}

/// Parse LSX XML and build LSF document structure
pub fn from_lsx(content: &str) -> Result<LsfDocument> {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut engine_version: u64 = 0;
    let mut string_table = StringTable::new();
    let mut nodes = Vec::new();
    let mut attributes: Vec<LsfAttribute> = Vec::new();
    let mut values_buffer = Vec::new();
    let mut node_keys = Vec::new();
    
    let mut node_stack: Vec<usize> = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"version" => {
                        engine_version = parse_version(&e)?;
                    }
                    b"node" => {
                        let node_idx = parse_and_create_node(
                            &e,
                            &mut string_table,
                            &mut nodes,
                            &mut node_keys,
                            &node_stack,
                        )?;
                        node_stack.push(node_idx);
                    }
                    b"attribute" => {
                        parse_and_create_attribute(
                            &e,
                            &mut string_table,
                            &mut attributes,
                            &mut values_buffer,
                            &mut nodes,
                            &node_stack,
                        )?;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"version" => {
                        engine_version = parse_version(&e)?;
                    }
                    b"node" => {
                        // Self-closing node - create but don't push to stack
                        parse_and_create_node(
                            &e,
                            &mut string_table,
                            &mut nodes,
                            &mut node_keys,
                            &node_stack,
                        )?;
                    }
                    b"attribute" => {
                        parse_and_create_attribute(
                            &e,
                            &mut string_table,
                            &mut attributes,
                            &mut values_buffer,
                            &mut nodes,
                            &node_stack,
                        )?;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"node" => {
                        node_stack.pop();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Error::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }
    
    Ok(LsfDocument {
        engine_version,
        names: string_table.to_name_lists(),
        nodes,
        attributes,
        values: values_buffer,
        node_keys,
    })
}

fn parse_version(e: &quick_xml::events::BytesStart) -> Result<u64> {
    let mut major = 0u32;
    let mut minor = 0u32;
    let mut revision = 0u32;
    let mut build = 0u32;
    
    for attr in e.attributes() {
        let attr = attr?;
        let value = String::from_utf8_lossy(&attr.value);
        match attr.key.as_ref() {
            b"major" => major = value.parse().unwrap_or(4),
            b"minor" => minor = value.parse().unwrap_or(0),
            b"revision" => revision = value.parse().unwrap_or(0),
            b"build" => build = value.parse().unwrap_or(0),
            _ => {}
        }
    }
    
    // Pack version into u64
    Ok(((major as u64 & 0x7F) << 55)
        | ((minor as u64 & 0xFF) << 47)
        | ((revision as u64 & 0xFFFF) << 31)
        | (build as u64 & 0x7FFFFFFF))
}

fn parse_and_create_node(
    e: &quick_xml::events::BytesStart,
    string_table: &mut StringTable,
    nodes: &mut Vec<LsfNode>,
    node_keys: &mut Vec<Option<String>>,
    node_stack: &[usize],
) -> Result<usize> {
    let mut node_id = String::new();
    let mut node_key: Option<String> = None;
    
    for attr in e.attributes() {
        let attr = attr?;
        let value = String::from_utf8_lossy(&attr.value).into_owned();
        match attr.key.as_ref() {
            b"id" => node_id = value,
            b"key" => node_key = Some(value),
            _ => {}
        }
    }
    
    let (name_outer, name_inner) = string_table.get_or_insert(&node_id);
    
    // Parent is the last node on the stack, or -1 if stack is empty
    let parent_index = node_stack.last().map(|&idx| idx as i32).unwrap_or(-1);
    
    let node = LsfNode {
        name_index_outer: name_outer,
        name_index_inner: name_inner,
        parent_index,
        first_attribute_index: -1,
    };
    
    let node_idx = nodes.len();
    nodes.push(node);
    node_keys.push(node_key);
    
    Ok(node_idx)
}

fn parse_and_create_attribute(
    e: &quick_xml::events::BytesStart,
    string_table: &mut StringTable,
    attributes: &mut Vec<LsfAttribute>,
    values_buffer: &mut Vec<u8>,
    nodes: &mut [LsfNode],
    node_stack: &[usize],
) -> Result<()> {
    let mut attr_id = String::new();
    let mut attr_type = String::new();
    let mut attr_value = String::new();
    let mut handle = String::new();
    let mut version: u16 = 0;
    
    for attr in e.attributes() {
        let attr = attr?;
        let value = String::from_utf8_lossy(&attr.value).into_owned();
        match attr.key.as_ref() {
            b"id" => attr_id = value,
            b"type" => attr_type = value,
            b"value" => attr_value = value,
            b"handle" => handle = value,
            b"version" => version = value.parse().unwrap_or(0),
            _ => {}
        }
    }
    
    if let Some(current_node_idx) = node_stack.last() {
        let type_id = type_name_to_id(&attr_type);
        let (name_outer, name_inner) = string_table.get_or_insert(&attr_id);
        
        // Serialize value to bytes
        let value_offset = values_buffer.len();
        let value_length = if type_id == 28 {
            // TranslatedString special handling
            serialize_translated_string(values_buffer, &handle, version, &attr_value)?
        } else {
            serialize_value(values_buffer, type_id, &attr_value)?
        };
        
        let type_info = type_id | ((value_length as u32) << 6);
        
        let attr = LsfAttribute {
            name_index_outer: name_outer,
            name_index_inner: name_inner,
            type_info,
            next_index: -1,
            offset: value_offset,
        };
        
        let attr_idx = attributes.len();
        
        // Link attribute to node
        let node = &mut nodes[*current_node_idx];
        if node.first_attribute_index == -1 {
            node.first_attribute_index = attr_idx as i32;
        } else {
            // Find last attribute in chain and link
            let mut last_idx = node.first_attribute_index as usize;
            while attributes[last_idx].next_index != -1 {
                last_idx = attributes[last_idx].next_index as usize;
            }
            attributes[last_idx].next_index = attr_idx as i32;
        }
        
        attributes.push(attr);
    }
    
    Ok(())
}

const STRING_HASH_MAP_SIZE: usize = 0x200; // 512 buckets

/// String table for managing name indices
struct StringTable {
    /// Maps string -> (outer_index, inner_index)
    string_map: HashMap<String, (usize, usize)>,
    /// Lists of strings grouped by hash
    name_lists: Vec<Vec<String>>,
}

impl StringTable {
    fn new() -> Self {
        // Pre-allocate all 512 buckets
        let mut name_lists = Vec::with_capacity(STRING_HASH_MAP_SIZE);
        for _ in 0..STRING_HASH_MAP_SIZE {
            name_lists.push(Vec::new());
        }
        
        Self {
            string_map: HashMap::new(),
            name_lists,
        }
    }
    
    fn get_or_insert(&mut self, s: &str) -> (usize, usize) {
        if let Some(&indices) = self.string_map.get(s) {
            return indices;
        }
        
        // Use LSLib's hash algorithm
        let hash = hash_string_lslib(s);
        let bucket = ((hash & 0x1ff) ^ ((hash >> 9) & 0x1ff) ^ ((hash >> 18) & 0x1ff) ^ ((hash >> 27) & 0x1ff)) as usize;
        
        let outer = bucket;
        let inner = self.name_lists[outer].len();
        
        self.name_lists[outer].push(s.to_string());
        self.string_map.insert(s.to_string(), (outer, inner));
        
        (outer, inner)
    }
    
    fn to_name_lists(self) -> Vec<Vec<String>> {
        self.name_lists
    }
}