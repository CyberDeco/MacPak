//! LSX file writing
//!
//! `LSLib`'s metadata output purposefully maintained as an homage.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

use super::document::{LsxDocument, LsxNode};
use crate::error::Result;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use std::fs;
use std::path::Path;

/// Write an LSX document to disk
///
/// # Errors
/// Returns an error if serialization or file writing fails.
pub fn write_lsx<P: AsRef<Path>>(doc: &LsxDocument, path: P) -> Result<()> {
    let xml = serialize_lsx(doc)?;
    fs::write(path, xml)?;
    Ok(())
}

/// Serialize LSX document to XML string
///
/// # Errors
/// Returns an error if XML serialization fails.
pub fn serialize_lsx(doc: &LsxDocument) -> Result<String> {
    let mut output = Vec::new();

    // Write UTF-8 BOM
    output.extend_from_slice(&[0xEF, 0xBB, 0xBF]);

    let mut writer = Writer::new_with_indent(&mut output, b'\t', 1);

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

    // <save>
    writer.write_event(Event::Start(BytesStart::new("save")))?;

    // <version>
    let mut version = BytesStart::new("version");
    version.push_attribute(("major", doc.major.to_string().as_str()));
    version.push_attribute(("minor", doc.minor.to_string().as_str()));
    version.push_attribute(("revision", doc.revision.to_string().as_str()));
    version.push_attribute(("build", doc.build.to_string().as_str()));

    // BG3 and DOS2 use byte-swapped GUIDs
    if doc.major >= 4 {
        version.push_attribute(("lslib_meta", "v1,bswap_guids"));
    } else {
        // Older games (DOS1) don't byte-swap
        version.push_attribute(("lslib_meta", "v1"));
    }

    writer.write_event(Event::Empty(version))?;

    // <region>s
    for region in &doc.regions {
        let mut region_tag = BytesStart::new("region");
        region_tag.push_attribute(("id", region.id.as_str()));
        writer.write_event(Event::Start(region_tag.borrow()))?;

        // Write root nodes
        for node in &region.nodes {
            write_node(&mut writer, node)?;
        }

        writer.write_event(Event::End(BytesEnd::new("region")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("save")))?;

    let xml = String::from_utf8(output)?;
    // Convert to Windows line endings (CRLF) to match LSLib output
    let xml = xml.replace('\n', "\r\n");
    // Fix spacing before self-closing tags
    let xml = xml.replace("/>", " />");
    Ok(xml)
}

fn write_node<W: std::io::Write>(writer: &mut Writer<W>, node: &LsxNode) -> Result<()> {
    let has_attributes = !node.attributes.is_empty();
    let has_children = !node.children.is_empty();

    let mut node_start = BytesStart::new("node");
    node_start.push_attribute(("id", node.id.as_str()));

    if let Some(ref key) = node.key {
        node_start.push_attribute(("key", key.as_str()));
    }

    if !has_attributes && !has_children {
        writer.write_event(Event::Empty(node_start))?;
        return Ok(());
    }

    writer.write_event(Event::Start(node_start.borrow()))?;

    // Write attributes
    if has_attributes {
        for attr in &node.attributes {
            let mut attr_tag = BytesStart::new("attribute");
            attr_tag.push_attribute(("id", attr.id.as_str()));
            attr_tag.push_attribute(("type", attr.type_name.as_str()));
            //attr_tag.push_attribute(("value", attr.value.as_str()));

            if let Some(ref handle) = attr.handle {
                // TranslatedString or TranslatedFSString
                attr_tag.push_attribute(("handle", handle.as_str()));

                if !attr.value.is_empty() {
                    attr_tag.push_attribute(("value", attr.value.as_str()));
                }

                if let Some(version) = attr.version {
                    attr_tag.push_attribute(("version", version.to_string().as_str()));
                }
            } else {
                attr_tag.push_attribute(("value", attr.value.as_str()));
            }
            writer.write_event(Event::Empty(attr_tag))?;
        }
    }

    // Write children
    if has_children {
        writer.write_event(Event::Start(BytesStart::new("children")))?;
        for child in &node.children {
            write_node(writer, child)?;
        }
        writer.write_event(Event::End(BytesEnd::new("children")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("node")))?;
    Ok(())
}
