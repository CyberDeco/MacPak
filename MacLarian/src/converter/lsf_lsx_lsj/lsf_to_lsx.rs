//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! LSF to LSX conversion

use crate::error::Result;
use crate::formats::lsf::{self, LsfDocument, LsfMetadataFormat};
use crate::formats::common::{get_type_name, extract_value, extract_translated_string};

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::collections::HashMap;
use std::path::Path;

/// Convert LSF file to LSX format
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_lsf_to_lsx<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    tracing::info!("Converting LSF→LSX: {:?} → {:?}", source.as_ref(), dest.as_ref());
    let lsf_doc = lsf::read_lsf(&source)?;
    let lsx_xml = to_lsx(&lsf_doc)?;
    std::fs::write(dest, lsx_xml)?;
    tracing::info!("Conversion complete");
    Ok(())
}

/// Convert LSF document to LSX XML string
///
/// # Errors
/// Returns an error if XML serialization fails.
pub fn to_lsx(doc: &LsfDocument) -> Result<String> {
    let mut output = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output, b'\t', 1);

    // Pre-build children index for O(1) lookup instead of O(n) per node
    // This changes overall complexity from O(n^2) to O(n)
    let children_by_parent: HashMap<i32, Vec<usize>> = {
        let mut map: HashMap<i32, Vec<usize>> = HashMap::with_capacity(doc.nodes.len());
        for (idx, node) in doc.nodes.iter().enumerate() {
            map.entry(node.parent_index).or_default().push(idx);
        }
        map
    };

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

    // <save>
    writer.write_event(Event::Start(BytesStart::new("save")))?;

    // <version>
    write_version(&mut writer, doc)?;

    // Each root node gets its own <region> wrapper
    for (i, node) in doc.nodes.iter().enumerate() {
        if node.parent_index == -1 {
            let region_id = doc.get_name(node.name_index_outer, node.name_index_inner)?;
            let mut region = BytesStart::new("region");
            region.push_attribute(("id", region_id));
            writer.write_event(Event::Start(region.borrow()))?;

            write_node(&mut writer, doc, i, &children_by_parent)?;

            writer.write_event(Event::End(BytesEnd::new("region")))?;
        }
    }

    writer.write_event(Event::End(BytesEnd::new("save")))?;

    let mut xml = String::from_utf8(output)?;
    // Fix spacing before self-closing tags
    xml = xml.replace("/>", " />");
    // Add trailing newline
    xml.push('\n');
    Ok(xml)
}

fn write_version<W: std::io::Write>(writer: &mut Writer<W>, doc: &LsfDocument) -> Result<()> {
    let mut major = ((doc.engine_version >> 55) & 0x7F) as u32;
    let mut minor = ((doc.engine_version >> 47) & 0xFF) as u32;
    let mut revision = ((doc.engine_version >> 31) & 0xFFFF) as u32;
    let mut build = (doc.engine_version & 0x7FFFFFFF) as u32;

    // Workaround for merged LSF files with missing engine version number (matches LSLib)
    if major == 0 {
        major = 4;
        minor = 0;
        revision = 9;
        build = 0;
    }
    
    let mut version = BytesStart::new("version");
    version.push_attribute(("major", major.to_string().as_str()));
    version.push_attribute(("minor", minor.to_string().as_str()));
    version.push_attribute(("revision", revision.to_string().as_str()));
    version.push_attribute(("build", build.to_string().as_str()));
    
    // Build metadata based on what we detected
    let mut meta = vec!["v1"];
    if major >= 4 {
        meta.push("bswap_guids");
    }
    // Use metadata format from header to determine adjacency tag
    match doc.metadata_format {
        LsfMetadataFormat::KeysAndAdjacency => meta.push("lsf_keys_adjacency"),
        LsfMetadataFormat::None2 => meta.push("lsf_adjacency"),
        LsfMetadataFormat::None => {}
    }
    
    version.push_attribute(("lslib_meta", meta.join(",").as_str()));
    writer.write_event(Event::Empty(version))?;
    Ok(())
}

fn write_node<W: std::io::Write>(
    writer: &mut Writer<W>,
    doc: &LsfDocument,
    node_idx: usize,
    children_by_parent: &HashMap<i32, Vec<usize>>,
) -> Result<()> {
    let node = &doc.nodes[node_idx];
    let node_name = doc.get_name(node.name_index_outer, node.name_index_inner)?;

    let has_attributes = node.first_attribute_index >= 0
        && (node.first_attribute_index as usize) < doc.attributes.len();
    // O(1) lookup instead of O(n) filter
    let children = children_by_parent.get(&(node_idx as i32));
    let has_children = children.is_some_and(|c| !c.is_empty());

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
            if attr_idx >= doc.attributes.len() {
                break;
            }
            write_attribute(writer, doc, attr_idx)?;
            let attr = &doc.attributes[attr_idx];
            if attr.next_index < 0 {
                break;
            }
            attr_idx = attr.next_index as usize;
        }
    }

    if let Some(child_indices) = children {
        writer.write_event(Event::Start(BytesStart::new("children")))?;
        for &child_idx in child_indices {
            write_node(writer, doc, child_idx, children_by_parent)?;
        }
        writer.write_event(Event::End(BytesEnd::new("children")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("node")))?;
    Ok(())
}

fn write_attribute<W: std::io::Write>(writer: &mut Writer<W>, doc: &LsfDocument, attr_idx: usize) -> Result<()> {
    let attr = doc.attributes.get(attr_idx)
        .ok_or_else(|| crate::error::Error::InvalidIndex(format!(
            "Attribute index {} out of bounds (len: {})", attr_idx, doc.attributes.len()
        )))?;
    let attr_name = doc.get_name(attr.name_index_outer, attr.name_index_inner)?;
    let type_id = attr.type_info & 0x3F;
    let value_length = (attr.type_info >> 6) as usize;
    
    let type_name = get_type_name(type_id);
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