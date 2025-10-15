//! LSJ to LSX conversion
//! 
//! Key insight: In LSJ, the region IS the root node.
//! LSJ: "regions": { "dialog": { ... } }
//! LSX: <region id="dialog"><node id="dialog">...</node></region>

use crate::error::Result;
use crate::formats::lsj::{LsjDocument, LsjNode as LsjNodeType, LsjAttribute as LsjAttrType};
use crate::formats::lsx::{self, LsxDocument, LsxRegion, LsxNode, LsxAttribute};
use std::path::Path;

/// Convert LSJ file to LSX format
pub fn convert_lsj_to_lsx<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    tracing::info!("Converting LSJ→LSX: {:?} → {:?}", source.as_ref(), dest.as_ref());
    
    let lsj_doc = crate::formats::lsj::read_lsj(&source)?;
    let lsx_doc = to_lsx(&lsj_doc)?;
    
    // Use proper LSX writer
    lsx::write_lsx(&lsx_doc, dest)?;
    
    tracing::info!("Conversion complete");
    Ok(())
}

/// Convert LSJ document to LSX document
pub fn to_lsx(lsj: &LsjDocument) -> Result<LsxDocument> {
    let (major, minor, revision, build) = lsj.parse_version();
    
    let mut regions = Vec::new();
    
    for (region_name, region_node) in &lsj.save.regions {
        let mut lsx_region = LsxRegion {
            id: region_name.clone(),
            nodes: Vec::new(),
        };
        
        // The region in LSJ represents the root node in LSX
        let root_node = convert_region_to_node(region_name, region_node)?;
        lsx_region.nodes.push(root_node);
        
        regions.push(lsx_region);
    }
    
    Ok(LsxDocument {
        major,
        minor,
        revision,
        build,
        regions,
    })
}

/// Convert a region (which is a node in LSJ) to an LSX node
fn convert_region_to_node(node_name: &str, lsj_node: &LsjNodeType) -> Result<LsxNode> {
    let mut lsx_node = LsxNode {
        id: node_name.to_string(),
        key: None,
        attributes: Vec::new(),
        children: Vec::new(),
    };
    
    // Convert attributes
    for (attr_name, attr) in &lsj_node.attributes {
        // Skip the "key" attribute if it exists - it's metadata, not a real attribute
        if attr_name == "key" {
            if let LsjAttrType::Simple { value, .. } = attr {
                lsx_node.key = Some(json_value_to_string(value));
            }
            continue;
        }
        
        let lsx_attr = convert_attribute(attr_name, attr)?;
        lsx_node.attributes.push(lsx_attr);
    }
    
    // Convert children - each array becomes child nodes
    for (child_name, child_array) in &lsj_node.children {
        for child in child_array {
            let child_node = convert_region_to_node(child_name, child)?;
            lsx_node.children.push(child_node);
        }
    }
    
    Ok(lsx_node)
}

fn convert_attribute(name: &str, attr: &LsjAttrType) -> Result<LsxAttribute> {
    match attr {
        LsjAttrType::Simple { type_name, value } => {
            Ok(LsxAttribute {
                id: name.to_string(),
                type_name: type_name.clone(),
                value: json_value_to_string(value),
                handle: None,
                version: None,
            })
        }
        LsjAttrType::TranslatedString { type_name, value, handle, version } => {
            Ok(LsxAttribute {
                id: name.to_string(),
                type_name: type_name.clone(),
                value: value.clone().unwrap_or_default(),
                handle: Some(handle.clone()),
                version: *version,
            })
        }
        LsjAttrType::TranslatedFSString { type_name, value, handle, .. } => {
            Ok(LsxAttribute {
                id: name.to_string(),
                type_name: type_name.clone(),
                value: value.clone().unwrap_or_default(),
                handle: Some(handle.clone()),
                version: None,
            })
        }
    }
}

fn json_value_to_string(value: &serde_json::Value) -> String {
    use serde_json::Value;
    
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Null => String::new(),
        _ => value.to_string(),
    }
}