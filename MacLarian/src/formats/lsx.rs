//! LSX (XML) format handler

use crate::error::{Error, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxDocument {
    pub version: u32,
    pub regions: Vec<LsxRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxRegion {
    pub id: String,
    pub nodes: Vec<LsxNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxNode {
    pub id: String,
    pub attributes: Vec<LsxAttribute>,
    pub children: Vec<LsxNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxAttribute {
    pub id: String,
    pub type_name: String,
    pub value: String,
}

impl LsxDocument {
    pub fn from_xml(content: &str) -> Result<Self> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);
        
        let mut doc = LsxDocument {
            version: 4,
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
                                if attr.key.as_ref() == b"major" {
                                    doc.version = String::from_utf8_lossy(&attr.value)
                                        .parse()
                                        .unwrap_or(4);
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
                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"id" {
                                    node_id = String::from_utf8_lossy(&attr.value).into_owned();
                                }
                            }
                            node_stack.push(LsxNode {
                                id: node_id,
                                attributes: Vec::new(),
                                children: Vec::new(),
                            });
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(e)) => {
                    if e.name().as_ref() == b"attribute" {
                        let mut attr_id = String::new();
                        let mut attr_type = String::new();
                        let mut attr_value = String::new();
                        
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"id" => attr_id = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"type" => attr_type = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"value" => attr_value = String::from_utf8_lossy(&attr.value).into_owned(),
                                _ => {}
                            }
                        }
                        
                        if let Some(node) = node_stack.last_mut() {
                            node.attributes.push(LsxAttribute {
                                id: attr_id,
                                type_name: attr_type,
                                value: attr_value,
                            });
                        }
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
}
