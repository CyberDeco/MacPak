//! LSF to LSX conversion

use crate::error::Result;
use crate::formats::lsf::LsfDocument;
use byteorder::{LittleEndian, ReadBytesExt};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::{Cursor, Read};
use std::path::Path;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Convert LSF file to LSX format
pub fn convert_lsf_to_lsx<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    tracing::info!("Converting LSF→LSX: {:?} → {:?}", source.as_ref(), dest.as_ref());
    let lsf_doc = LsfDocument::from_file(&source)?;
    let lsx_xml = to_lsx(&lsf_doc)?;
    std::fs::write(dest, lsx_xml)?;
    tracing::info!("Conversion complete");
    Ok(())
}

/// Convert LSF document to LSX XML string
pub fn to_lsx(doc: &LsfDocument) -> Result<String> {
    let mut output = Vec::new();
    
    // Write UTF-8 BOM
    output.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    
    let mut writer = Writer::new_with_indent(&mut output, b'\t', 1);
    
    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;
    
    // <save>
    writer.write_event(Event::Start(BytesStart::new("save")))?;
    
    // <version>
    write_version(&mut writer, doc.engine_version)?;
    
    // <region>
    let region_id = doc.nodes.first()
        .filter(|n| n.parent_index == -1)
        .map(|n| doc.get_name(n.name_index_outer, n.name_index_inner))
        .transpose()?
        .unwrap_or("root");
    
    let mut region = BytesStart::new("region");
    region.push_attribute(("id", region_id));
    writer.write_event(Event::Start(region.borrow()))?;
    
    // Write root nodes
    for (i, node) in doc.nodes.iter().enumerate() {
        if node.parent_index == -1 {
            write_node(&mut writer, doc, i)?;
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

fn write_version<W: std::io::Write>(writer: &mut Writer<W>, engine_version: u64) -> Result<()> {
    let major = ((engine_version >> 55) & 0x7F) as u32;
    let minor = ((engine_version >> 47) & 0xFF) as u32;
    let revision = ((engine_version >> 31) & 0xFFFF) as u32;
    let build = (engine_version & 0x7FFFFFFF) as u32;
    
    let mut version = BytesStart::new("version");
    version.push_attribute(("major", major.to_string().as_str()));
    version.push_attribute(("minor", minor.to_string().as_str()));
    version.push_attribute(("revision", revision.to_string().as_str()));
    version.push_attribute(("build", build.to_string().as_str()));
    version.push_attribute(("lslib_meta", "v1,bswap_guids,lsf_keys_adjacency"));
    writer.write_event(Event::Empty(version))?;
    Ok(())
}

fn write_node<W: std::io::Write>(writer: &mut Writer<W>, doc: &LsfDocument, node_idx: usize) -> Result<()> {
    let node = &doc.nodes[node_idx];
    let node_name = doc.get_name(node.name_index_outer, node.name_index_inner)?;
    
    let has_attributes = node.first_attribute_index >= 0;
    let children: Vec<_> = doc.nodes
        .iter()
        .enumerate()
        .filter(|(_, child)| child.parent_index == node_idx as i32)
        .collect();
    let has_children = !children.is_empty();
    
    // Get key attribute from the keys section
    let key_attr = doc.node_keys.get(node_idx).and_then(|k| k.as_deref());
    
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
            write_attribute(writer, doc, attr_idx)?;
            let attr = &doc.attributes[attr_idx];
            if attr.next_index < 0 {
                break;
            }
            attr_idx = attr.next_index as usize;
        }
    }
    
    if has_children {
        writer.write_event(Event::Start(BytesStart::new("children")))?;
        for (child_idx, _) in children {
            write_node(writer, doc, child_idx)?;
        }
        writer.write_event(Event::End(BytesEnd::new("children")))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("node")))?;
    Ok(())
}

fn write_attribute<W: std::io::Write>(writer: &mut Writer<W>, doc: &LsfDocument, attr_idx: usize) -> Result<()> {
    let attr = &doc.attributes[attr_idx];
    let attr_name = doc.get_name(attr.name_index_outer, attr.name_index_inner)?;
    let type_id = attr.type_info & 0x3F;
    let value_length = (attr.type_info >> 6) as usize;
    
    let type_name = match type_id {
        0 => "None",
        1 => "uint8",
        2 => "int16",
        3 => "uint16",
        4 => "int32",
        5 => "uint32",
        6 => "float",
        7 => "double",
        8 => "ivec2",
        9 => "ivec3",
        10 => "ivec4",
        11 => "fvec2",
        12 => "fvec3",
        13 => "fvec4",
        14 => "mat2x2",
        15 => "mat3x3",
        16 => "mat3x4",
        17 => "mat4x3",
        18 => "mat4x4",
        19 => "bool",
        20 => "string",
        21 => "path",
        22 => "FixedString",
        23 => "LSString",
        24 => "uint64",
        25 => "ScratchBuffer",
        26 => "old_int64",
        27 => "int8",
        28 => "TranslatedString",
        29 => "WString",
        30 => "LSWString",
        31 => "guid",
        32 => "int64",
        33 => "TranslatedFSString",
        _ => "Unknown",
    };
    
    let value_str = extract_value(&doc.values, attr.offset, value_length, type_id)?;
    
    let mut attr_start = BytesStart::new("attribute");
    attr_start.push_attribute(("id", attr_name));
    attr_start.push_attribute(("type", type_name));
    
    // TranslatedString has special format: handle and version instead of value
    if type_id == 28 {
        if let Ok((handle, version, value)) = extract_translated_string(&doc.values, attr.offset, value_length) {
            attr_start.push_attribute(("handle", handle.as_str()));
            if let Some(val) = value {
                attr_start.push_attribute(("value", val.as_str()));
            } else {
                attr_start.push_attribute(("version", version.to_string().as_str()));
            }
        }
    } else {
        attr_start.push_attribute(("value", value_str.as_str()));
    }
    
    writer.write_event(Event::Empty(attr_start))?;
    Ok(())
}

fn extract_translated_string(values: &[u8], offset: usize, length: usize) -> Result<(String, u16, Option<String>)> {
    if offset + length > values.len() {
        return Ok((String::new(), 0, None));
    }
    
    let bytes = &values[offset..offset + length];
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
    
    // For BG3, the value is typically empty, so just return None
    let value = None;
    
    Ok((handle, version, value))
}

fn extract_value(values: &[u8], offset: usize, length: usize, type_id: u32) -> Result<String> {
    if offset + length > values.len() {
        return Ok(String::new());
    }
    
    let bytes = &values[offset..offset + length];
    
    Ok(match type_id {
        // String types (null-terminated)
        20 | 21 | 22 | 23 | 29 | 30 => {
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            String::from_utf8_lossy(&bytes[..end]).into_owned()
        }
        // Bool
        19 => if bytes.first() == Some(&1) { "True" } else { "False" }.to_string(),
        // Integer types
        0 => String::new(), // None type
        1 | 27 => bytes.first().map(|v| v.to_string()).unwrap_or_default(),
        2 => i16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        3 => u16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        4 => i32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        5 => u32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        24 => u64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        26 | 32 => i64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        // Float types
        6 => f32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        7 => f64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        // Vector types (space-separated values)
        8 => format_ivec(bytes, 2),
        9 => format_ivec(bytes, 3),
        10 => format_ivec(bytes, 4),
        11 => format_fvec(bytes, 2),
        12 => format_fvec(bytes, 3),
        13 => format_fvec(bytes, 4),
        // Matrix types (space-separated values)
        14 | 15 | 16 | 17 | 18 => format_matrix(bytes),
        // UUID
        31 => format_uuid(bytes),
        // Binary types
        25 => BASE64.encode(bytes), // ScratchBuffer as base64
        // Default: byte array for unknown/unhandled types
        _ => {
            let byte_list: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
            format!("[{}]", byte_list.join(", "))
        }
    })
}

fn format_ivec(bytes: &[u8], count: usize) -> String {
    let values: Vec<String> = (0..count)
        .filter_map(|i| {
            let offset = i * 4;
            if offset + 4 <= bytes.len() {
                Some(i32::from_le_bytes(bytes[offset..offset+4].try_into().ok()?).to_string())
            } else {
                None
            }
        })
        .collect();
    values.join(" ")
}

fn format_fvec(bytes: &[u8], count: usize) -> String {
    let values: Vec<String> = (0..count)
        .filter_map(|i| {
            let offset = i * 4;
            if offset + 4 <= bytes.len() {
                Some(f32::from_le_bytes(bytes[offset..offset+4].try_into().ok()?).to_string())
            } else {
                None
            }
        })
        .collect();
    values.join(" ")
}

fn format_matrix(bytes: &[u8]) -> String {
    // Matrices are stored as space-separated floats
    let count = bytes.len() / 4;
    let values: Vec<String> = (0..count)
        .filter_map(|i| {
            let offset = i * 4;
            if offset + 4 <= bytes.len() {
                Some(f32::from_le_bytes(bytes[offset..offset+4].try_into().ok()?).to_string())
            } else {
                None
            }
        })
        .collect();
    values.join(" ")
}

fn format_uuid(bytes: &[u8]) -> String {
    if bytes.len() >= 16 {
        // BG3 uses byte-swapped GUIDs per Windows GUID format
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