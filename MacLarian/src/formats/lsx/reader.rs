//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! LSX file reading (from your original lsx.rs)

use super::document::{LsxDocument, LsxRegion, LsxNode, LsxAttribute};
use crate::error::{Error, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;
use std::path::Path;

/// Read an LSX file from disk
///
/// # Errors
/// Returns an error if the file cannot be read or has invalid XML.
pub fn read_lsx<P: AsRef<Path>>(path: P) -> Result<LsxDocument> {
    let content = fs::read_to_string(path)?;
    parse_lsx(&content)
}

/// Parse LSX from XML string
///
/// # Errors
/// Returns an error if the XML is malformed or has an invalid structure.
pub fn parse_lsx(content: &str) -> Result<LsxDocument> {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    
    let mut doc = LsxDocument {
        major: 4,
        minor: 0,
        revision: 0,
        build: 0,
        regions: Vec::new(),
    };
    
    let mut buf = Vec::new();
    let mut current_region: Option<LsxRegion> = None;
    let mut node_stack: Vec<LsxNode> = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"version" => {
                        for attr in e.attributes() {
                            let attr = attr?;
                            let value_str = String::from_utf8_lossy(&attr.value);
                            match attr.key.as_ref() {
                                b"major" => doc.major = value_str.parse().unwrap_or(4),
                                b"minor" => doc.minor = value_str.parse().unwrap_or(0),
                                b"revision" => doc.revision = value_str.parse().unwrap_or(0),
                                b"build" => doc.build = value_str.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                    }
                    b"region" => {
                        let mut region_id = String::new();
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"id" {
                                region_id = String::from_utf8_lossy(&attr.value).into_owned();
                            }
                        }
                        current_region = Some(LsxRegion {
                            id: region_id,
                            nodes: Vec::new(),
                        });
                    }
                    b"node" => {
                        let mut node_id = String::new();
                        let mut node_key = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"id" => node_id = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"key" => node_key = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                                _ => {}
                            }
                        }
                        node_stack.push(LsxNode {
                            id: node_id,
                            key: node_key,
                            attributes: Vec::new(),
                            children: Vec::new(),
                        });
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"version" => {
                        for attr in e.attributes() {
                            let attr = attr?;
                            let value_str = String::from_utf8_lossy(&attr.value);
                            match attr.key.as_ref() {
                                b"major" => doc.major = value_str.parse().unwrap_or(4),
                                b"minor" => doc.minor = value_str.parse().unwrap_or(0),
                                b"revision" => doc.revision = value_str.parse().unwrap_or(0),
                                b"build" => doc.build = value_str.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                    }
                    b"attribute" => {
                        let mut attr_id = String::new();
                        let mut attr_type = String::new();
                        let mut attr_value = String::new();
                        let mut handle = None;
                        let mut version = None;
                        
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"id" => attr_id = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"type" => attr_type = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"value" => attr_value = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"handle" => handle = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                                b"version" => version = attr.value.as_ref().iter()
                                    .map(|&b| b as char)
                                    .collect::<String>()
                                    .parse()
                                    .ok(),
                                _ => {}
                            }
                        }
                        
                        if let Some(node) = node_stack.last_mut() {
                            node.attributes.push(LsxAttribute {
                                id: attr_id,
                                type_name: attr_type,
                                value: attr_value,
                                handle,
                                version,
                            });
                        }
                    }
                    b"node" => {
                        // Self-closing node with no attributes/children
                        let mut node_id = String::new();
                        let mut node_key = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"id" => node_id = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"key" => node_key = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                                _ => {}
                            }
                        }
                        
                        let empty_node = LsxNode {
                            id: node_id,
                            key: node_key,
                            attributes: Vec::new(),
                            children: Vec::new(),
                        };
                        
                        if let Some(parent) = node_stack.last_mut() {
                            parent.children.push(empty_node);
                        } else if let Some(ref mut region) = current_region {
                            region.nodes.push(empty_node);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"node" => {
                        if let Some(completed_node) = node_stack.pop() {
                            if let Some(parent) = node_stack.last_mut() {
                                parent.children.push(completed_node);
                            } else if let Some(ref mut region) = current_region {
                                region.nodes.push(completed_node);
                            }
                        }
                    }
                    b"region" => {
                        if let Some(region) = current_region.take() {
                            doc.regions.push(region);
                        }
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
    
    Ok(doc)
}